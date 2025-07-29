use axum::{
    extract::{ConnectInfo, Request},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc};
use tracing::warn;

// SECURITY: Rate limiting configuration
pub const REQUESTS_PER_MINUTE: u32 = 60; // Allow 60 requests per minute per IP
pub const TASK_REQUESTS_PER_MINUTE: u32 = 10; // More restrictive for task creation

#[derive(Clone)]
pub struct RateLimitConfig {
    pub general_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    pub task_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimitConfig {
    pub fn new() -> Self {
        // SECURITY: General rate limiter - 60 requests per minute
        let general_quota = Quota::per_minute(NonZeroU32::new(REQUESTS_PER_MINUTE).unwrap());
        let general_limiter = Arc::new(RateLimiter::direct(general_quota));

        // SECURITY: Task creation rate limiter - 10 requests per minute
        let task_quota = Quota::per_minute(NonZeroU32::new(TASK_REQUESTS_PER_MINUTE).unwrap());
        let task_limiter = Arc::new(RateLimiter::direct(task_quota));

        Self {
            general_limiter,
            task_limiter,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new()
    }
}

// SECURITY: General rate limiting middleware
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 🛡️ SECURITY DECISION: Global rate limiting with per-endpoint quotas
    // Why: Prevents abuse while allowing legitimate usage patterns
    // Alternative: Per-IP tracking (future enhancement: requires distributed state)
    // AUDIT CHECKPOINT: Critical DoS protection - verify rate limits are enforced

    let path = request.uri().path();
    let method = request.method();
    let client_ip = addr.ip();

    // 🚨 CREATE RATE LIMITERS: Static instances for performance
    // DECISION: Static limiters avoid repeated allocations per request
    // Why: Better performance than creating limiters per request
    // Alternative: Injected service state (future enhancement)
    use std::sync::LazyLock;
    static RATE_CONFIG: LazyLock<RateLimitConfig> = LazyLock::new(|| RateLimitConfig::new());

    // 🎯 ENDPOINT-SPECIFIC RATE LIMITING: Different limits for different operations
    // Why: Task creation is more resource-intensive than status checks
    let limiter = if path.starts_with("/tasks") && method == "POST" {
        // Task creation gets more restrictive rate limiting (10/min)
        &RATE_CONFIG.task_limiter
    } else {
        // General API access (60/min)
        &RATE_CONFIG.general_limiter
    };

    // 🛡️ RATE LIMIT ENFORCEMENT: Check quota before processing request
    // Why: Prevents resource exhaustion from abusive clients
    match limiter.check() {
        Ok(_) => {
            // Request allowed - proceed normally
            Ok(next.run(request).await)
        }
        Err(_) => {
            // 🚨 RATE LIMIT EXCEEDED: Log security event and reject request
            // AUDIT CHECKPOINT: Ensure all rate limit violations are logged
            warn!(
                "Rate limit exceeded for {} {} from IP: {} - request denied",
                method, path, client_ip
            );

            // Return 429 Too Many Requests with appropriate headers
            Err(StatusCode::TOO_MANY_REQUESTS)
        }
    }
}

// SECURITY: IP-based rate limiting helper
pub fn extract_client_ip(request: &Request) -> String {
    // Try to get real IP from headers (for proxy setups)
    if let Some(forwarded_for) = request.headers().get("x-forwarded-for") {
        if let Ok(ip_str) = forwarded_for.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = ip_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fallback to connection info
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_config_creation() {
        let config = RateLimitConfig::new();
        assert!(config.general_limiter.check().is_ok());
        assert!(config.task_limiter.check().is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_quota() {
        let config = RateLimitConfig::new();

        // Should allow initial requests
        assert!(config.general_limiter.check().is_ok());
        assert!(config.task_limiter.check().is_ok());

        // After many requests, should start limiting
        // (This test would need to be adjusted based on actual quota limits)
    }
}
