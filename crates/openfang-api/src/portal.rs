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
// Session token
// ---------------------------------------------------------------------------

const SESSION_SECRET: &str = "openfang_portal_v1";
const SESSION_EXPIRY_SECS: i64 = 86400;

#[derive(Debug, Serialize, Deserialize)]
struct SessionPayload {
    email: String,
    role: String,
    tenant_ids: Vec<String>,
    exp: i64,
}

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

fn verify_session_token(token: &str) -> Option<SessionPayload> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(token).ok()?;
    let combined = String::from_utf8(decoded).ok()?;
    let dot = combined.rfind('.')?;
    let json = &combined[..dot];
    let sig = &combined[dot + 1..];
    let expected_sig = {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        SESSION_SECRET.hash(&mut h);
        json.hash(&mut h);
        format!("{:016x}", h.finish())
    };
    if sig != expected_sig { return None; }
    let payload: SessionPayload = serde_json::from_str(json).ok()?;
    if chrono::Utc::now().timestamp() > payload.exp { return None; }
    Some(payload)
}

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

pub async fn portal_page() -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], axum::response::Html(PORTAL_HTML))
}

/// POST /api/portal/login
pub async fn portal_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PortalLoginRequest>,
) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || req.password.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email and password are required"}))).into_response();
    }
    // Super Admin: password == system API key
    let system_api_key = &state.kernel.config.api_key;
    if !system_api_key.is_empty() && req.password == *system_api_key {
        let payload = SessionPayload { email: email.clone(), role: "admin".into(), tenant_ids: vec![], exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS };
        let token = create_session_token(&payload);
        info!(email = %email, "Super admin portal login via API key");
        return Json(serde_json::json!({"token":token,"email":email,"role":"admin","display_name":"System Admin","expires_in":SESSION_EXPIRY_SECS})).into_response();
    }
    // Normal member login
    let mut data = load_tenants(&state);
    let mut found_role = String::new();
    let mut tenant_ids: Vec<String> = Vec::new();
    let mut display_name = String::new();
    let mut matched = false;
    for tenant in &data.tenants {
        for member in &tenant.members {
            if member.email.to_lowercase() == email {
                if let Some(hash) = &member.password_hash {
                    if verify_password(&req.password, hash) {
                        matched = true;
                        if found_role.is_empty() || member.role == "admin" { found_role = member.role.clone(); }
                        if display_name.is_empty() { display_name = member.display_name.clone().unwrap_or_else(|| email.clone()); }
                        tenant_ids.push(tenant.id.clone());
                    }
                }
            }
        }
    }
    if !matched {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid email or password"}))).into_response();
    }
    for tenant in &mut data.tenants {
        for member in &mut tenant.members {
            if member.email.to_lowercase() == email {
                member.last_login = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
            }
        }
    }
    let _ = save_tenants(&state, &data);
    let payload = SessionPayload { email: email.clone(), role: found_role.clone(), tenant_ids: if found_role == "admin" { vec![] } else { tenant_ids }, exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS };
    let token = create_session_token(&payload);
    info!(email = %email, role = %found_role, "Portal login successful");
    Json(serde_json::json!({"token":token,"email":email,"role":found_role,"display_name":display_name,"expires_in":SESSION_EXPIRY_SECS})).into_response()
}

pub async fn portal_me(headers: axum::http::HeaderMap) -> impl IntoResponse {
    match extract_session(&headers) {
        Some(s) => Json(serde_json::json!({"email":s.email,"role":s.role,"tenant_ids":s.tenant_ids})).into_response(),
        None => (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Invalid or expired session"}))).into_response(),
    }
}

pub async fn portal_tenants(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_tenants(&state);
    let tenants: Vec<serde_json::Value> = data.tenants.iter()
        .filter(|t| session.role == "admin" || session.tenant_ids.contains(&t.id))
        .map(|t| serde_json::json!({"id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,"provider":t.provider,"model":t.model,"messages_today":t.messages_today,"max_messages_per_day":t.max_messages_per_day,"channels_active":t.channels_active,"members_count":t.members.len(),"created_at":t.created_at,"version":t.version}))
        .collect();
    Json(serde_json::json!({"tenants":tenants})).into_response()
}

pub async fn portal_tenant_detail(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" && !session.tenant_ids.contains(&id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => {
            let members: Vec<serde_json::Value> = t.members.iter().map(|m| serde_json::json!({"email":m.email,"role":m.role,"display_name":m.display_name,"added_at":m.added_at,"last_login":m.last_login,"has_password":m.password_hash.is_some()})).collect();
            Json(serde_json::json!({"id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,"provider":t.provider,"model":t.model,"temperature":t.temperature,"messages_today":t.messages_today,"max_messages_per_day":t.max_messages_per_day,"max_channels":t.max_channels,"max_members":t.max_members,"channels":t.channels,"members":members,"created_at":t.created_at,"version":t.version,"access_token":t.access_token})).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

pub async fn portal_all_members(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let data = load_tenants(&state);
    let mut map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();
    for tenant in &data.tenants {
        for member in &tenant.members {
            let key = member.email.to_lowercase();
            let entry = map.entry(key.clone()).or_insert_with(|| serde_json::json!({"email":member.email,"display_name":member.display_name,"role":member.role,"has_password":member.password_hash.is_some(),"last_login":member.last_login,"tenants":[]}));
            if let Some(arr) = entry["tenants"].as_array_mut() { arr.push(serde_json::json!({"id":tenant.id,"name":tenant.name,"role":member.role})); }
            if member.role == "admin" { entry["role"] = serde_json::json!("admin"); }
        }
    }
    Json(serde_json::json!({"members":map.into_values().collect::<Vec<_>>()})).into_response()
}

pub async fn portal_set_password(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<SetPasswordRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    if req.password.len() < 4 { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Password must be at least 4 characters"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) { Some(m) => m, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response() };
    member.password_hash = Some(hash_password(&req.password));
    if let Some(name) = req.display_name { member.display_name = Some(name); }
    let _ = save_tenants(&state, &data);
    info!(email = %req.email, tenant = %id, "Set portal password");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ---------------------------------------------------------------------------
// Embedded Portal HTML
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
:root{--orange:#FF5C00;--orange-hover:#e65200;--orange-light:#fff7ed;--orange-bg:rgba(255,92,0,.08);--bg:#fff;--bg2:#f9fafb;--text:#111827;--dim:#6b7280;--muted:#9ca3af;--border:#e5e7eb;--green:#22c55e;--green-bg:#f0fdf4;--green-text:#15803d;--red:#ef4444;--blue:#3b82f6;--purple:#8b5cf6;--purple-bg:#faf5ff;--purple-text:#7c3aed}
body{font-family:'Inter',system-ui,-apple-system,sans-serif;margin:0;min-height:100vh;background:var(--bg)}

/* ══ LOGIN ══ */
.login-screen{display:flex;min-height:100vh}
.login-left{flex:1;background:var(--bg2);position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden}
.login-left::before{content:'';position:absolute;inset:0;background-image:linear-gradient(rgba(0,0,0,.03) 1px,transparent 1px),linear-gradient(90deg,rgba(0,0,0,.03) 1px,transparent 1px);background-size:40px 40px;pointer-events:none}
.login-left>*{position:relative;z-index:1}
.brand{display:flex;align-items:center;gap:10px;margin-bottom:40px}
.brand svg{width:36px;height:36px}.brand span{font-size:1.4rem;font-weight:700;letter-spacing:-.5px}
.login-left h2{font-size:2.2rem;font-weight:700;line-height:1.2;letter-spacing:-1px;margin-bottom:16px}
.hl{color:var(--orange)}
.login-left .desc{color:var(--dim);font-size:.95rem;line-height:1.6;margin-bottom:40px}
.terminal-card{background:var(--bg);border:1px solid var(--border);border-radius:12px;overflow:hidden;box-shadow:0 4px 24px rgba(0,0,0,.06);margin-bottom:40px}
.terminal-dots{display:flex;gap:6px;padding:12px 16px;border-bottom:1px solid var(--border)}
.terminal-dots span{width:10px;height:10px;border-radius:50%}
.terminal-dots span:nth-child(1){background:#ff5f57}.terminal-dots span:nth-child(2){background:#febc2e}.terminal-dots span:nth-child(3){background:#28c840}
.terminal-code{padding:16px 20px;font-family:'JetBrains Mono',monospace;font-size:.8rem;line-height:1.8;color:var(--dim)}
.terminal-code .prompt{color:var(--text);font-weight:500}.terminal-code .cmd{color:var(--orange)}
.metrics{display:flex;gap:32px}
.metric .val{font-size:1.5rem;font-weight:700}.metric .val .unit{color:var(--orange);font-weight:600}
.metric .lbl{font-size:.75rem;color:var(--muted);margin-top:2px}
.login-right{width:480px;display:flex;flex-direction:column;justify-content:center;padding:48px;position:relative}
.login-right::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at top right,rgba(255,92,0,.04),transparent 60%);pointer-events:none}
.login-right>*{position:relative;z-index:1}
.brand-sm{display:flex;align-items:center;gap:8px;justify-content:center;margin-bottom:32px}
.brand-sm svg{width:28px;height:28px}.brand-sm span{font-size:1.1rem;font-weight:700}
.login-right h1{font-size:1.75rem;font-weight:700;margin-bottom:8px}
.login-right .subtitle{color:var(--dim);font-size:.9rem;margin-bottom:32px}
.form-group{margin-bottom:16px}
.form-group label{display:block;font-size:.8rem;font-weight:500;color:var(--dim);margin-bottom:6px}
.input-wrap{position:relative}
.input-wrap input{width:100%;padding:12px 16px 12px 44px;border:1px solid var(--border);border-radius:12px;font-size:.9rem;font-family:inherit;color:var(--text);outline:none;transition:border-color .2s,box-shadow .2s}
.input-wrap input:focus{border-color:var(--orange);box-shadow:0 0 0 3px rgba(255,92,0,.1)}
.input-wrap input::placeholder{color:var(--muted)}
.input-wrap .icon{position:absolute;left:14px;top:50%;transform:translateY(-50%);color:var(--muted)}
.btn-login{width:100%;padding:14px;background:var(--orange);color:#fff;border:none;border-radius:12px;font-size:.95rem;font-weight:600;font-family:inherit;cursor:pointer;transition:background .2s;margin-top:8px}
.btn-login:hover{background:var(--orange-hover)}.btn-login:disabled{opacity:.5;cursor:not-allowed}
.error-msg{color:var(--red);font-size:.8rem;margin-top:12px;display:none}
.login-footer{margin-top:24px;text-align:center;font-size:.8rem;color:var(--muted)}
.login-footer a{color:var(--orange);text-decoration:none;font-weight:500}

/* ══ DASHBOARD (light theme, admin-style) ══ */
.dashboard{display:none;height:100vh;overflow:hidden;background:var(--bg2)}
.dash-layout{display:flex;height:100vh}

/* Sidebar */
.sidebar{width:220px;background:var(--bg);border-right:1px solid var(--border);display:flex;flex-direction:column;flex-shrink:0}
.sb-header{padding:16px 20px;display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--border)}
.sb-header svg{width:28px;height:28px}
.sb-header span{font-size:1rem;font-weight:700;letter-spacing:-.3px}
.sb-user{padding:12px 20px;font-size:.8rem;color:var(--dim);border-bottom:1px solid var(--border)}
.sb-nav{flex:1;padding:8px}
.sb-item{display:flex;align-items:center;gap:10px;padding:10px 12px;border-radius:8px;font-size:.85rem;font-weight:500;color:var(--dim);cursor:pointer;transition:all .15s;text-decoration:none}
.sb-item:hover{background:var(--bg2);color:var(--text)}
.sb-item.active{background:var(--orange-light);color:var(--orange)}
.sb-item svg{width:18px;height:18px;flex-shrink:0}
.sb-bottom{padding:8px;border-top:1px solid var(--border)}
.sb-bottom .sb-item{font-size:.8rem;padding:8px 12px}

/* Main content */
.main{flex:1;overflow-y:auto;display:flex;flex-direction:column}
.main-header{padding:20px 32px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--border);background:var(--bg)}
.main-header h1{font-size:1.3rem;font-weight:700;display:flex;align-items:center;gap:10px}
.main-header h1 svg{color:var(--orange)}
.main-content{padding:24px 32px;flex:1}

/* Search & filter bar */
.toolbar{display:flex;gap:12px;margin-bottom:16px;align-items:center}
.search-box{flex:1;position:relative}
.search-box input{width:100%;padding:10px 16px 10px 40px;border:1px solid var(--border);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--text);background:var(--bg);outline:none;transition:border-color .2s}
.search-box input:focus{border-color:var(--orange)}
.search-box input::placeholder{color:var(--muted)}
.search-box svg{position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--muted);width:16px;height:16px}
.filter-btn{padding:10px 16px;border:1px solid var(--border);border-radius:10px;background:var(--bg);font-size:.85rem;font-family:inherit;color:var(--text);cursor:pointer;display:flex;align-items:center;gap:6px}
.filter-btn:hover{border-color:var(--orange)}

/* Stats row */
.stats-row{display:flex;gap:16px;margin-bottom:20px;font-size:.85rem;font-weight:500}
.stats-row .stat-label{color:var(--dim)}
.stats-row .stat-val{font-weight:700}
.stats-row .stat-val.green{color:var(--green-text)}

/* Table */
.data-table{width:100%;border-collapse:collapse;font-size:.85rem;background:var(--bg);border:1px solid var(--border);border-radius:10px;overflow:hidden}
.data-table th{padding:12px 16px;text-align:left;font-weight:600;font-size:.75rem;text-transform:uppercase;color:var(--dim);background:var(--bg2);border-bottom:1px solid var(--border);letter-spacing:.3px}
.data-table td{padding:12px 16px;border-bottom:1px solid var(--border);vertical-align:middle}
.data-table tr:last-child td{border-bottom:none}
.data-table tr:hover td{background:var(--bg2)}
.data-table .name-link{color:var(--orange);font-weight:500;cursor:pointer;text-decoration:none}
.data-table .name-link:hover{text-decoration:underline}
.status-badge{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600}
.status-badge.running{background:var(--green-bg);color:var(--green-text)}
.status-badge.stopped{background:#fef2f2;color:#dc2626}
.plan-badge{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600;background:var(--purple-bg);color:var(--purple-text)}
.version-tag{font-family:'JetBrains Mono',monospace;font-size:.75rem;color:var(--dim);display:flex;align-items:center;gap:4px}
.version-tag::before{content:'🔒';font-size:.65rem}

/* Detail panel */
.detail-panel{background:var(--bg);border:1px solid var(--border);border-radius:12px;padding:24px}
.detail-header{display:flex;align-items:center;justify-content:space-between;margin-bottom:20px}
.detail-header h2{font-size:1.2rem;font-weight:700}
.detail-header .back-btn{color:var(--orange);font-size:.85rem;cursor:pointer;font-weight:500;background:none;border:none;font-family:inherit}
.detail-header .back-btn:hover{text-decoration:underline}
.detail-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:16px;margin-bottom:24px}
.detail-card{background:var(--bg2);border-radius:10px;padding:14px}
.detail-card .d-label{font-size:.7rem;color:var(--dim);text-transform:uppercase;letter-spacing:.3px;margin-bottom:4px;font-weight:600}
.detail-card .d-value{font-size:1.1rem;font-weight:700}
.members-section h3{font-size:.95rem;font-weight:600;margin-bottom:12px}

@media(max-width:900px){
  .login-screen{flex-direction:column}.login-left{display:none}.login-right{width:100%;min-height:100vh}
  .sidebar{display:none}
}
</style>
</head>
<body>

<!-- ══ LOGIN ══ -->
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
    <div class="form-group"><label>Email address</label><div class="input-wrap"><svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg><input type="email" id="loginEmail" placeholder="you@example.com" autocomplete="email"></div></div>
    <div class="form-group"><label>Password</label><div class="input-wrap"><svg class="icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg><input type="password" id="loginPass" placeholder="Enter your password" autocomplete="current-password" onkeydown="if(event.key==='Enter')doLogin()"></div></div>
    <button class="btn-login" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="error-msg" id="loginError"></div>
    <div class="login-footer">System admins can use their <a href="javascript:void(0)">API key</a> as password</div>
  </div>
</div>

<!-- ══ DASHBOARD ══ -->
<div class="dashboard" id="dashView">
  <div class="dash-layout">
    <!-- Sidebar -->
    <div class="sidebar">
      <div class="sb-header">
        <svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg>
        <span>openfang</span>
      </div>
      <div class="sb-user" id="sbUser">Admin</div>
      <div class="sb-nav">
        <a class="sb-item active" onclick="showPage('tenants')">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>
          Tenants
        </a>
        <a class="sb-item" onclick="showPage('members')" id="membersNav" style="display:none">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4-4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>
          Members
        </a>
      </div>
      <div class="sb-bottom">
        <a class="sb-item" onclick="doLogout()">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"/></svg>
          Logout
        </a>
      </div>
    </div>

    <!-- Main -->
    <div class="main">
      <div class="main-header">
        <h1>
          <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>
          <span id="pageTitle">Tenants</span>
        </h1>
      </div>
      <div class="main-content" id="mainContent">
      </div>
    </div>
  </div>
</div>

<script>
let session=null,tenants=[],currentPage='tenants',currentDetail=null;
const LOGO_SVG='<svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8z" fill="#fff"/></svg>';

function api(m,p,b){const o={method:m,headers:{'Content-Type':'application/json'}};if(session)o.headers.Authorization='Bearer '+session.token;if(b)o.body=JSON.stringify(b);return fetch(p,o).then(r=>r.json())}

async function doLogin(){
  const e=document.getElementById('loginEmail').value.trim(),p=document.getElementById('loginPass').value,err=document.getElementById('loginError');
  err.style.display='none';
  if(!e||!p){err.textContent='Please fill in all fields';err.style.display='block';return}
  document.getElementById('loginBtn').disabled=true;
  try{const d=await api('POST','/api/portal/login',{email:e,password:p});if(d.error){err.textContent=d.error;err.style.display='block';return}session=d;localStorage.setItem('portal_session',JSON.stringify(d));showDash()}
  catch(x){err.textContent='Connection error';err.style.display='block'}
  finally{document.getElementById('loginBtn').disabled=false}
}

function doLogout(){session=null;localStorage.removeItem('portal_session');document.getElementById('loginView').style.display='flex';document.getElementById('dashView').style.display='none'}

async function showDash(){
  document.getElementById('loginView').style.display='none';
  document.getElementById('dashView').style.display='block';
  document.getElementById('sbUser').textContent=session.display_name||session.email;
  if(session.role==='admin')document.getElementById('membersNav').style.display='';
  await loadTenants();
  showPage('tenants');
}

async function loadTenants(){const d=await api('GET','/api/portal/tenants');tenants=d.tenants||[]}

function showPage(page){
  currentPage=page;currentDetail=null;
  document.querySelectorAll('.sb-item').forEach(el=>el.classList.remove('active'));
  if(page==='tenants'){
    document.querySelector('.sb-nav .sb-item:first-child').classList.add('active');
    document.getElementById('pageTitle').textContent='Tenants';
    renderTenantsList();
  }else if(page==='members'){
    document.getElementById('membersNav').classList.add('active');
    document.getElementById('pageTitle').textContent='Members';
    renderMembersPage();
  }
}

function renderTenantsList(){
  const running=tenants.filter(t=>t.status==='running').length;
  const c=document.getElementById('mainContent');
  const search=`<div class="toolbar">
    <div class="search-box"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg><input type="text" id="searchInput" placeholder="Search by name or slug..." oninput="filterTenants()"></div>
    <button class="filter-btn" onclick="toggleFilter()"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 3H2l8 9.46V19l4 2v-8.54L22 3z"/></svg><span id="filterLabel">All statuses</span> ▾</button>
  </div>`;
  const stats=`<div class="stats-row"><span class="stat-label">Total: <span class="stat-val">${tenants.length}</span></span><span class="stat-label">Running: <span class="stat-val green">${running}</span></span></div>`;
  const rows=tenants.map(t=>`<tr>
    <td><a class="name-link" onclick="openDetail('${t.id}')">${t.name}</a></td>
    <td style="font-family:'JetBrains Mono',monospace;font-size:.8rem;color:var(--dim)">${t.slug}</td>
    <td><span class="status-badge ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></td>
    <td><span class="plan-badge">${t.plan||'free'}</span></td>
    <td><span class="version-tag">${t.version||'—'}</span></td>
    <td style="color:var(--dim)">${t.created_at?new Date(t.created_at).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'}):'—'}</td>
  </tr>`).join('');
  c.innerHTML=search+stats+`<table class="data-table" id="tenantsTable"><thead><tr><th>Name</th><th>Slug</th><th>Status</th><th>Plan</th><th>Version</th><th>Created</th></tr></thead><tbody>${rows}</tbody></table>`;
}

let statusFilter='all';
function toggleFilter(){statusFilter=statusFilter==='all'?'running':statusFilter==='running'?'stopped':'all';document.getElementById('filterLabel').textContent=statusFilter==='all'?'All statuses':statusFilter==='running'?'Running only':'Stopped only';filterTenants()}

function filterTenants(){
  const q=(document.getElementById('searchInput')?.value||'').toLowerCase();
  const rows=document.querySelectorAll('#tenantsTable tbody tr');
  let i=0;
  tenants.forEach((t,idx)=>{
    const matchSearch=!q||t.name.toLowerCase().includes(q)||t.slug.toLowerCase().includes(q);
    const matchStatus=statusFilter==='all'||(statusFilter==='running'&&t.status==='running')||(statusFilter==='stopped'&&t.status!=='running');
    if(rows[idx])rows[idx].style.display=matchSearch&&matchStatus?'':'none';
  });
}

async function openDetail(id){
  const d=await api('GET','/api/portal/tenants/'+id);
  if(d.error)return;
  currentDetail=d;
  document.getElementById('pageTitle').textContent=d.name;
  const mh=d.members?d.members.map(m=>`<tr>
    <td>${m.display_name||m.email}</td>
    <td style="color:var(--dim)">${m.email}</td>
    <td><span class="status-badge ${m.role==='admin'?'running':'stopped'}" style="${m.role==='admin'?'':'background:var(--orange-bg);color:var(--orange)'}">${m.role}</span></td>
    <td>${m.has_password?'✅':'❌'}</td>
    <td style="color:var(--dim);font-size:.8rem">${m.last_login||'Never'}</td>
  </tr>`).join(''):'';

  document.getElementById('mainContent').innerHTML=`<div class="detail-panel">
    <div class="detail-header"><h2>${d.name}</h2><button class="back-btn" onclick="showPage('tenants')">← Back to Tenants</button></div>
    <p style="color:var(--dim);font-size:.85rem;margin-bottom:20px">${d.slug} · <span class="plan-badge">${d.plan||'free'}</span></p>
    <div class="detail-grid">
      <div class="detail-card"><div class="d-label">Provider</div><div class="d-value" style="font-size:.95rem;text-transform:capitalize">${d.provider}</div></div>
      <div class="detail-card"><div class="d-label">Model</div><div class="d-value" style="font-size:.85rem">${d.model}</div></div>
      <div class="detail-card"><div class="d-label">Messages Today</div><div class="d-value">${d.messages_today}</div></div>
      <div class="detail-card"><div class="d-label">Daily Limit</div><div class="d-value">${d.max_messages_per_day>=4294967295?'∞':d.max_messages_per_day}</div></div>
      <div class="detail-card"><div class="d-label">Status</div><div class="d-value"><span class="status-badge ${d.status}">${d.status==='running'?'Running':'Stopped'}</span></div></div>
      <div class="detail-card"><div class="d-label">Members</div><div class="d-value">${(d.members||[]).length}</div></div>
    </div>
    <div class="members-section">
      <h3>Members</h3>
      <table class="data-table"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Last Login</th></tr></thead><tbody>${mh}</tbody></table>
    </div>
  </div>`;
}

async function renderMembersPage(){
  const d=await api('GET','/api/portal/members');
  const members=d.members||[];
  const rows=members.map(m=>`<tr>
    <td>${m.display_name||m.email}</td>
    <td style="color:var(--dim)">${m.email}</td>
    <td><span class="status-badge ${m.role==='admin'?'running':'stopped'}" style="${m.role==='admin'?'':'background:var(--orange-bg);color:var(--orange)'}">${m.role}</span></td>
    <td>${m.has_password?'✅':'❌'}</td>
    <td>${(m.tenants||[]).map(t=>t.name).join(', ')||'—'}</td>
    <td style="color:var(--dim);font-size:.8rem">${m.last_login||'Never'}</td>
  </tr>`).join('');
  document.getElementById('mainContent').innerHTML=`
    <div class="stats-row"><span class="stat-label">Total Members: <span class="stat-val">${members.length}</span></span></div>
    <table class="data-table"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Tenants</th><th>Last Login</th></tr></thead><tbody>${rows}</tbody></table>`;
}

(function(){const s=localStorage.getItem('portal_session');if(s){try{session=JSON.parse(s);showDash()}catch(e){localStorage.removeItem('portal_session')}}})();
</script>
</body></html>"##;
