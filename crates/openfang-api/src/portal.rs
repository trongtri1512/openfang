//! Tenant Self-Service Portal Ã¢â‚¬â€ role-based access for tenant members.
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

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: String,
    pub display_name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RemoveMemberRequest {
    pub email: String,
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

/// PUT /api/portal/tenants/:id/members/role Ã¢â‚¬â€ Update member role.
pub async fn portal_update_role(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<UpdateRoleRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let valid_roles = ["owner", "manager", "contributor", "viewer", "admin", "member"];
    let new_role = req.role.to_lowercase();
    if !valid_roles.contains(&new_role.as_str()) { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Invalid role"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) { Some(m) => m, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response() };
    member.role = new_role.clone();
    let _ = save_tenants(&state, &data);
    info!(email = %req.email, tenant = %id, role = %new_role, "Updated member role via portal");
    Json(serde_json::json!({"ok":true,"role":new_role})).into_response()
}

/// POST /api/portal/tenants/:id/members Ã¢â‚¬â€ Add member to tenant.
pub async fn portal_add_member(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<AddMemberRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    if email.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email is required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if tenant.members.iter().any(|m| m.email.to_lowercase() == email) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"Member already exists"}))).into_response();
    }
    let pw_hash = req.password.as_ref().filter(|p| p.len() >= 4).map(|p| hash_password(p));
    tenant.members.push(crate::tenants::TenantMember {
        email: email.clone(),
        role: req.role.to_lowercase(),
        display_name: req.display_name.clone(),
        added_at: Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        last_login: None,
        password_hash: pw_hash,
    });
    let _ = save_tenants(&state, &data);
    info!(email = %email, tenant = %id, role = %req.role, "Added member via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// DELETE /api/portal/tenants/:id/members Ã¢â‚¬â€ Remove member from tenant.
pub async fn portal_remove_member(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<RemoveMemberRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let before = tenant.members.len();
    tenant.members.retain(|m| m.email.to_lowercase() != email);
    if tenant.members.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response(); }
    let _ = save_tenants(&state, &data);
    info!(email = %email, tenant = %id, "Removed member via portal");
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
:root{--o:#FF5C00;--oh:#e65200;--obg:rgba(255,92,0,.08);--ol:#fff7ed;--bg:#fff;--bg2:#f9fafb;--t:#111827;--d:#6b7280;--m:#9ca3af;--b:#e5e7eb;--g:#22c55e;--gb:#f0fdf4;--gt:#15803d;--r:#ef4444;--pb:#faf5ff;--pt:#7c3aed}
body{font-family:'Inter',system-ui,sans-serif;margin:0;min-height:100vh;background:var(--bg)}
.login-screen{display:flex;min-height:100vh}
.login-left{flex:1;background:var(--bg2);position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden}
.login-left::before{content:'';position:absolute;inset:0;background-image:linear-gradient(rgba(0,0,0,.03) 1px,transparent 1px),linear-gradient(90deg,rgba(0,0,0,.03) 1px,transparent 1px);background-size:40px 40px;pointer-events:none}
.login-left>*{position:relative;z-index:1}
.brand{display:flex;align-items:center;gap:10px;margin-bottom:40px}
.brand svg{width:36px;height:36px}.brand span{font-size:1.4rem;font-weight:700;letter-spacing:-.5px}
.login-left h2{font-size:2.2rem;font-weight:700;line-height:1.2;letter-spacing:-1px;margin-bottom:16px}
.hl{color:var(--o)}
.login-left .desc{color:var(--d);font-size:.95rem;line-height:1.6;margin-bottom:40px}
.tc{background:var(--bg);border:1px solid var(--b);border-radius:12px;overflow:hidden;box-shadow:0 4px 24px rgba(0,0,0,.06);margin-bottom:40px}
.td{display:flex;gap:6px;padding:12px 16px;border-bottom:1px solid var(--b)}
.td span{width:10px;height:10px;border-radius:50%}
.td span:nth-child(1){background:#ff5f57}.td span:nth-child(2){background:#febc2e}.td span:nth-child(3){background:#28c840}
.tcd{padding:16px 20px;font-family:'JetBrains Mono',monospace;font-size:.8rem;line-height:1.8;color:var(--d)}
.tcd .p{color:var(--t);font-weight:500}.tcd .c{color:var(--o)}
.mets{display:flex;gap:32px}
.met .v{font-size:1.5rem;font-weight:700}.met .v .u{color:var(--o);font-weight:600}
.met .l{font-size:.75rem;color:var(--m);margin-top:2px}
.login-right{width:480px;display:flex;flex-direction:column;justify-content:center;padding:48px;position:relative}
.login-right::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at top right,rgba(255,92,0,.04),transparent 60%);pointer-events:none}
.login-right>*{position:relative;z-index:1}
.bsm{display:flex;align-items:center;gap:8px;justify-content:center;margin-bottom:32px}
.bsm svg{width:28px;height:28px}.bsm span{font-size:1.1rem;font-weight:700}
.login-right h1{font-size:1.75rem;font-weight:700;margin-bottom:8px}
.login-right .sub{color:var(--d);font-size:.9rem;margin-bottom:32px}
.fg{margin-bottom:16px}
.fg label{display:block;font-size:.8rem;font-weight:500;color:var(--d);margin-bottom:6px}
.iw{position:relative}
.iw input{width:100%;padding:12px 16px 12px 44px;border:1px solid var(--b);border-radius:12px;font-size:.9rem;font-family:inherit;color:var(--t);outline:none;transition:border-color .2s,box-shadow .2s}
.iw input:focus{border-color:var(--o);box-shadow:0 0 0 3px rgba(255,92,0,.1)}
.iw input::placeholder{color:var(--m)}
.iw .ic{position:absolute;left:14px;top:50%;transform:translateY(-50%);color:var(--m)}
.bl{width:100%;padding:14px;background:var(--o);color:#fff;border:none;border-radius:12px;font-size:.95rem;font-weight:600;font-family:inherit;cursor:pointer;transition:background .2s;margin-top:8px}
.bl:hover{background:var(--oh)}.bl:disabled{opacity:.5;cursor:not-allowed}
.em{color:var(--r);font-size:.8rem;margin-top:12px;display:none}
.lf{margin-top:24px;text-align:center;font-size:.8rem;color:var(--m)}.lf a{color:var(--o);text-decoration:none;font-weight:500}

.dashboard{display:none;height:100vh;overflow:hidden;background:var(--bg2)}
.dl{display:flex;height:100vh}
.sb{width:220px;background:var(--bg);border-right:1px solid var(--b);display:flex;flex-direction:column;flex-shrink:0}
.sbh{padding:16px 20px;display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--b)}
.sbh svg{width:28px;height:28px}.sbh span{font-size:1rem;font-weight:700}
.sbu{padding:12px 20px;font-size:.8rem;color:var(--d);border-bottom:1px solid var(--b)}
.sbn{flex:1;padding:8px}
.si{display:flex;align-items:center;gap:10px;padding:10px 12px;border-radius:8px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;transition:all .15s;text-decoration:none}
.si:hover{background:var(--bg2);color:var(--t)}
.si.active{background:var(--ol);color:var(--o)}
.si svg{width:18px;height:18px;flex-shrink:0}
.sbb{padding:8px;border-top:1px solid var(--b)}
.sbb .si{font-size:.8rem;padding:8px 12px}
.mn{flex:1;overflow-y:auto;display:flex;flex-direction:column}
.mh{padding:20px 32px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--b);background:var(--bg)}
.mh h1{font-size:1.3rem;font-weight:700;display:flex;align-items:center;gap:10px}
.mc{padding:24px 32px;flex:1}
.tb{display:flex;gap:12px;margin-bottom:16px;align-items:center}
.sx{flex:1;position:relative}
.sx input{width:100%;padding:10px 16px 10px 40px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);background:var(--bg);outline:none}
.sx input:focus{border-color:var(--o)}
.sx input::placeholder{color:var(--m)}
.sx svg{position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--m);width:16px;height:16px}
.fb{padding:10px 16px;border:1px solid var(--b);border-radius:10px;background:var(--bg);font-size:.85rem;font-family:inherit;color:var(--t);cursor:pointer;display:flex;align-items:center;gap:6px}
.sr{display:flex;gap:16px;margin-bottom:20px;font-size:.85rem;font-weight:500}
.sr .sl{color:var(--d)}.sr .sv{font-weight:700}.sr .sv.gn{color:var(--gt)}
.dt{width:100%;border-collapse:collapse;font-size:.85rem;background:var(--bg);border:1px solid var(--b);border-radius:10px;overflow:hidden}
.dt th{padding:12px 16px;text-align:left;font-weight:600;font-size:.75rem;text-transform:uppercase;color:var(--d);background:var(--bg2);border-bottom:1px solid var(--b)}
.dt td{padding:12px 16px;border-bottom:1px solid var(--b);vertical-align:middle}
.dt tr:last-child td{border-bottom:none}
.dt tr:hover td{background:var(--bg2)}
.dt .nl{color:var(--o);font-weight:500;cursor:pointer;text-decoration:none}
.dt .nl:hover{text-decoration:underline}
.sb-r{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600}
.sb-r.running{background:var(--gb);color:var(--gt)}.sb-r.stopped{background:#fef2f2;color:#dc2626}
.pb{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600;background:var(--pb);color:var(--pt)}
.vt{font-family:'JetBrains Mono',monospace;font-size:.75rem;color:var(--d)}
.btn-o{background:var(--o);color:#fff;border:none;border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:600;font-family:inherit;cursor:pointer;display:inline-flex;align-items:center;gap:6px}
.btn-o:hover{background:var(--oh)}
.btn-r{color:var(--r);background:none;border:none;font-size:.8rem;font-weight:500;cursor:pointer;font-family:inherit}
.btn-r:hover{text-decoration:underline}

/* Breadcrumb + Detail */
.bc{font-size:.8rem;color:var(--d);margin-bottom:8px}
.bc a{color:var(--d);text-decoration:none;cursor:pointer}.bc a:hover{color:var(--o)}
.dh{display:flex;align-items:center;justify-content:space-between;margin-bottom:20px}
.dh h2{font-size:1.4rem;font-weight:700;display:flex;align-items:center;gap:12px}
.dh .slug{font-family:'JetBrains Mono',monospace;font-size:.85rem;color:var(--d);font-weight:400}

/* Tabs */
.tabs{display:flex;gap:0;border-bottom:2px solid var(--b);margin-bottom:24px}
.tab{padding:10px 20px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;border-bottom:2px solid transparent;margin-bottom:-2px;transition:all .15s}
.tab:hover{color:var(--t)}.tab.active{color:var(--o);border-bottom-color:var(--o)}

/* Detail grid */
.dg{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:16px;margin-bottom:24px}
.dc{background:var(--bg2);border-radius:10px;padding:14px}
.dc .dl2{font-size:.7rem;color:var(--d);text-transform:uppercase;letter-spacing:.3px;margin-bottom:4px;font-weight:600}
.dc .dv{font-size:1.1rem;font-weight:700}

/* Role select */
.role-sel{padding:4px 8px;border:1px solid var(--b);border-radius:6px;font-size:.8rem;font-family:inherit;color:var(--t);cursor:pointer;background:var(--bg);min-width:110px}
.role-sel:focus{border-color:var(--o);outline:none}

/* Modal */
.modal-bg{display:none;position:fixed;inset:0;background:rgba(0,0,0,.4);z-index:100;align-items:center;justify-content:center}
.modal-bg.show{display:flex}
.modal{background:var(--bg);border-radius:12px;padding:24px;width:420px;max-width:90vw;box-shadow:0 20px 60px rgba(0,0,0,.15)}
.modal h3{font-size:1.1rem;margin-bottom:16px}
.modal .fg input,.modal .fg select{width:100%;padding:10px 14px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);outline:none}
.modal .fg input:focus,.modal .fg select:focus{border-color:var(--o)}
.modal .actions{display:flex;gap:8px;justify-content:flex-end;margin-top:16px}
.modal .btn-cancel{background:var(--bg2);border:1px solid var(--b);border-radius:8px;padding:8px 16px;font-size:.85rem;cursor:pointer;font-family:inherit}

@media(max-width:900px){.login-screen{flex-direction:column}.login-left{display:none}.login-right{width:100%;min-height:100vh}.sb{display:none}}
</style>
</head>
<body>
<div class="login-screen" id="loginView">
  <div class="login-left">
    <div class="brand"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h2>Deploy &amp; manage<br>AI agents with <span class="hl">the official<br>OpenFang runtime</span></h2>
    <p class="desc">Self-service portal for team members. Manage your tenants, view analytics, and collaborate securely.</p>
    <div class="tc"><div class="td"><span></span><span></span><span></span></div><div class="tcd"><div><span class="p">$</span> <span class="c">openfang serve</span></div><div>booted in &lt;200ms</div><div>hands 7 active</div><div>gateway ready :3000</div></div></div>
    <div class="mets"><div class="met"><div class="v">32 <span class="u">MB</span></div><div class="l">Binary</div></div><div class="met"><div class="v">180<span class="u">ms</span></div><div class="l">Cold Start</div></div><div class="met"><div class="v">26<span class="u">+</span></div><div class="l">Providers</div></div></div>
  </div>
  <div class="login-right">
    <div class="bsm"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
    <h1>Welcome back</h1>
    <p class="sub">Sign in to manage your tenants and agents.</p>
    <div class="fg"><label>Email address</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg><input type="email" id="loginEmail" placeholder="admin@trongtri.com" autocomplete="email"></div></div>
    <div class="fg"><label>Password</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg><input type="password" id="loginPass" placeholder="Enter your password or API key" autocomplete="current-password" onkeydown="if(event.key==='Enter')doLogin()"></div></div>
    <button class="bl" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="em" id="loginError"></div>
    <div class="lf">System admins can use their <a href="javascript:void(0)">API key</a> as password</div>
  </div>
</div>

<div class="dashboard" id="dashView">
  <div class="dl">
    <div class="sb">
      <div class="sbh"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
      <div class="sbu" id="sbUser">Admin</div>
      <div class="sbn">
        <a class="si active" onclick="showPage('tenants')"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>Tenants</a>
        <a class="si" onclick="showPage('members')" id="membersNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4-4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>Members</a>
      </div>
      <div class="sbb"><a class="si" onclick="doLogout()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"/></svg>Logout</a></div>
    </div>
    <div class="mn">
      <div class="mh"><h1 id="pageTitle">Tenants</h1><div id="headerActions"></div></div>
      <div class="mc" id="mainContent"></div>
    </div>
  </div>
</div>

<!-- Add Member Modal -->
<div class="modal-bg" id="addMemberModal">
  <div class="modal">
    <h3>âž• Add Member</h3>
    <div class="fg"><label>Email</label><input type="email" id="amEmail" placeholder="user@example.com"></div>
    <div class="fg"><label>Display Name</label><input type="text" id="amName" placeholder="John Doe"></div>
    <div class="fg"><label>Role</label><select id="amRole"><option value="viewer">Viewer</option><option value="contributor">Contributor</option><option value="manager">Manager</option><option value="owner">Owner</option></select></div>
    <div class="fg"><label>Password (optional)</label><input type="password" id="amPass" placeholder="Min 4 chars for portal login"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeAddModal()">Cancel</button><button class="btn-o" onclick="doAddMember()">Add Member</button></div>
  </div>
</div>

<script>
let S=null,T=[],D=null,CTab='overview';
const ROLES=['Owner','Manager','Contributor','Viewer'];
function api(m,p,b){const o={method:m,headers:{'Content-Type':'application/json'}};if(S)o.headers.Authorization='Bearer '+S.token;if(b)o.body=JSON.stringify(b);return fetch(p,o).then(r=>r.json())}
async function doLogin(){const e=document.getElementById('loginEmail').value.trim(),p=document.getElementById('loginPass').value,err=document.getElementById('loginError');err.style.display='none';if(!e||!p){err.textContent='Please fill in all fields';err.style.display='block';return}document.getElementById('loginBtn').disabled=true;try{const d=await api('POST','/api/portal/login',{email:e,password:p});if(d.error){err.textContent=d.error;err.style.display='block';return}S=d;localStorage.setItem('ps',JSON.stringify(d));showDash()}catch(x){err.textContent='Connection error';err.style.display='block'}finally{document.getElementById('loginBtn').disabled=false}}
function doLogout(){S=null;localStorage.removeItem('ps');document.getElementById('loginView').style.display='flex';document.getElementById('dashView').style.display='none'}
async function showDash(){document.getElementById('loginView').style.display='none';document.getElementById('dashView').style.display='block';document.getElementById('sbUser').textContent=S.display_name||S.email;if(S.role==='admin')document.getElementById('membersNav').style.display='';await loadT();showPage('tenants')}
async function loadT(){const d=await api('GET','/api/portal/tenants');T=d.tenants||[]}

function showPage(p){D=null;document.querySelectorAll('.si').forEach(el=>el.classList.remove('active'));document.getElementById('headerActions').innerHTML='';
if(p==='tenants'){document.querySelector('.sbn .si:first-child').classList.add('active');document.getElementById('pageTitle').textContent='Tenants';renderList()}
else if(p==='members'){document.getElementById('membersNav').classList.add('active');document.getElementById('pageTitle').textContent='Members';renderMembers()}}

function renderList(){
  const run=T.filter(t=>t.status==='running').length;
  const rows=T.map(t=>`<tr><td><a class="nl" onclick="openDetail('${t.id}')">${t.name}</a></td><td class="vt">${t.slug}</td><td><span class="sb-r ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></td><td><span class="pb">${t.plan||'free'}</span></td><td class="vt">${t.version||'â€”'}</td><td style="color:var(--d)">${t.created_at?new Date(t.created_at).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'}):'â€”'}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="tb"><div class="sx"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg><input type="text" id="si2" placeholder="Search by name or slug..." oninput="filterT()"></div><button class="fb" onclick="toggleF()"><span id="fl">All statuses</span> â–¾</button></div><div class="sr"><span class="sl">Total: <span class="sv">${T.length}</span></span><span class="sl">Running: <span class="sv gn">${run}</span></span></div><table class="dt" id="tt"><thead><tr><th>Name</th><th>Slug</th><th>Status</th><th>Plan</th><th>Version</th><th>Created</th></tr></thead><tbody>${rows}</tbody></table>`;
}

let sf='all';
function toggleF(){sf=sf==='all'?'running':sf==='running'?'stopped':'all';document.getElementById('fl').textContent=sf==='all'?'All statuses':sf==='running'?'Running only':'Stopped only';filterT()}
function filterT(){const q=(document.getElementById('si2')?.value||'').toLowerCase();document.querySelectorAll('#tt tbody tr').forEach((r,i)=>{const t=T[i];if(!t)return;const ms=!q||t.name.toLowerCase().includes(q)||t.slug.toLowerCase().includes(q);const mf=sf==='all'||(sf==='running'&&t.status==='running')||(sf==='stopped'&&t.status!=='running');r.style.display=ms&&mf?'':'none'})}

async function openDetail(id){
  const d=await api('GET','/api/portal/tenants/'+id);if(d.error)return;D=d;CTab='overview';
  document.getElementById('pageTitle').innerHTML=`<span style="display:flex;align-items:center;gap:12px">${d.name} <span class="sb-r ${d.status}" style="font-size:.75rem">${d.status==='running'?'Running':'Stopped'}</span></span>`;
  document.getElementById('headerActions').innerHTML='';
  renderDetail();
}

function renderDetail(){
  if(!D)return;const t=D;const isAdmin=S.role==='admin';
  const bc=`<div class="bc"><a onclick="showPage('tenants')">Tenants</a> â€º ${t.name}</div>`;
  const slug=`<p style="color:var(--d);font-size:.85rem;margin-bottom:20px"><span class="vt">${t.slug}</span> Â· <span class="pb">${t.plan||'free'}</span> Â· <span class="vt">${t.version||''}</span></p>`;
  const tabs=['Overview','Channels','Usage','Members'];
  const tabsHtml=`<div class="tabs">${tabs.map(tb=>`<div class="tab${CTab===tb.toLowerCase()?' active':''}" onclick="CTab='${tb.toLowerCase()}';renderDetail()">${tb}</div>`).join('')}</div>`;
  let body='';
  if(CTab==='overview'){
    body=`<div class="dg"><div class="dc"><div class="dl2">Provider</div><div class="dv" style="font-size:.95rem;text-transform:capitalize">${t.provider}</div></div><div class="dc"><div class="dl2">Model</div><div class="dv" style="font-size:.85rem">${t.model}</div></div><div class="dc"><div class="dl2">Temperature</div><div class="dv">${t.temperature}</div></div><div class="dc"><div class="dl2">Messages Today</div><div class="dv">${t.messages_today}</div></div><div class="dc"><div class="dl2">Daily Limit</div><div class="dv">${t.max_messages_per_day>=4294967295?'âˆž':t.max_messages_per_day}</div></div><div class="dc"><div class="dl2">Max Channels</div><div class="dv">${t.max_channels>=4294967295?'âˆž':t.max_channels}</div></div><div class="dc"><div class="dl2">Max Members</div><div class="dv">${t.max_members>=4294967295?'âˆž':t.max_members}</div></div><div class="dc"><div class="dl2">Members</div><div class="dv">${(t.members||[]).length}</div></div></div>`;
    if(t.access_token){body+=`<div class="dc" style="margin-bottom:16px"><div class="dl2">Magic Access Link</div><div class="dv" style="font-size:.8rem;word-break:break-all"><a href="/access/?t=${t.access_token}" target="_blank" style="color:var(--o)">${location.origin}/access/?t=${t.access_token}</a></div></div>`}
  }else if(CTab==='channels'){
    const ch=t.channels||[];
    if(ch.length===0){body='<p style="color:var(--d);padding:40px;text-align:center">No channels configured.</p>'}
    else{body=`<table class="dt"><thead><tr><th>Type</th><th>Name</th><th>Status</th></tr></thead><tbody>${ch.map(c=>`<tr><td>${c.channel_type||'â€”'}</td><td>${c.name||c.channel_type||'â€”'}</td><td><span class="sb-r running">Active</span></td></tr>`).join('')}</tbody></table>`}
  }else if(CTab==='usage'){
    body=`<div class="dg"><div class="dc"><div class="dl2">Messages Today</div><div class="dv" style="font-size:2rem">${t.messages_today}</div></div><div class="dc"><div class="dl2">Daily Limit</div><div class="dv" style="font-size:2rem">${t.max_messages_per_day>=4294967295?'âˆž':t.max_messages_per_day}</div></div></div><div class="dc"><div class="dl2">Usage</div><div style="background:var(--b);border-radius:8px;height:20px;margin-top:8px;overflow:hidden"><div style="height:100%;background:var(--o);border-radius:8px;width:${t.max_messages_per_day>=4294967295?0:Math.min(100,t.messages_today/t.max_messages_per_day*100)}%"></div></div><p style="font-size:.8rem;color:var(--d);margin-top:6px">${t.max_messages_per_day>=4294967295?'Unlimited':Math.round(t.messages_today/t.max_messages_per_day*100)+'% used'}</p></div>`;
  }else if(CTab==='members'){
    const addBtn=isAdmin?`<button class="btn-o" onclick="openAddModal()">âž• Add Member</button>`:'';
    const mh=t.members?t.members.map(m=>{
      const roleHtml=isAdmin?`<select class="role-sel" onchange="changeRole('${m.email}',this.value)">${ROLES.map(r=>`<option value="${r.toLowerCase()}"${m.role===r.toLowerCase()?' selected':''}>${r}</option>`).join('')}</select>`:`<span class="sb-r" style="background:var(--obg);color:var(--o)">${m.role}</span>`;
      const actions=isAdmin?`<button class="btn-r" onclick="removeMember('${m.email}')">ðŸ—‘ Remove</button>`:'';
      return `<tr><td>${m.email}</td><td>${roleHtml}</td><td style="color:var(--d);font-size:.8rem">${m.added_at?new Date(m.added_at).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'}):'â€”'}</td><td>${actions}</td></tr>`;
    }).join(''):'';
    body=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><h3 style="font-size:1rem;font-weight:600">Members</h3>${addBtn}</div><table class="dt"><thead><tr><th>Email</th><th>Role</th><th>Joined</th><th>Actions</th></tr></thead><tbody>${mh}</tbody></table>`;
  }
  document.getElementById('mainContent').innerHTML=bc+slug+tabsHtml+body;
}

async function changeRole(email,role){const d=await api('PUT',`/api/portal/tenants/${D.id}/members/role`,{email,role});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetail()}else{alert(d.error||'Failed')}}
async function removeMember(email){if(!confirm('Remove '+email+'?'))return;const d=await api('DELETE',`/api/portal/tenants/${D.id}/members`,{email});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetail()}else{alert(d.error||'Failed')}}
function openAddModal(){document.getElementById('addMemberModal').classList.add('show')}
function closeAddModal(){document.getElementById('addMemberModal').classList.remove('show');document.getElementById('amEmail').value='';document.getElementById('amName').value='';document.getElementById('amPass').value=''}
async function doAddMember(){const e=document.getElementById('amEmail').value.trim(),n=document.getElementById('amName').value.trim(),r=document.getElementById('amRole').value,p=document.getElementById('amPass').value;if(!e){alert('Email is required');return}const body={email:e,role:r};if(n)body.display_name=n;if(p)body.password=p;const d=await api('POST',`/api/portal/tenants/${D.id}/members`,body);if(d.ok){closeAddModal();D=await api('GET','/api/portal/tenants/'+D.id);renderDetail()}else{alert(d.error||'Failed')}}

async function renderMembers(){
  const d=await api('GET','/api/portal/members');const ms=d.members||[];
  const rows=ms.map(m=>`<tr><td>${m.display_name||m.email}</td><td style="color:var(--d)">${m.email}</td><td><span class="sb-r" style="background:var(--obg);color:var(--o)">${m.role}</span></td><td>${m.has_password?'âœ…':'âŒ'}</td><td>${(m.tenants||[]).map(t=>t.name).join(', ')||'â€”'}</td><td style="color:var(--d);font-size:.8rem">${m.last_login||'Never'}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Members: <span class="sv">${ms.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Tenants</th><th>Last Login</th></tr></thead><tbody>${rows}</tbody></table>`;
}

(function(){const s=localStorage.getItem('ps');if(s){try{S=JSON.parse(s);showDash()}catch(e){localStorage.removeItem('ps')}}})();
</script>
</body></html>"##;
