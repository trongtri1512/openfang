//! Tenant Self-Service Portal — role-based access for tenant members.
//!
//! Provides a separate `/portal/` UI and `/api/portal/*` API endpoints
//! for tenant members to view/manage their assigned tenants.
//!
//! Authentication uses a simple token-based session (base64-encoded JSON).
//! Roles: "admin" sees all tenants, "member" sees only assigned tenants.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use crate::routes::AppState;
use crate::tenants::{load_tenants, save_tenants, verify_password, hash_password};

// ---------------------------------------------------------------------------
// Session token (simple base64-encoded JSON, not full JWT for simplicity)
// ---------------------------------------------------------------------------

const SESSION_SECRET: &str = "openfang_portal_v1";
const SESSION_EXPIRY_SECS: i64 = 86400; // 24 hours

#[derive(Debug, Serialize, Deserialize)]
struct SessionPayload {
    email: String,
    role: String,
    /// Tenant IDs this member belongs to (empty for admin = all access).
    tenant_ids: Vec<String>,
    /// Expiry timestamp (Unix seconds).
    exp: i64,
}

/// Create a session token from payload.
fn create_session_token(payload: &SessionPayload) -> String {
    use base64::Engine;
    let json = serde_json::to_string(payload).unwrap_or_default();
    let signature = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        SESSION_SECRET.hash(&mut h);
        json.hash(&mut h);
        format!("{:016x}", h.finish())
    };
    let combined = format!("{}.{}", json, signature);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(combined)
}

/// Verify and decode a session token. Returns None if invalid or expired.
fn verify_session_token(token: &str) -> Option<SessionPayload> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(token).ok()?;
    let combined = String::from_utf8(decoded).ok()?;
    let dot = combined.rfind('.')?;
    let json = &combined[..dot];
    let sig = &combined[dot + 1..];

    // Verify signature
    let expected_sig = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        SESSION_SECRET.hash(&mut h);
        json.hash(&mut h);
        format!("{:016x}", h.finish())
    };
    if sig != expected_sig {
        return None;
    }

    let payload: SessionPayload = serde_json::from_str(json).ok()?;

    // Check expiry
    let now = chrono::Utc::now().timestamp();
    if now > payload.exp {
        return None;
    }

    Some(payload)
}

/// Extract session from Authorization header.
fn extract_session(headers: &axum::http::HeaderMap) -> Option<SessionPayload> {
    let auth = headers.get("authorization")?.to_str().ok()?;
    let token = auth.strip_prefix("Bearer ")?;
    verify_session_token(token)
}

// ---------------------------------------------------------------------------
// Request/response types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PortalLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct SetPasswordRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// GET /portal/ — Serve Portal login/dashboard page.
pub async fn portal_page() -> impl IntoResponse {
    (
        [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
        axum::response::Html(PORTAL_HTML),
    )
}

/// POST /api/portal/login — Authenticate member, return session token.
///
/// Super Admin: if password matches the system API key, grants full admin access.
pub async fn portal_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PortalLoginRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || req.password.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Email and password are required"})),
        ).into_response();
    }

    // ── Super Admin: password == system API key ──
    let system_api_key = &state.kernel.config.api_key;
    if !system_api_key.is_empty() && req.password == *system_api_key {
        let payload = SessionPayload {
            email: email.clone(),
            role: "admin".to_string(),
            tenant_ids: vec![], // empty = access all
            exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS,
        };
        let token = create_session_token(&payload);

        info!(email = %email, "Super admin portal login via API key");

        return Json(serde_json::json!({
            "token": token,
            "email": email,
            "role": "admin",
            "display_name": "System Admin",
            "expires_in": SESSION_EXPIRY_SECS,
        })).into_response();
    }

    // ── Normal member login ──
    let mut data = load_tenants(&state);
    let mut found_role = String::new();
    let mut tenant_ids: Vec<String> = Vec::new();
    let mut display_name = String::new();
    let mut matched = false;

    // Search across ALL tenants for this email
    for tenant in &data.tenants {
        for member in &tenant.members {
            if member.email.to_lowercase() == email {
                if let Some(hash) = &member.password_hash {
                    if verify_password(&req.password, hash) {
                        matched = true;
                        if found_role.is_empty() || member.role == "admin" {
                            found_role = member.role.clone();
                        }
                        if display_name.is_empty() {
                            display_name = member.display_name.clone().unwrap_or_else(|| email.clone());
                        }
                        tenant_ids.push(tenant.id.clone());
                    }
                }
            }
        }
    }

    if !matched {
        return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid email or password"})),
        ).into_response();
    }

    // Update last_login
    for tenant in &mut data.tenants {
        for member in &mut tenant.members {
            if member.email.to_lowercase() == email {
                member.last_login = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
            }
        }
    }
    let _ = save_tenants(&state, &data);

    let payload = SessionPayload {
        email: email.clone(),
        role: found_role.clone(),
        tenant_ids: if found_role == "admin" { vec![] } else { tenant_ids },
        exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS,
    };
    let token = create_session_token(&payload);

    info!(email = %email, role = %found_role, "Portal login successful");

    Json(serde_json::json!({
        "token": token,
        "email": email,
        "role": found_role,
        "display_name": display_name,
        "expires_in": SESSION_EXPIRY_SECS,
    })).into_response()
}

/// GET /api/portal/me — Get current session info.
pub async fn portal_me(headers: axum::http::HeaderMap) -> impl IntoResponse {
    match extract_session(&headers) {
        Some(s) => Json(serde_json::json!({"email":s.email,"role":s.role,"tenant_ids":s.tenant_ids})).into_response(),
        None => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    }
}

/// GET /api/portal/tenants — List tenants (filtered by role).
pub async fn portal_tenants(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    };
    let data = load_tenants(&state);
    let tenants: Vec<serde_json::Value> = data.tenants.iter()
        .filter(|t| session.role == "admin" || session.tenant_ids.contains(&t.id))
        .map(|t| serde_json::json!({
            "id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,
            "provider":t.provider,"model":t.model,"messages_today":t.messages_today,
            "max_messages_per_day":t.max_messages_per_day,"channels_active":t.channels_active,
            "members_count":t.members.len(),"created_at":t.created_at,
        }))
        .collect();
    Json(serde_json::json!({"tenants":tenants})).into_response()
}

/// GET /api/portal/tenants/:id — Tenant detail (if authorized).
pub async fn portal_tenant_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    };
    if session.role != "admin" && !session.tenant_ids.contains(&id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => {
            let members: Vec<serde_json::Value> = t.members.iter().map(|m| serde_json::json!({
                "email":m.email,"role":m.role,"display_name":m.display_name,
                "added_at":m.added_at,"last_login":m.last_login,"has_password":m.password_hash.is_some(),
            })).collect();
            Json(serde_json::json!({
                "id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,
                "provider":t.provider,"model":t.model,"temperature":t.temperature,
                "messages_today":t.messages_today,"max_messages_per_day":t.max_messages_per_day,
                "max_channels":t.max_channels,"max_members":t.max_members,
                "channels":t.channels,"members":members,"created_at":t.created_at,
            })).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

/// GET /api/portal/members — Global members list (admin only).
pub async fn portal_all_members(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    };
    if session.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response();
    }
    let data = load_tenants(&state);
    let mut map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    for tenant in &data.tenants {
        for member in &tenant.members {
            let key = member.email.to_lowercase();
            let entry = map.entry(key.clone()).or_insert_with(|| serde_json::json!({
                "email":member.email,"display_name":member.display_name,"role":member.role,
                "has_password":member.password_hash.is_some(),"last_login":member.last_login,"tenants":[],
            }));
            if let Some(arr) = entry["tenants"].as_array_mut() {
                arr.push(serde_json::json!({"id":tenant.id,"name":tenant.name,"role":member.role}));
            }
            if member.role == "admin" { entry["role"] = serde_json::json!("admin"); }
        }
    }
    let members: Vec<serde_json::Value> = map.into_values().collect();
    Json(serde_json::json!({"members":members})).into_response()
}

/// POST /api/portal/tenants/:id/set-password — Set member password (admin only).
pub async fn portal_set_password(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(req): Json<SetPasswordRequest>,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    };
    if session.role != "admin" {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response();
    }
    if req.password.len() < 4 {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Password must be at least 4 characters"}))).into_response();
    }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    };
    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) {
        Some(m) => m,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response(),
    };
    member.password_hash = Some(hash_password(&req.password));
    if let Some(name) = req.display_name { member.display_name = Some(name); }
    let _ = save_tenants(&state, &data);
    info!(email = %req.email, tenant = %id, "Set portal password");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ---------------------------------------------------------------------------
// Embedded Portal HTML (login + dashboard SPA)
// ---------------------------------------------------------------------------

const PORTAL_HTML: &str = r##"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>OpenFang Portal</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--orange:#FF5C00;--orange-hover:#e65200;--bg:#fff;--bg2:#f9fafb;--text:#111827;--dim:#6b7280;--muted:#9ca3af;--border:#e5e7eb;--dark:#0f0f23;--dark2:#1a1a2e;--dborder:#2a2a4a;--dtext:#e0e0e0;--ddim:#8888aa;--green:#00d4aa}
body{font-family:'Inter',system-ui,-apple-system,sans-serif;margin:0;min-height:100vh}
.login-screen{display:flex;min-height:100vh}
.login-left{flex:1;background:var(--bg2);position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden}
.login-left::before{content:'';position:absolute;inset:0;background-image:linear-gradient(rgba(0,0,0,.03) 1px,transparent 1px),linear-gradient(90deg,rgba(0,0,0,.03) 1px,transparent 1px);background-size:40px 40px;pointer-events:none}
.login-left>*{position:relative;z-index:1}
.brand{display:flex;align-items:center;gap:10px;margin-bottom:40px}
.brand svg{width:36px;height:36px}.brand span{font-size:1.4rem;font-weight:700;letter-spacing:-.5px}
.login-left h2{font-size:2.2rem;font-weight:700;line-height:1.2;letter-spacing:-1px;margin-bottom:16px}
.login-left h2 .hl{color:var(--orange)}
.login-left .desc{color:var(--dim);font-size:.95rem;line-height:1.6;margin-bottom:40px}
.terminal-card{background:var(--bg);border:1px solid var(--border);border-radius:12px;overflow:hidden;box-shadow:0 4px 24px rgba(0,0,0,.06);margin-bottom:40px}
.terminal-dots{display:flex;gap:6px;padding:12px 16px;border-bottom:1px solid var(--border)}
.terminal-dots span{width:10px;height:10px;border-radius:50%}
.terminal-dots span:nth-child(1){background:#ff5f57}.terminal-dots span:nth-child(2){background:#febc2e}.terminal-dots span:nth-child(3){background:#28c840}
.terminal-code{padding:16px 20px;font-family:'JetBrains Mono',monospace;font-size:.8rem;line-height:1.8;color:var(--dim)}
.terminal-code .prompt{color:var(--text);font-weight:500}.terminal-code .cmd{color:var(--orange)}
.metrics{display:flex;gap:32px}
.metric .val{font-size:1.5rem;font-weight:700;letter-spacing:-.5px}.metric .val .unit{color:var(--orange);font-weight:600}
.metric .lbl{font-size:.75rem;color:var(--muted);margin-top:2px}
.login-right{width:480px;display:flex;flex-direction:column;justify-content:center;padding:48px;position:relative}
.login-right::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at top right,rgba(255,92,0,.04),transparent 60%);pointer-events:none}
.login-right>*{position:relative;z-index:1}
.brand-sm{display:flex;align-items:center;gap:8px;justify-content:center;margin-bottom:32px}
.brand-sm svg{width:28px;height:28px}.brand-sm span{font-size:1.1rem;font-weight:700;letter-spacing:-.3px}
.login-right h1{font-size:1.75rem;font-weight:700;letter-spacing:-.5px;margin-bottom:8px}
.login-right .subtitle{color:var(--dim);font-size:.9rem;margin-bottom:32px}
.form-group{margin-bottom:16px}
.form-group label{display:block;font-size:.8rem;font-weight:500;color:var(--dim);margin-bottom:6px}
.input-wrap{position:relative}
.input-wrap input{width:100%;padding:12px 16px 12px 44px;border:1px solid var(--border);border-radius:12px;font-size:.9rem;font-family:inherit;color:var(--text);outline:none;transition:border-color .2s,box-shadow .2s}
.input-wrap input:focus{border-color:var(--orange);box-shadow:0 0 0 3px rgba(255,92,0,.1)}
.input-wrap input::placeholder{color:var(--muted)}
.input-wrap .icon{position:absolute;left:14px;top:50%;transform:translateY(-50%);color:var(--muted)}
.btn-login{width:100%;padding:14px;background:var(--orange);color:#fff;border:none;border-radius:12px;font-size:.95rem;font-weight:600;font-family:inherit;cursor:pointer;transition:background .2s,transform .1s;margin-top:8px}
.btn-login:hover{background:var(--orange-hover)}.btn-login:active{transform:scale(.99)}.btn-login:disabled{opacity:.5;cursor:not-allowed}
.error-msg{color:#ef4444;font-size:.8rem;margin-top:12px;display:none}
.login-footer{margin-top:24px;text-align:center;font-size:.8rem;color:var(--muted)}
.login-footer a{color:var(--orange);text-decoration:none;font-weight:500}
.dashboard{display:none;height:100vh;overflow:hidden;background:var(--dark);color:var(--dtext)}
.dash-header{background:var(--dark2);border-bottom:1px solid var(--dborder);padding:12px 20px;display:flex;align-items:center;justify-content:space-between}
.dash-header h1{font-size:1rem;display:flex;align-items:center;gap:8px}
.dash-header .user-info{display:flex;align-items:center;gap:12px;font-size:.85rem}
.dash-header .badge{background:var(--orange);color:#fff;padding:2px 8px;border-radius:4px;font-size:.7rem;font-weight:600;text-transform:uppercase}
.btn-ghost{background:none;border:1px solid var(--dborder);color:var(--dtext);padding:6px 12px;border-radius:6px;cursor:pointer;font-size:.8rem;font-family:inherit}
.btn-ghost:hover{border-color:var(--orange);color:var(--orange)}
.dash-body{display:flex;height:calc(100vh - 50px)}
.sidebar{width:260px;background:var(--dark2);border-right:1px solid var(--dborder);overflow-y:auto;flex-shrink:0;padding:12px}
.sidebar h3{font-size:.7rem;color:var(--ddim);text-transform:uppercase;padding:8px 12px;letter-spacing:.5px;font-weight:600}
.tenant-item{padding:10px 12px;border-radius:8px;cursor:pointer;font-size:.85rem;margin-bottom:2px;display:flex;align-items:center;gap:8px;transition:background .15s;color:var(--dtext)}
.tenant-item:hover{background:#16213e}.tenant-item.active{background:var(--orange);color:#fff}
.tenant-item .dot{width:8px;height:8px;border-radius:50%;flex-shrink:0}
.tenant-item .dot.running{background:var(--green)}.tenant-item .dot.stopped{background:var(--ddim)}
.content{flex:1;overflow-y:auto;padding:24px}
.stat-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px;margin-bottom:24px}
.stat-card{background:var(--dark2);border:1px solid var(--dborder);border-radius:10px;padding:16px}
.stat-card .label{font-size:.75rem;color:var(--ddim);margin-bottom:4px}.stat-card .value{font-size:1.4rem;font-weight:700}
.members-table{width:100%;border-collapse:collapse;font-size:.85rem}
.members-table th,.members-table td{padding:10px 12px;text-align:left;border-bottom:1px solid var(--dborder)}
.members-table th{color:var(--ddim);font-weight:600;font-size:.75rem;text-transform:uppercase}
.members-table tr:hover{background:#16213e}
.role-badge{padding:2px 8px;border-radius:4px;font-size:.7rem;text-transform:uppercase;font-weight:600}
.role-badge.admin{background:rgba(255,92,0,.15);color:var(--orange)}.role-badge.member{background:rgba(0,212,170,.15);color:var(--green)}
@media(max-width:900px){.login-screen{flex-direction:column}.login-left{display:none}.login-right{width:100%;min-height:100vh}.sidebar{display:none}.stat-grid{grid-template-columns:1fr}}
</style>
</head>
<body>
<div class="login-screen" id="loginView">
  <div class="login-left">
    <div class="brand"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h2>Deploy &amp; manage<br>AI agents with <span class="hl">the official<br>OpenFang runtime</span></h2>
    <p class="desc">Self-service portal for team members. Manage your tenants, view analytics, and collaborate securely.</p>
    <div class="terminal-card">
      <div class="terminal-dots"><span></span><span></span><span></span></div>
      <div class="terminal-code"><div><span class="prompt">$</span> <span class="cmd">openfang serve</span></div><div>booted in &lt;200ms</div><div>hands 7 active</div><div>gateway ready :3000</div></div>
    </div>
    <div class="metrics">
      <div class="metric"><div class="val">32 <span class="unit">MB</span></div><div class="lbl">Binary</div></div>
      <div class="metric"><div class="val">180<span class="unit">ms</span></div><div class="lbl">Cold Start</div></div>
      <div class="metric"><div class="val">26<span class="unit">+</span></div><div class="lbl">Providers</div></div>
    </div>
  </div>
  <div class="login-right">
    <div class="brand-sm"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h1>Welcome back</h1>
    <p class="subtitle">Sign in to manage your tenants and agents.</p>
    <div class="form-group"><label>Email address</label><div class="input-wrap"><svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg><input type="email" id="loginEmail" placeholder="admin@trongtri.com" autocomplete="email"></div></div>
    <div class="form-group"><label>Password</label><div class="input-wrap"><svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg><input type="password" id="loginPass" placeholder="Enter your password or API key" autocomplete="current-password" onkeydown="if(event.key==='Enter')doLogin()"></div></div>
    <button class="btn-login" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="error-msg" id="loginError"></div>
    <div class="login-footer">System admins can use their <a href="javascript:void(0)">API key</a> as password</div>
  </div>
</div>
<div class="dashboard" id="dashView">
  <div class="dash-header"><h1>🏢 OpenFang Portal</h1><div class="user-info"><span id="userName"></span><span class="badge" id="userRole"></span><button class="btn-ghost" onclick="doLogout()">Logout</button></div></div>
  <div class="dash-body">
    <div class="sidebar"><h3>Tenants</h3><div id="tenantList"></div></div>
    <div class="content" id="contentArea"><div style="text-align:center;padding:60px;color:var(--ddim)"><h2 style="margin-bottom:8px">👈 Select a tenant</h2><p>Choose a tenant from the sidebar to view details.</p></div></div>
  </div>
</div>
<script>
let session=null,tenants=[],cur=null;
function api(m,p,b){const o={method:m,headers:{'Content-Type':'application/json'}};if(session)o.headers.Authorization='Bearer '+session.token;if(b)o.body=JSON.stringify(b);return fetch(p,o).then(r=>r.json())}
async function doLogin(){const e=document.getElementById('loginEmail').value.trim(),p=document.getElementById('loginPass').value,err=document.getElementById('loginError');err.style.display='none';if(!e||!p){err.textContent='Please fill in all fields';err.style.display='block';return}document.getElementById('loginBtn').disabled=true;try{const d=await api('POST','/api/portal/login',{email:e,password:p});if(d.error){err.textContent=d.error;err.style.display='block';return}session=d;localStorage.setItem('portal_session',JSON.stringify(d));showDash()}catch(x){err.textContent='Connection error';err.style.display='block'}finally{document.getElementById('loginBtn').disabled=false}}
function doLogout(){session=null;localStorage.removeItem('portal_session');document.getElementById('loginView').style.display='flex';document.getElementById('dashView').style.display='none'}
async function showDash(){document.getElementById('loginView').style.display='none';document.getElementById('dashView').style.display='block';document.getElementById('userName').textContent=session.display_name||session.email;document.getElementById('userRole').textContent=session.role;await loadTenants()}
async function loadTenants(){const d=await api('GET','/api/portal/tenants');tenants=d.tenants||[];document.getElementById('tenantList').innerHTML=tenants.map(t=>`<div class="tenant-item" onclick="selectTenant('${t.id}')"><span class="dot ${t.status==='running'?'running':'stopped'}"></span><span>${t.name}</span></div>`).join('')}
async function selectTenant(id){const d=await api('GET','/api/portal/tenants/'+id);if(d.error)return;cur=d;document.querySelectorAll('.tenant-item').forEach(el=>el.classList.toggle('active',el.onclick.toString().includes(id)));renderDetail(d)}
function renderDetail(t){const mh=t.members?t.members.map(m=>`<tr><td>${m.display_name||m.email}</td><td>${m.email}</td><td><span class="role-badge ${m.role}">${m.role}</span></td><td>${m.has_password?'✅':'❌'}</td><td>${m.last_login||'Never'}</td></tr>`).join(''):'';document.getElementById('contentArea').innerHTML=`<h2 style="margin-bottom:4px">${t.name}</h2><p style="color:var(--ddim);font-size:.85rem;margin-bottom:20px">${t.slug} · ${t.plan}</p><div class="stat-grid"><div class="stat-card"><div class="label">Provider</div><div class="value" style="font-size:1rem">${t.provider}</div></div><div class="stat-card"><div class="label">Model</div><div class="value" style="font-size:1rem">${t.model}</div></div><div class="stat-card"><div class="label">Messages Today</div><div class="value">${t.messages_today}</div></div><div class="stat-card"><div class="label">Daily Limit</div><div class="value">${t.max_messages_per_day>=4294967295?'∞':t.max_messages_per_day}</div></div><div class="stat-card"><div class="label">Channels</div><div class="value">${(t.channels||[]).length}</div></div><div class="stat-card"><div class="label">Members</div><div class="value">${(t.members||[]).length}</div></div></div><h3 style="margin-bottom:12px">Members</h3><table class="members-table"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Last Login</th></tr></thead><tbody>${mh}</tbody></table>`}
(function(){const s=localStorage.getItem('portal_session');if(s){try{session=JSON.parse(s);showDash()}catch(e){localStorage.removeItem('portal_session')}}})();
</script>
</body></html>"##;
