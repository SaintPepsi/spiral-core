use crate::config::ApiConfig;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use std::sync::Arc;
use tracing::warn;

#[derive(Clone)]
pub struct AuthState {
    pub config: ApiConfig,
}

/// 🔐 AUTHENTICATION MIDDLEWARE: Primary security enforcement point
/// AUDIT CHECKPOINT: Critical security boundary - all requests pass through here
/// Verify: API key validation, header parsing, authentication logging, timing attack prevention
pub async fn auth_middleware(
    State(auth_state): State<Arc<AuthState>>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, Response> {
    let method = request.method().clone();
    let path = request.uri().path();
    let client_ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    tracing::debug!(
        "Auth middleware processing request to: {} from IP: {}",
        path,
        client_ip
    );

    // 🌐 CORS PREFLIGHT BYPASS: Allow OPTIONS requests to proceed without auth
    // REASONING: CORS preflight requests need to succeed to enable browser CORS
    if method == Method::OPTIONS {
        tracing::debug!("Bypassing auth for CORS preflight request to: {}", path);
        return Ok(next.run(request).await);
    }

    // 🛡️ SECURITY POLICY AUDIT CHECKPOINT: No bypass paths allowed
    // CRITICAL: Every endpoint requires authentication to prevent unauthorized access
    // Previous vulnerability: Health check bypass removed for security hardening

    // 🔑 API KEY EXTRACTION: Support multiple authentication header formats
    // AUDIT: Verify both x-api-key and Authorization header handling
    let provided_key = if let Some(header_value) = headers.get("x-api-key") {
        // Direct API key header
        header_value.to_str().map_err(|_| {
            warn!(
                "Malformed x-api-key header from IP: {} for path: {}",
                client_ip, path
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Unauthorized"})),
            )
                .into_response()
        })?
    } else if let Some(header_value) = headers.get("authorization") {
        // Authorization header - must start with "Bearer "
        let auth_str = header_value.to_str().map_err(|_| {
            warn!(
                "Malformed authorization header from IP: {} for path: {}",
                client_ip, path
            );
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Unauthorized"})),
            )
                .into_response()
        })?;

        // 🏷️ BEARER TOKEN SUPPORT: Standard OAuth-style authentication
        if let Some(token) = auth_str.strip_prefix("Bearer ") {
            token
        } else {
            // 🚨 SECURITY: Reject authorization headers without proper Bearer prefix
            warn!(
                "Invalid authorization header format from IP: {} for path: {}",
                client_ip, path
            );
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Unauthorized"})),
            )
                .into_response());
        }
    } else {
        warn!("Missing API key in request to: {}", path);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({"error": "Unauthorized"})),
        )
            .into_response());
    };

    // 🔐 VALIDATE API KEY
    match &auth_state.config.api_key {
        Some(expected_key) => {
            // 🔐 CONSTANT-TIME COMPARISON AUDIT CHECKPOINT: Prevent timing attacks
            // CRITICAL: Use secure comparison to prevent API key extraction via timing
            // SECURITY DECISION: Use constant-time comparison to prevent timing attacks
            // Why: Prevents attackers from determining correct API key characters via timing analysis
            // Alternative: Regular `==` (rejected: vulnerable to timing attacks)
            use subtle::ConstantTimeEq;
            if provided_key
                .as_bytes()
                .ct_eq(expected_key.as_bytes())
                .into()
            {
                // ✅ AUTHENTICATION SUCCESS: Proceed to next middleware/handler
                tracing::debug!(
                    "Authentication successful for path: {} from IP: {}",
                    path,
                    client_ip
                );
                Ok(next.run(request).await)
            } else {
                // 🚨 AUTHENTICATION FAILURE AUDIT CHECKPOINT: Invalid credentials
                // CRITICAL: Log for security monitoring but don't reveal details
                warn!(
                    "Authentication failed for path: {} from IP: {} (invalid key)",
                    path, client_ip
                );
                Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Unauthorized"})),
                )
                    .into_response())
            }
        }
        None => {
            warn!("API authentication enabled but no API key configured");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Internal Server Error"})),
            )
                .into_response())
        }
    }
}

pub fn create_auth_state(config: ApiConfig) -> Arc<AuthState> {
    Arc::new(AuthState { config })
}
