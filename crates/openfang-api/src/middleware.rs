//! Production middleware for the OpenFang API server.
//!
//! Provides:
//! - Request ID generation and propagation
//! - Per-endpoint structured request logging
//! - In-memory rate limiting (per IP)
//! - Bearer token authentication (global key + per-user RBAC keys)

use axum::body::Body;
use axum::http::{Request, Response, StatusCode};
use axum::middleware::Next;
use std::time::Instant;
use tracing::info;

/// Request ID header name (standard).
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Authentication configuration passed as middleware state.
///
/// Holds the global API key for backward compatibility and a reference to
/// the kernel's AuthManager for per-user API key authentication.
#[derive(Clone)]
pub struct AuthConfig {
    /// Global API key (empty = no global auth).
    pub global_api_key: String,
    /// Arc reference to the kernel for accessing the AuthManager.
    pub kernel: std::sync::Arc<openfang_kernel::OpenFangKernel>,
}

/// Authenticated user identity injected into request extensions.
///
/// Downstream handlers can extract this to determine who made the request
/// and what role they have.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    /// User display name ("admin" for global key, or the configured user name).
    pub name: String,
    /// User role string ("owner", "admin", "user", "viewer").
    pub role: String,
}

/// Middleware: inject a unique request ID and log the request/response.
pub async fn request_logging(request: Request<Body>, next: Next) -> Response<Body> {
    let request_id = uuid::Uuid::new_v4().to_string();
    let method = request.method().clone();
    let uri = request.uri().path().to_string();
    let start = Instant::now();

    let mut response = next.run(request).await;

    let elapsed = start.elapsed();
    let status = response.status().as_u16();

    info!(
        request_id = %request_id,
        method = %method,
        path = %uri,
        status = status,
        latency_ms = elapsed.as_millis() as u64,
        "API request"
    );

    // Inject the request ID into the response
    if let Ok(header_val) = request_id.parse() {
        response.headers_mut().insert(REQUEST_ID_HEADER, header_val);
    }

    response
}

/// Bearer token authentication middleware.
///
/// Authentication flow:
/// 1. If no global key AND no per-user keys → restrict to loopback only
/// 2. Check public endpoints that don't require auth
/// 3. Extract Bearer token (or X-API-Key header, or ?token= query param)
/// 4. Match against global `api_key` (backward compatible) → role: owner
/// 5. Match against per-user `api_key_hash` via AuthManager → role from config
/// 6. Inject `AuthenticatedUser` into request extensions for downstream handlers
pub async fn auth(
    axum::extract::State(config): axum::extract::State<AuthConfig>,
    mut request: Request<Body>,
    next: Next,
) -> Response<Body> {
<<<<<<< HEAD
    let has_global_key = !config.global_api_key.is_empty();
    let has_user_keys = config.kernel.auth.is_enabled();

    // If no API key configured AND no per-user keys, restrict to loopback addresses only.
    if !has_global_key && !has_user_keys {
=======
    // If no API key configured, skip authentication entirely (open access).
    if api_key.is_empty() {
        return next.run(request).await;
    }

    // Shutdown is loopback-only (CLI on same machine) — skip token auth
    let path = request.uri().path();
    if path == "/api/shutdown" {
>>>>>>> b2e2b1a038ffd5e3e4ca61e65cf1a6e14e9b9003
        let is_loopback = request
            .extensions()
            .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
            .map(|ci| ci.0.ip().is_loopback())
            .unwrap_or(true); // default true for unix sockets / tests
        if is_loopback {
            return next.run(request).await;
        }
<<<<<<< HEAD

        // Loopback with no auth → treat as owner
        request.extensions_mut().insert(AuthenticatedUser {
            name: "local".to_string(),
            role: "owner".to_string(),
        });
        return next.run(request).await;
=======
>>>>>>> b2e2b1a038ffd5e3e4ca61e65cf1a6e14e9b9003
    }

    // Public endpoints that don't require auth (dashboard needs these)
    if path == "/"
        || path == "/logo.png"
        || path == "/favicon.ico"
        || path == "/.well-known/agent.json"
        || path.starts_with("/a2a/")
        || path == "/api/health"
        || path == "/api/health/detail"
        || path == "/api/status"
        || path == "/api/version"
        || path == "/api/agents"
        || path == "/api/profiles"
        || path == "/api/config"
        || path.starts_with("/api/uploads/")
        // Auth endpoints must be accessible without auth
        || path == "/api/auth/login"
        || path == "/api/auth/me"
        // Dashboard read endpoints — allow unauthenticated so the SPA can
        // render before the user enters their API key.
        || path == "/api/models"
        || path == "/api/models/aliases"
        || path == "/api/providers"
        || path == "/api/budget"
        || path == "/api/budget/agents"
        || path.starts_with("/api/budget/agents/")
        || path == "/api/network/status"
        || path == "/api/a2a/agents"
        || path == "/api/approvals"
        || path.starts_with("/api/approvals/")
        || path == "/api/channels"
        || path == "/api/hands"
        || path == "/api/hands/active"
        || path.starts_with("/api/hands/")
        || path == "/api/skills"
        || path == "/api/sessions"
        || path == "/api/integrations"
        || path == "/api/integrations/available"
        || path == "/api/integrations/health"
        || path == "/api/workflows"
        || path == "/api/logs/stream"
        || path.starts_with("/api/cron/")
        || path.starts_with("/api/providers/github-copilot/oauth/")
        // Tenant magic access link — auth via ?t= query param
        || path == "/access/"
        || path == "/api/access/chat"
        // Portal — uses its own session-based auth
        || path == "/portal"
        || path == "/portal/"
        || path.starts_with("/api/portal/")
    {
        return next.run(request).await;
    }

    // Extract token from headers or query params
    let token = extract_token(&request);

    if let Some(token) = token {
        // 1) Check against global API key (constant-time)
        if has_global_key {
            use subtle::ConstantTimeEq;
            let global_match = if token.len() == config.global_api_key.len() {
                token
                    .as_bytes()
                    .ct_eq(config.global_api_key.as_bytes())
                    .into()
            } else {
                false
            };

            if global_match {
                request.extensions_mut().insert(AuthenticatedUser {
                    name: "admin".to_string(),
                    role: "owner".to_string(),
                });
                return next.run(request).await;
            }
        }

        // 2) Check against per-user API key hashes
        if has_user_keys {
            if let Some(user) = config.kernel.auth.authenticate_by_api_key(token) {
                request.extensions_mut().insert(AuthenticatedUser {
                    name: user.name.clone(),
                    role: format!("{}", user.role),
                });
                return next.run(request).await;
            }
        }

        // Token provided but not matching anything
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("www-authenticate", "Bearer")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::json!({"error": "Invalid API key"}).to_string(),
            ))
            .unwrap_or_default();
    }

    // No token provided
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("www-authenticate", "Bearer")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::json!({"error": "Missing Authorization: Bearer <api_key> header"})
                .to_string(),
        ))
        .unwrap_or_default()
}

/// Extract Bearer token from request headers or query params.
fn extract_token<'a>(request: &'a Request<Body>) -> Option<&'a str> {
    // Check Authorization: Bearer <token> header
    let bearer = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if bearer.is_some() {
        return bearer;
    }

    // Fallback to X-API-Key header
    let api_key = request
        .headers()
        .get("x-api-key")
        .and_then(|v| v.to_str().ok());

    if api_key.is_some() {
        return api_key;
    }

    // Fallback to ?token= query parameter (for SSE/WebSocket clients)
    request
        .uri()
        .query()
        .and_then(|q| q.split('&').find_map(|pair| pair.strip_prefix("token=")))
}

/// Security headers middleware — applied to ALL API responses.
pub async fn security_headers(request: Request<Body>, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("x-content-type-options", "nosniff".parse().unwrap());
    headers.insert("x-frame-options", "DENY".parse().unwrap());
    headers.insert("x-xss-protection", "1; mode=block".parse().unwrap());
    // All JS/CSS is bundled inline — only external resource is Google Fonts.
    headers.insert(
        "content-security-policy",
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://fonts.gstatic.com; img-src 'self' data: blob:; connect-src 'self' ws://localhost:* ws://127.0.0.1:* wss://localhost:* wss://127.0.0.1:*; font-src 'self' https://fonts.gstatic.com; media-src 'self' blob:; frame-src 'self' blob:; object-src 'none'; base-uri 'self'; form-action 'self'"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "referrer-policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert(
        "cache-control",
        "no-store, no-cache, must-revalidate".parse().unwrap(),
    );
    headers.insert(
        "strict-transport-security",
        "max-age=63072000; includeSubDomains".parse().unwrap(),
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_id_header_constant() {
        assert_eq!(REQUEST_ID_HEADER, "x-request-id");
    }
}
