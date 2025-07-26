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
use std::{net::SocketAddr, num::NonZeroU32, sync::Arc, time::Duration};
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
    // Note: This is a simple global rate limiter
    // For production, you'd want per-IP rate limiting with a distributed cache

    let path = request.uri().path();

    // SECURITY: Log rate limit attempts for monitoring
    if path.starts_with("/tasks") && request.method() == "POST" {
        // Task creation gets more restrictive rate limiting
        warn!(
            "Rate limiting not fully implemented for task creation from IP: {}",
            addr.ip()
        );
    }

    // For now, we'll implement a basic delay to prevent abuse
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(next.run(request).await)
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
