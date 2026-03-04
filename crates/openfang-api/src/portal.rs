//! Tenant Self-Service Portal - role-based access for tenant members.
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
use crate::tenants::{load_tenants, save_tenants, seed_defaults, verify_password, hash_password};

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

    let mut data = load_tenants(&state);
    // Seed default plans if empty
    if seed_defaults(&mut data) { let _ = save_tenants(&state, &data); }

    // 1) Check global users first
    if let Some(user) = data.users.iter().find(|u| u.email.to_lowercase() == email) {
        if let Some(hash) = &user.password_hash {
            if verify_password(&req.password, hash) {
                // Find tenant IDs where this user is a member
                let tenant_ids: Vec<String> = data.tenants.iter()
                    .filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == email))
                    .map(|t| t.id.clone())
                    .collect();
                let role = user.role.clone();
                let display_name = user.display_name.clone().unwrap_or_else(|| email.clone());
                // Update last_login
                if let Some(u) = data.users.iter_mut().find(|u| u.email.to_lowercase() == email) {
                    u.last_login = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
                }
                let _ = save_tenants(&state, &data);
                let payload = SessionPayload { email: email.clone(), role: role.clone(), tenant_ids: if role == "admin" { vec![] } else { tenant_ids }, exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS };
                let token = create_session_token(&payload);
                info!(email = %email, role = %role, "Portal user login");
                return Json(serde_json::json!({"token":token,"email":email,"role":role,"display_name":display_name,"expires_in":SESSION_EXPIRY_SECS})).into_response();
            }
        }
    }

    // 2) Fallback: scan tenant members (backward compatible)
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

/// PUT /api/portal/tenants/:id/members/role - Update member role.
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

/// POST /api/portal/tenants/:id/members - Add member to tenant.
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
        added_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        last_login: None,
        password_hash: pw_hash,
    });
    let _ = save_tenants(&state, &data);
    info!(email = %email, tenant = %id, role = %req.role, "Added member via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// DELETE /api/portal/tenants/:id/members - Remove member from tenant.
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
// Portal: Config, Actions, Channels (admin-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PortalUpdateConfigRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub api_key: Option<String>,
}

/// PUT /api/portal/tenants/:id/config - Update tenant config.
pub async fn portal_update_config(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalUpdateConfigRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if let Some(provider) = req.provider { tenant.provider = provider; }
    if let Some(model) = req.model { tenant.model = model; }
    if let Some(temp) = req.temperature { tenant.temperature = temp; }
    if let Some(key) = req.api_key { tenant.api_key = Some(key); }
    let updated = tenant.clone();
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Updated tenant config via portal");
    Json(serde_json::json!({"ok":true,"tenant":updated})).into_response()
}

/// POST /api/portal/tenants/:id/restart - Restart tenant.
pub async fn portal_restart(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.status = crate::tenants::TenantStatus::Running; t.messages_today = 0; let _ = save_tenants(&state, &data); info!(tenant_id = %id, "Restarted tenant via portal"); Json(serde_json::json!({"ok":true,"status":"running"})).into_response() }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

/// POST /api/portal/tenants/:id/stop - Stop tenant.
pub async fn portal_stop(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.status = crate::tenants::TenantStatus::Stopped; let _ = save_tenants(&state, &data); info!(tenant_id = %id, "Stopped tenant via portal"); Json(serde_json::json!({"ok":true,"status":"stopped"})).into_response() }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

/// DELETE /api/portal/tenants/:id - Delete tenant.
pub async fn portal_delete_tenant(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let before = data.tenants.len();
    data.tenants.retain(|t| t.id != id);
    if data.tenants.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(); }
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Deleted tenant via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

#[derive(Debug, Deserialize)]
pub struct PortalAddChannelRequest {
    pub channel_type: String,
    pub name: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PortalRemoveChannelRequest {
    pub name: String,
}

/// POST /api/portal/tenants/:id/channels - Add channel.
pub async fn portal_add_channel(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalAddChannelRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if tenant.channels.len() as u32 >= tenant.max_channels { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Channel limit reached"}))).into_response(); }
    let channel = crate::tenants::TenantChannel {
        name: req.name.unwrap_or_else(|| req.channel_type.clone()),
        channel_type: req.channel_type,
        enabled: true,
        config: req.config,
        added_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    };
    tenant.channels.push(channel);
    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Added channel via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// DELETE /api/portal/tenants/:id/channels - Remove channel.
pub async fn portal_remove_channel(State(state): State<Arc<AppState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalRemoveChannelRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let before = tenant.channels.len();
    tenant.channels.retain(|c| c.name != req.name);
    if tenant.channels.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel not found"}))).into_response(); }
    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Removed channel via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ---------------------------------------------------------------------------
// Portal: Users CRUD (admin-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub display_name: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub plan_id: Option<String>,
}

/// GET /api/portal/users - List all portal users.
pub async fn portal_list_users(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let data = load_tenants(&state);
    let users: Vec<serde_json::Value> = data.users.iter().map(|u| {
        let tenant_count = data.tenants.iter().filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == u.email.to_lowercase())).count();
        serde_json::json!({"email":u.email,"display_name":u.display_name,"role":u.role,"plan_id":u.plan_id,"created_at":u.created_at,"last_login":u.last_login,"max_tenants":u.max_tenants,"tenant_count":tenant_count,"has_password":u.password_hash.is_some()})
    }).collect();
    Json(serde_json::json!({"users":users})).into_response()
}

/// POST /api/portal/users - Create a portal user.
pub async fn portal_create_user(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Json(req): Json<CreateUserRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    if email.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email is required"}))).into_response(); }
    let mut data = load_tenants(&state);
    if seed_defaults(&mut data) { let _ = save_tenants(&state, &data); data = load_tenants(&state); }
    if data.users.iter().any(|u| u.email.to_lowercase() == email) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"User already exists"}))).into_response();
    }
    let role = req.role.unwrap_or_else(|| "user".into());
    let plan_id = req.plan_id.clone().or_else(|| data.plans.iter().find(|p| p.is_default).map(|p| p.id.clone()));
    let max_t = plan_id.as_ref().and_then(|pid| data.plans.iter().find(|p| p.id == *pid)).map(|p| p.max_tenants).unwrap_or(3);
    let user = crate::tenants::PortalUser {
        email: email.clone(),
        display_name: req.display_name,
        password_hash: req.password.filter(|p| p.len() >= 4).map(|p| hash_password(&p)),
        role,
        plan_id,
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        last_login: None,
        max_tenants: max_t,
    };
    data.users.push(user);
    let _ = save_tenants(&state, &data);
    info!(email = %email, "Created portal user");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// PUT /api/portal/users/:email - Update a portal user.
pub async fn portal_update_user(State(state): State<Arc<AppState>>, Path(user_email): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<CreateUserRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let target = user_email.to_lowercase();
    let mut data = load_tenants(&state);
    let user = match data.users.iter_mut().find(|u| u.email.to_lowercase() == target) { Some(u) => u, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"User not found"}))).into_response() };
    if let Some(name) = req.display_name { user.display_name = Some(name); }
    if let Some(role) = req.role { user.role = role; }
    if let Some(plan_id) = req.plan_id.clone() {
        user.plan_id = Some(plan_id.clone());
        if let Some(plan) = data.plans.iter().find(|p| p.id == plan_id) { user.max_tenants = plan.max_tenants; }
    }
    if let Some(pw) = req.password.filter(|p| p.len() >= 4) { user.password_hash = Some(hash_password(&pw)); }
    let _ = save_tenants(&state, &data);
    info!(email = %target, "Updated portal user");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// DELETE /api/portal/users/:email - Delete a portal user.
pub async fn portal_delete_user(State(state): State<Arc<AppState>>, Path(user_email): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let target = user_email.to_lowercase();
    let mut data = load_tenants(&state);
    let before = data.users.len();
    data.users.retain(|u| u.email.to_lowercase() != target);
    if data.users.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"User not found"}))).into_response(); }
    let _ = save_tenants(&state, &data);
    info!(email = %target, "Deleted portal user");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ---------------------------------------------------------------------------
// Portal: Plans CRUD (admin-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreatePlanRequest {
    pub name: String,
    pub max_messages_per_day: u32,
    pub max_channels: u32,
    pub max_members: u32,
    pub max_tenants: u32,
    pub price_label: Option<String>,
    pub is_default: Option<bool>,
}

/// GET /api/portal/plans - List all plans.
pub async fn portal_list_plans(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" && session.role != "user" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Login required"}))).into_response(); }
    let mut data = load_tenants(&state);
    if seed_defaults(&mut data) { let _ = save_tenants(&state, &data); }
    Json(serde_json::json!({"plans":data.plans})).into_response()
}

/// POST /api/portal/plans - Create a plan.
pub async fn portal_create_plan(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Json(req): Json<CreatePlanRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let id = req.name.trim().to_lowercase().replace(' ', "-");
    if data.plans.iter().any(|p| p.id == id) { return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"Plan already exists"}))).into_response(); }
    let plan = crate::tenants::ServicePlan {
        id: id.clone(), name: req.name.trim().into(),
        max_messages_per_day: req.max_messages_per_day, max_channels: req.max_channels,
        max_members: req.max_members, max_tenants: req.max_tenants,
        price_label: req.price_label.unwrap_or_default(),
        is_default: req.is_default.unwrap_or(false),
    };
    data.plans.push(plan);
    let _ = save_tenants(&state, &data);
    info!(plan_id = %id, "Created service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// PUT /api/portal/plans/:id - Update a plan.
pub async fn portal_update_plan(State(state): State<Arc<AppState>>, Path(plan_id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<CreatePlanRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let plan = match data.plans.iter_mut().find(|p| p.id == plan_id) { Some(p) => p, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Plan not found"}))).into_response() };
    plan.name = req.name.trim().into();
    plan.max_messages_per_day = req.max_messages_per_day;
    plan.max_channels = req.max_channels;
    plan.max_members = req.max_members;
    plan.max_tenants = req.max_tenants;
    if let Some(lbl) = req.price_label { plan.price_label = lbl; }
    if let Some(d) = req.is_default { plan.is_default = d; }
    let _ = save_tenants(&state, &data);
    info!(plan_id = %plan_id, "Updated service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// DELETE /api/portal/plans/:id - Delete a plan.
pub async fn portal_delete_plan(State(state): State<Arc<AppState>>, Path(plan_id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_tenants(&state);
    let before = data.plans.len();
    data.plans.retain(|p| p.id != plan_id);
    if data.plans.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Plan not found"}))).into_response(); }
    let _ = save_tenants(&state, &data);
    info!(plan_id = %plan_id, "Deleted service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ---------------------------------------------------------------------------
// Portal: Self-Service Tenant Creation
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct PortalCreateTenantRequest {
    pub name: String,
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// POST /api/portal/my/tenants - User creates a tenant (becomes Owner).
pub async fn portal_create_my_tenant(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, Json(req): Json<PortalCreateTenantRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if req.name.trim().is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Tenant name is required"}))).into_response(); }

    let mut data = load_tenants(&state);
    if seed_defaults(&mut data) { let _ = save_tenants(&state, &data); data = load_tenants(&state); }

    // Find user's plan to get quotas
    let user = data.users.iter().find(|u| u.email.to_lowercase() == session.email.to_lowercase());
    let (max_msg, max_ch, max_mem, max_t) = if let Some(u) = user {
        let plan = u.plan_id.as_ref().and_then(|pid| data.plans.iter().find(|p| p.id == *pid));
        match plan {
            Some(p) => (p.max_messages_per_day, p.max_channels, p.max_members, u.max_tenants),
            None => (100, 3, 5, 3), // defaults if no plan
        }
    } else if session.role == "admin" {
        (u32::MAX, u32::MAX, u32::MAX, u32::MAX)
    } else {
        (100, 3, 5, 2)
    };

    // Check tenant quota
    let current_count = data.tenants.iter().filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == session.email.to_lowercase() && m.role == "owner")).count() as u32;
    if session.role != "admin" && current_count >= max_t {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":format!("Tenant limit reached ({}/{}). Please upgrade your plan.", current_count, max_t)}))).into_response();
    }

    let plan = if let Some(u) = user {
        match u.plan_id.as_deref() {
            Some("pro") => crate::tenants::TenantPlan::Pro,
            Some("enterprise") => crate::tenants::TenantPlan::Enterprise,
            _ => crate::tenants::TenantPlan::Free,
        }
    } else { crate::tenants::TenantPlan::Free };

    let slug = crate::tenants::generate_slug(req.name.trim());
    let tenant = crate::tenants::Tenant {
        id: uuid::Uuid::new_v4().to_string(),
        name: req.name.trim().to_string(),
        slug,
        status: crate::tenants::TenantStatus::Running,
        plan,
        provider: req.provider.unwrap_or_else(|| "groq".into()),
        model: req.model.unwrap_or_else(|| "llama-3.3-70b-versatile".into()),
        temperature: 0.7,
        max_messages_per_day: max_msg,
        max_channels: max_ch,
        max_members: max_mem,
        messages_today: 0,
        channels_active: 0,
        members: vec![crate::tenants::TenantMember {
            email: session.email.clone(),
            role: "owner".into(),
            display_name: user.and_then(|u| u.display_name.clone()),
            added_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            last_login: None,
            password_hash: None,
        }],
        access_token: crate::tenants::generate_access_token(),
        created_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        version: format!("openfang-{}", env!("CARGO_PKG_VERSION")),
        api_key: None,
        channels: vec![],
    };

    let tid = tenant.id.clone();
    data.tenants.push(tenant);
    let _ = save_tenants(&state, &data);
    info!(email = %session.email, tenant_id = %tid, "User created tenant via portal");
    Json(serde_json::json!({"ok":true,"tenant_id":tid})).into_response()
}

// ---------------------------------------------------------------------------
// Portal: System API Proxies (channels, providers, models)
// ---------------------------------------------------------------------------

/// GET /api/portal/system/channels - Proxy to /api/channels.
pub async fn portal_system_channels(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    crate::routes::list_channels(State(state)).await.into_response()
}

/// GET /api/portal/system/providers - Proxy to /api/providers.
pub async fn portal_system_providers(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    crate::routes::list_providers(State(state)).await.into_response()
}

/// GET /api/portal/system/models - Proxy to /api/models with optional ?provider= filter.
pub async fn portal_system_models(State(state): State<Arc<AppState>>, headers: axum::http::HeaderMap, query: axum::extract::Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    crate::routes::list_models(State(state), query).await.into_response()
}

/// Serve portal page for both /portal/ and /portal/{id} (permalink).
pub async fn portal_page_with_id() -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], axum::response::Html(PORTAL_HTML))
}


const PORTAL_HTML: &str = r##"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>OpenFang Portal</title>
<link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet">
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--o:#FF5C00;--oh:#e65200;--obg:rgba(255,92,0,.08);--ol:#fff7ed;--bg:#fff;--bg2:#f9fafb;--bg3:#f3f4f6;--t:#111827;--d:#6b7280;--m:#9ca3af;--b:#e5e7eb;--g:#22c55e;--gb:#f0fdf4;--gt:#15803d;--r:#ef4444;--rb:#fef2f2;--rt:#dc2626;--pb:#faf5ff;--pt:#7c3aed;--bb:#eff6ff;--bt:#2563eb}
body{font-family:'Inter',system-ui,sans-serif;margin:0;min-height:100vh;background:var(--bg2)}
/* Login Screen */
.login-screen{display:flex;min-height:100vh;background:var(--bg)}
.login-left{flex:1;background:var(--bg2);position:relative;display:flex;flex-direction:column;justify-content:center;padding:48px 64px;overflow:hidden}
.login-left::before{content:'';position:absolute;inset:0;background-image:linear-gradient(rgba(0,0,0,.03) 1px,transparent 1px),linear-gradient(90deg,rgba(0,0,0,.03) 1px,transparent 1px);background-size:40px 40px}
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
.login-right::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at top right,rgba(255,92,0,.04),transparent 60%)}
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
/* Dashboard Layout */
.dashboard{display:none;min-height:100vh}
.dl{display:flex;min-height:100vh}
.sb{width:220px;background:var(--bg);border-right:1px solid var(--b);display:flex;flex-direction:column;flex-shrink:0;position:fixed;left:0;top:0;bottom:0;z-index:10}
.sbh{padding:16px 20px;display:flex;align-items:center;gap:10px;border-bottom:1px solid var(--b)}
.sbh svg{width:28px;height:28px}.sbh span{font-size:1rem;font-weight:700}
.sbu{padding:12px 20px;font-size:.8rem;color:var(--d);border-bottom:1px solid var(--b)}
.sbn{flex:1;padding:8px}
.si{display:flex;align-items:center;gap:10px;padding:10px 12px;border-radius:8px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;transition:all .15s;text-decoration:none}
.si:hover{background:var(--bg2);color:var(--t)}.si.active{background:var(--ol);color:var(--o)}
.si svg{width:18px;height:18px;flex-shrink:0}
.sbb{padding:8px;border-top:1px solid var(--b)}
.sbb .si{font-size:.8rem;padding:8px 12px}
.mn{flex:1;margin-left:220px;display:flex;flex-direction:column;min-height:100vh}
.mh{padding:20px 32px;display:flex;align-items:center;justify-content:space-between;border-bottom:1px solid var(--b);background:var(--bg)}
.mh h1{font-size:1.3rem;font-weight:700;display:flex;align-items:center;gap:10px}
.mc{padding:24px 32px;flex:1}
/* List View */
.tb{display:flex;gap:12px;margin-bottom:16px;align-items:center}
.sx{flex:1;position:relative}
.sx input{width:100%;padding:10px 16px 10px 40px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);background:var(--bg);outline:none}
.sx input:focus{border-color:var(--o)}
.sx input::placeholder{color:var(--m)}
.sx svg{position:absolute;left:12px;top:50%;transform:translateY(-50%);color:var(--m);width:16px;height:16px}
.fb{padding:10px 16px;border:1px solid var(--b);border-radius:10px;background:var(--bg);font-size:.85rem;font-family:inherit;color:var(--t);cursor:pointer;display:flex;align-items:center;gap:6px}
.sr{display:flex;gap:16px;margin-bottom:20px;font-size:.85rem;font-weight:500}
.sr .sl{color:var(--d)}.sr .sv{font-weight:700}.sr .sv.gn{color:var(--gt)}
/* Table */
.dt{width:100%;border-collapse:collapse;font-size:.85rem;background:var(--bg);border:1px solid var(--b);border-radius:10px;overflow:hidden}
.dt th{padding:12px 16px;text-align:left;font-weight:600;font-size:.75rem;text-transform:uppercase;color:var(--d);background:var(--bg2);border-bottom:1px solid var(--b)}
.dt td{padding:12px 16px;border-bottom:1px solid var(--b);vertical-align:middle}
.dt tr:last-child td{border-bottom:none}
.dt tr:hover td{background:var(--bg2)}
.dt .nl{color:var(--o);font-weight:500;cursor:pointer;text-decoration:none}
.dt .nl:hover{text-decoration:underline}
/* Badges */
.badge{display:inline-block;padding:3px 10px;border-radius:20px;font-size:.75rem;font-weight:600}
.badge.running{background:var(--gb);color:var(--gt)}.badge.stopped{background:var(--rb);color:var(--rt)}
.badge.plan{background:var(--pb);color:var(--pt)}.badge.pro{background:var(--ol);color:var(--o)}
.vt{font-family:'JetBrains Mono',monospace;font-size:.75rem;color:var(--d)}
/* Buttons */
.btn-o{background:var(--o);color:#fff;border:none;border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:600;font-family:inherit;cursor:pointer;display:inline-flex;align-items:center;gap:6px;transition:background .15s}
.btn-o:hover{background:var(--oh)}
.btn-g{background:var(--bg);color:var(--t);border:1px solid var(--b);border-radius:8px;padding:8px 16px;font-size:.85rem;font-weight:500;font-family:inherit;cursor:pointer;display:inline-flex;align-items:center;gap:6px;transition:all .15s}
.btn-g:hover{background:var(--bg2);border-color:var(--m)}
.btn-r{color:var(--r);background:none;border:none;font-size:.8rem;font-weight:500;cursor:pointer;font-family:inherit;display:inline-flex;align-items:center;gap:4px}
.btn-r:hover{text-decoration:underline}
/* Detail Header */
.bc{font-size:.8rem;color:var(--d);margin-bottom:8px}
.bc a{color:var(--d);text-decoration:none;cursor:pointer}.bc a:hover{color:var(--o)}
.dh{display:flex;align-items:flex-start;justify-content:space-between;margin-bottom:4px}
.dh h2{font-size:1.5rem;font-weight:700;line-height:1.3}
.dh-meta{font-family:'JetBrains Mono',monospace;font-size:.85rem;color:var(--d);margin-bottom:20px;display:flex;align-items:center;gap:8px}
.dh-actions{display:flex;gap:8px;align-items:center;flex-shrink:0}
/* Tabs */
.tabs{display:flex;gap:0;border-bottom:2px solid var(--b);margin-bottom:24px}
.tab{padding:10px 20px;font-size:.85rem;font-weight:500;color:var(--d);cursor:pointer;border-bottom:2px solid transparent;margin-bottom:-2px;transition:all .15s;white-space:nowrap}
.tab:hover{color:var(--t)}.tab.active{color:var(--o);border-bottom-color:var(--o)}
/* Stat Cards */
.cards{display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:24px}
.card{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:20px}
.card .card-label{font-size:.8rem;color:var(--d);display:flex;align-items:center;gap:6px;margin-bottom:8px}
.card .card-val{font-size:1.3rem;font-weight:700}
.card .card-sub{font-size:.75rem;color:var(--d);margin-top:4px}
.card .bar{height:4px;background:var(--bg3);border-radius:4px;margin-top:10px;overflow:hidden}
.card .bar-fill{height:100%;background:var(--g);border-radius:4px}
/* Section Box */
.sbox{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:20px;margin-bottom:20px}
.sbox h3{font-size:1rem;font-weight:700;margin-bottom:4px}
.sbox .sbox-desc{font-size:.85rem;color:var(--d);margin-bottom:16px}
/* Detail Grid */
.detail-grid{display:grid;grid-template-columns:1fr 1fr;gap:12px 40px}
.detail-item{padding:8px 0;border-bottom:1px solid var(--bg3)}
.detail-item .di-label{font-size:.75rem;color:var(--d);text-transform:uppercase;letter-spacing:.3px;font-weight:600}
.detail-item .di-value{font-size:.9rem;font-weight:500;margin-top:2px}
/* Config Form */
.config-section{background:var(--bg);border:1px solid var(--b);border-radius:12px;padding:24px;margin-bottom:20px}
.config-section h3{font-size:.8rem;font-weight:600;text-transform:uppercase;letter-spacing:.5px;color:var(--d);margin-bottom:16px}
.config-row{display:grid;grid-template-columns:1fr 1fr;gap:16px;margin-bottom:16px}
.config-row .fg{margin-bottom:0}
.fg select,.fg input[type=text],.fg input[type=password]{width:100%;padding:10px 14px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);outline:none;background:var(--bg)}
.fg select:focus,.fg input:focus{border-color:var(--o)}
/* Empty State */
.empty{text-align:center;padding:60px 40px;background:var(--bg);border:1px solid var(--b);border-radius:12px}
.empty .empty-icon{font-size:2.5rem;margin-bottom:12px;color:var(--m)}
.empty h4{font-size:1rem;margin-bottom:8px}
.empty p{font-size:.85rem;color:var(--d);margin-bottom:20px}
/* Role Dropdown */
.role-sel{padding:6px 10px;border:1px solid var(--b);border-radius:8px;font-size:.8rem;font-family:inherit;color:var(--t);cursor:pointer;background:var(--bg);min-width:110px;outline:none}
.role-sel:focus{border-color:var(--o)}
/* Modal */
.modal-bg{display:none;position:fixed;inset:0;background:rgba(0,0,0,.4);z-index:100;align-items:center;justify-content:center}
.modal-bg.show{display:flex}
.modal{background:var(--bg);border-radius:12px;padding:24px;width:440px;max-width:90vw;box-shadow:0 20px 60px rgba(0,0,0,.15)}
.modal h3{font-size:1.1rem;font-weight:700;margin-bottom:16px}
.modal .fg input,.modal .fg select{width:100%;padding:10px 14px;border:1px solid var(--b);border-radius:10px;font-size:.85rem;font-family:inherit;color:var(--t);outline:none}
.modal .fg input:focus,.modal .fg select:focus{border-color:var(--o)}
.modal .actions{display:flex;gap:8px;justify-content:flex-end;margin-top:20px}
.modal .btn-cancel{background:var(--bg2);border:1px solid var(--b);border-radius:8px;padding:8px 16px;font-size:.85rem;cursor:pointer;font-family:inherit}
/* Warning */
.warn{display:flex;align-items:center;gap:8px;padding:12px 16px;background:var(--ol);border:1px solid #fed7aa;border-radius:10px;font-size:.85rem;color:#9a3412;margin-bottom:16px}
@media(max-width:900px){.login-screen{flex-direction:column}.login-left{display:none}.login-right{width:100%;min-height:100vh}.sb{display:none}.mn{margin-left:0}}
</style>
</head>
<body>
<!-- LOGIN -->
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
    <div class="fg"><label>Email address</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="4" width="20" height="16" rx="2"/><path d="M22 7l-10 6L2 7"/></svg><input type="email" id="loginEmail" placeholder="email" autocomplete="email@domain.com"></div></div>
    <div class="fg"><label>Password</label><div class="iw"><svg class="ic" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="11" width="18" height="11" rx="2"/><path d="M7 11V7a5 5 0 0110 0v4"/></svg><input type="password" id="loginPass" placeholder="Enter your password" autocomplete="current-password" onkeydown="if(event.key==='Enter')doLogin()"></div></div>
    <button class="bl" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="em" id="loginError"></div>
    <div class="lf">System admins can use their <a href="javascript:void(0)">API key</a> as password</div>
  </div>
</div>

<!-- DASHBOARD -->
<div class="dashboard" id="dashView">
  <div class="dl">
    <div class="sb">
      <div class="sbh"><svg viewBox="0 0 40 40" fill="none"><rect width="40" height="40" rx="8" fill="#111827"/><path d="M20 8C13.4 8 8 13.4 8 20s5.4 12 12 12 12-5.4 12-12S26.6 8 20 8zm0 2c2.8 0 5.3 1.2 7.1 3H12.9c1.8-1.8 4.3-3 7.1-3zm-8.5 5h17c.9 1.5 1.5 3.2 1.5 5h-20c0-1.8.6-3.5 1.5-5zm-1.3 7h19.6c-.3 1.8-1.1 3.5-2.2 4.8l-3.4-2.4c-.3-.2-.7-.1-.9.2s-.1.7.2.9l3.1 2.2c-1.6 1.4-3.6 2.2-5.6 2.3v-4c0-.4-.3-.7-.7-.7s-.6.3-.6.7v4c-2.1-.1-4-.9-5.6-2.3l3.1-2.2c.3-.2.4-.6.2-.9s-.6-.4-.9-.2l-3.4 2.4c-1.1-1.3-1.9-3-2.2-4.8z" fill="#fff"/></svg><span>openfang</span></div>
      <div class="sbu" id="sbUser">Admin</div>
      <div class="sbn">
        <a class="si active" onclick="showPage('tenants')"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg>Tenants</a>
        <a class="si" onclick="showPage('members')" id="membersNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 00-4-4H5a4 4 0 00-4-4v2"/><circle cx="9" cy="7" r="4"/><path d="M23 21v-2a4 4 0 00-3-3.87M16 3.13a4 4 0 010 7.75"/></svg>Members</a>
        <a class="si" onclick="showPage('users')" id="usersNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>Users</a>
        <a class="si" onclick="showPage('plans')" id="plansNav" style="display:none"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M7 10l5 5 5-5M12 15V3"/></svg>Plans</a>
      </div>
      <div class="sbb"><a class="si" onclick="doLogout()"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"/></svg>Logout</a></div>
    </div>
    <div class="mn">
      <div class="mh"><h1 id="pageTitle">Tenants</h1><div id="headerActions"></div></div>
      <div class="mc" id="mainContent"></div>
    </div>
  </div>
</div>

<div class="modal-bg" id="addMemberModal">
  <div class="modal">
    <h3>Add Member</h3>
    <div class="fg"><label>Email</label><input type="email" id="amEmail" placeholder="user@example.com"></div>
    <div class="fg"><label>Display Name</label><input type="text" id="amName" placeholder="John Doe"></div>
    <div class="fg"><label>Role</label><select id="amRole"><option value="viewer">Viewer</option><option value="contributor">Contributor</option><option value="manager">Manager</option><option value="owner">Owner</option></select></div>
    <div class="fg"><label>Password (optional)</label><input type="password" id="amPass" placeholder="Min 4 chars for portal login"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('addMemberModal')">Cancel</button><button class="btn-o" onclick="doAddMember()">Add Member</button></div>
  </div>
</div>

<!-- Add Channel Modal -->
<div class="modal-bg" id="addChannelModal">
  <div class="modal">
    <h3>Add Channel</h3>
    <div class="fg"><label>Channel Type</label><select id="acType"><option value="">Select type...</option><option value="telegram">Telegram</option><option value="discord">Discord</option><option value="slack">Slack</option><option value="whatsapp">WhatsApp</option><option value="signal">Signal</option><option value="matrix">Matrix</option><option value="email">Email</option><option value="zalo">Zalo</option><option value="web">Web Widget</option></select></div>
    <div class="fg"><label>Display Name (optional)</label><input type="text" id="acName" placeholder="e.g. My Telegram Bot"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('addChannelModal')">Cancel</button><button class="btn-o" onclick="addChannel()">Add Channel</button></div>
  </div>
</div>

<!-- Create User Modal -->
<div class="modal-bg" id="createUserModal">
  <div class="modal">
    <h3>Create User</h3>
    <div class="fg"><label>Email</label><input type="email" id="cuEmail" placeholder="user@example.com"></div>
    <div class="fg"><label>Display Name</label><input type="text" id="cuName" placeholder="John Doe"></div>
    <div class="fg"><label>Password</label><input type="password" id="cuPass" placeholder="Min 4 characters"></div>
    <div class="fg"><label>Role</label><select id="cuRole"><option value="user">User</option><option value="admin">Admin</option></select></div>
    <div class="fg"><label>Plan</label><select id="cuPlan"></select></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createUserModal')">Cancel</button><button class="btn-o" onclick="doCreateUser()">Create User</button></div>
  </div>
</div>

<!-- Create Plan Modal -->
<div class="modal-bg" id="createPlanModal">
  <div class="modal">
    <h3>Create Plan</h3>
    <div class="fg"><label>Plan Name</label><input type="text" id="cpName" placeholder="e.g. Starter"></div>
    <div class="config-row"><div class="fg"><label>Messages/Day</label><input type="number" id="cpMsg" value="500"></div><div class="fg"><label>Max Channels</label><input type="number" id="cpCh" value="5"></div></div>
    <div class="config-row"><div class="fg"><label>Max Members</label><input type="number" id="cpMem" value="10"></div><div class="fg"><label>Max Tenants</label><input type="number" id="cpTen" value="5"></div></div>
    <div class="fg"><label>Price Label</label><input type="text" id="cpPrice" placeholder="e.g. $19/mo"></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createPlanModal')">Cancel</button><button class="btn-o" onclick="doCreatePlan()">Create Plan</button></div>
  </div>
</div>

<!-- Create Tenant Modal -->
<div class="modal-bg" id="createTenantModal">
  <div class="modal">
    <h3>Create Tenant</h3>
    <div class="fg"><label>Tenant Name</label><input type="text" id="ctName" placeholder="e.g. My AI Bot"></div>
    <div class="config-row"><div class="fg"><label>Provider</label><select id="ctProvider"><option value="groq">Groq</option><option value="openai">OpenAI</option><option value="anthropic">Anthropic</option><option value="openrouter">OpenRouter</option><option value="deepseek">DeepSeek</option><option value="ollama">Ollama</option><option value="gemini">Gemini</option></select></div><div class="fg"><label>Model</label><input type="text" id="ctModel" value="llama-3.3-70b-versatile"></div></div>
    <div class="actions"><button class="btn-cancel" onclick="closeModal('createTenantModal')">Cancel</button><button class="btn-o" onclick="doCreateMyTenant()">Create Tenant</button></div>
  </div>
</div>

<script>
let S=null,T=[],D=null,CTab='overview';
const ROLES=['Owner','Manager','Contributor','Viewer'];
const INF=4294967295;
function api(m,p,b){const o={method:m,headers:{'Content-Type':'application/json'}};if(S)o.headers.Authorization='Bearer '+S.token;if(b)o.body=JSON.stringify(b);return fetch(p,o).then(r=>r.json())}
function fmt(v){return v>=INF?'Unlimited':v}
function fmtDate(d){if(!d)return '-';return new Date(d).toLocaleDateString('en-US',{month:'short',day:'numeric',year:'numeric'})}

// Auth
async function doLogin(){const e=document.getElementById('loginEmail').value.trim(),p=document.getElementById('loginPass').value,err=document.getElementById('loginError');err.style.display='none';if(!e||!p){err.textContent='Please fill in all fields';err.style.display='block';return}document.getElementById('loginBtn').disabled=true;try{const d=await api('POST','/api/portal/login',{email:e,password:p});if(d.error){err.textContent=d.error;err.style.display='block';return}S=d;localStorage.setItem('ps',JSON.stringify(d));showDash()}catch(x){err.textContent='Connection error';err.style.display='block'}finally{document.getElementById('loginBtn').disabled=false}}
function doLogout(){S=null;localStorage.removeItem('ps');document.getElementById('loginView').style.display='flex';document.getElementById('dashView').style.display='none'}
async function showDash(){document.getElementById('loginView').style.display='none';document.getElementById('dashView').style.display='block';document.getElementById('sbUser').textContent=S.display_name||S.email;if(S.role==='admin'){document.getElementById('membersNav').style.display='';document.getElementById('usersNav').style.display='';document.getElementById('plansNav').style.display=''}await loadT();showPage('tenants')}
async function loadT(){const d=await api('GET','/api/portal/tenants');T=d.tenants||[]}

// Navigation
function showPage(p){D=null;document.querySelectorAll('.sbn .si').forEach(el=>el.classList.remove('active'));document.getElementById('headerActions').innerHTML='';history.pushState({page:p},'','/portal/');
if(p==='tenants'){document.querySelector('.sbn .si:first-child').classList.add('active');document.getElementById('pageTitle').innerHTML='<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="width:24px;height:24px"><rect x="2" y="3" width="20" height="18" rx="2"/><path d="M2 9h20M9 21V9"/></svg> Tenants';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateTenantModal()">+ Create Tenant</button>';renderList()}
else if(p==='members'){document.getElementById('membersNav').classList.add('active');document.getElementById('pageTitle').textContent='Members';renderMembers()}
else if(p==='users'){document.getElementById('usersNav').classList.add('active');document.getElementById('pageTitle').textContent='Users';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openCreateUserModal()">+ Create User</button>';renderUsers()}
else if(p==='plans'){document.getElementById('plansNav').classList.add('active');document.getElementById('pageTitle').textContent='Service Plans';document.getElementById('headerActions').innerHTML='<button class="btn-o" onclick="openModal(\"createPlanModal\")">+ Create Plan</button>';renderPlans()}}

// Tenant List
function renderList(){
  const run=T.filter(t=>t.status==='running').length;
  const rows=T.map(t=>`<tr><td><a class="nl" onclick="openDetail('${t.id}')">${t.name}</a></td><td class="vt">${t.slug}</td><td><span class="badge ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></td><td><span class="badge pro">${(t.plan||'free').charAt(0).toUpperCase()+(t.plan||'free').slice(1)}</span></td><td class="vt">${t.version||'-'}</td><td style="color:var(--d)">${fmtDate(t.created_at)}</td><td>${S.role==='admin'?'<button class="btn-r" onclick="event.stopPropagation()">Delete</button>':''}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="tb"><div class="sx"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg><input type="text" id="si2" placeholder="Search by name or slug..." oninput="filterT()"></div><button class="fb" onclick="toggleF()"><span id="fl">All statuses</span> &#9662;</button></div><div class="sr"><span class="sl">Total: <span class="sv">${T.length}</span></span><span class="sl">Running: <span class="sv gn">${run}</span></span></div><table class="dt" id="tt"><thead><tr><th>Name</th><th>Slug</th><th>Status</th><th>Plan</th><th>Version</th><th>Created</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
let sf='all';
function toggleF(){sf=sf==='all'?'running':sf==='running'?'stopped':'all';document.getElementById('fl').textContent=sf==='all'?'All statuses':sf==='running'?'Running only':'Stopped only';filterT()}
function filterT(){const q=(document.getElementById('si2')?.value||'').toLowerCase();document.querySelectorAll('#tt tbody tr').forEach((r,i)=>{const t=T[i];if(!t)return;const ms=!q||t.name.toLowerCase().includes(q)||t.slug.toLowerCase().includes(q);const mf=sf==='all'||(sf==='running'&&t.status==='running')||(sf==='stopped'&&t.status!=='running');r.style.display=ms&&mf?'':'none'})}

// Tenant Detail
async function openDetail(id){
  const d=await api('GET','/api/portal/tenants/'+id);if(d.error)return;D=d;CTab='overview';history.pushState({page:'detail',id:id},d.name,'/portal/'+id);renderDetailPage();
}
function renderDetailPage(){
  if(!D)return;const t=D;const isAdmin=S.role==='admin';
  document.getElementById('pageTitle').innerHTML=`<span>${t.name}</span>`;
  const ha=isAdmin?`<a class="btn-o" href="/access/?t=${t.access_token||''}" target="_blank">Open Dashboard</a><button class="btn-g" onclick="doRestart()">Restart</button><button class="btn-g" onclick="doStop()">Stop</button><button class="btn-r" style="padding:8px 16px;border:1px solid var(--b);border-radius:8px" onclick="doDeleteTenant()">Delete</button>`:'';
  document.getElementById('headerActions').innerHTML=ha;
  renderDetailBody();
}
async function renderDetailBody(){
  if(!D)return;const t=D;const isAdmin=S.role==='admin';
  const bc=`<div class="bc"><a onclick="showPage('tenants')">Tenants</a> &gt; ${t.name}</div>`;
  const header=`<div class="dh"><h2>${t.name}</h2></div><div class="dh-meta"><span>${t.slug}</span> &middot; <span class="badge ${t.status}">${t.status==='running'?'Running':'Stopped'}</span></div>`;
  const TABS=['Overview','Config','Channels','Usage','Members'];
  const tabsHtml=`<div class="tabs">${TABS.map(tb=>`<div class="tab${CTab===tb.toLowerCase()?' active':''}" onclick="CTab='${tb.toLowerCase()}';renderDetailBody()">${tb}</div>`).join('')}</div>`;
  let body='';
  if(CTab==='overview') body=renderOverview(t);
  else if(CTab==='config') body=await renderConfig(t);
  else if(CTab==='channels') body=await renderChannels(t);
  else if(CTab==='usage') body=renderUsage(t);
  else if(CTab==='members') body=renderMembersTab(t,isAdmin);
  document.getElementById('mainContent').innerHTML=bc+header+tabsHtml+body;
  if(CTab==='config') loadModelsForProvider();
}

// Tab: Overview
function renderOverview(t){
  const chCount=(t.channels||[]).length;
  let html=`<div class="cards">
    <div class="card"><div class="card-label">Status</div><div class="card-val"><span class="badge ${t.status}" style="font-size:.85rem;padding:4px 14px">${t.status==='running'?'Running':'Stopped'}</span></div></div>
    <div class="card"><div class="card-label">Provider</div><div class="card-val" style="font-size:1rem;text-transform:capitalize">${t.provider||'-'}</div><div class="card-sub">${t.model||''}</div></div>
    <div class="card"><div class="card-label">Channels</div><div class="card-val">${chCount} / ${fmt(t.max_channels)}</div></div>
    <div class="card"><div class="card-label">Messages</div><div class="card-val">${t.messages_today} today</div><div class="card-sub">Limit: ${fmt(t.max_messages_per_day)}/day</div></div>
  </div>`;
  // Magic Access Link
  html+=`<div class="sbox"><h3>Magic Access Link</h3><div class="sbox-desc">One-time link for instant dashboard access. Share it directly or send via email.</div>`;
  if(t.access_token){html+=`<div style="display:flex;align-items:center;gap:8px"><code style="flex:1;padding:10px 14px;background:var(--bg2);border:1px solid var(--b);border-radius:8px;font-size:.8rem;word-break:break-all">${location.origin}/access/?t=${t.access_token}</code><button class="btn-g" onclick="navigator.clipboard.writeText('${location.origin}/access/?t=${t.access_token}')">Copy</button></div>`}
  else{html+=`<button class="btn-g">Generate Access Link</button>`}
  html+=`</div>`;
  // Tenant Details
  html+=`<div class="sbox"><h3>Tenant Details</h3><div class="detail-grid">
    <div class="detail-item"><div class="di-label">ID</div><div class="di-value vt" style="font-size:.85rem">${t.id}</div></div>
    <div class="detail-item"><div class="di-label">Subdomain</div><div class="di-value"><span class="vt" style="color:var(--o)">${t.slug}.${location.hostname}</span></div></div>
    <div class="detail-item"><div class="di-label">Plan</div><div class="di-value"><span class="badge pro">${(t.plan||'free').charAt(0).toUpperCase()+(t.plan||'free').slice(1)}</span> - ${fmt(t.max_messages_per_day)} msg/day, ${fmt(t.max_channels)} ch, ${fmt(t.max_members)} members</div></div>
    <div class="detail-item"><div class="di-label">Temperature</div><div class="di-value">${t.temperature}</div></div>
    <div class="detail-item"><div class="di-label">Version</div><div class="di-value vt">${t.version||'-'}</div></div>
    <div class="detail-item"><div class="di-label">Created</div><div class="di-value">${fmtDate(t.created_at)}</div></div>
  </div></div>`;
  return html;
}

// Tab: Config
async function renderConfig(t){
  const isAdmin=S.role==='admin';
  const dis=isAdmin?'':'disabled';
  // Load providers from system API
  let provOpts=`<option value="${t.provider}">${t.provider}</option>`;
  try{const pd=await api('GET','/api/portal/system/providers');const provs=pd.providers||[];
    provOpts=provs.map(p=>`<option value="${p.id}"${t.provider===p.id?' selected':''}>${p.display_name}${p.auth_status==='configured'?' [OK]':p.auth_status==='not_required'?' [Local]':' [No Key]'}</option>`).join('');
  }catch(e){}
  let html=`<div class="config-section"><h3>AI Provider</h3>
    <div class="config-row"><div class="fg"><label>Provider</label><select id="cfgProvider" ${dis} onchange="loadModelsForProvider()">${provOpts}</select></div><div class="fg"><label>Model</label><select id="cfgModel" ${dis}><option value="${t.model||''}">${t.model||'Select model'}</option></select></div></div>
    <div class="config-row"><div class="fg"><label>Temperature</label><input type="text" id="cfgTemp" value="${t.temperature}" ${dis} style="width:120px"></div><div class="fg"><label>API Key</label><input type="password" id="cfgApiKey" value="${t.api_key||''}" placeholder="Provider API key" ${dis}></div></div>`;
  if(isAdmin){html+=`<div style="margin-top:12px"><button class="btn-o" onclick="saveConfig()">Save Config</button></div>`}
  else{html+=`<div class="warn">Configuration changes are managed by the system administrator.</div>`}
  html+=`</div>
  <div class="config-section"><h3>Quotas</h3>
    <div class="config-row"><div class="fg"><label>Messages per Day</label><input type="text" value="${fmt(t.max_messages_per_day)}" disabled></div><div class="fg"><label>Max Channels</label><input type="text" value="${fmt(t.max_channels)}" disabled></div></div>
    <div class="fg"><label>Max Members</label><input type="text" value="${fmt(t.max_members)}" disabled style="width:200px"></div>
  </div>`;
  return html;
}
async function loadModelsForProvider(){
  const prov=document.getElementById('cfgProvider').value;
  const sel=document.getElementById('cfgModel');
  sel.innerHTML='<option>Loading...</option>';
  try{const d=await api('GET','/api/portal/system/models?provider='+prov);const ms=d.models||[];
    sel.innerHTML=ms.map(m=>`<option value="${m.id}"${D&&D.model===m.id?' selected':''}>${m.display_name} (${m.id})</option>`).join('');
    if(ms.length===0)sel.innerHTML='<option value="">No models found</option>';
  }catch(e){sel.innerHTML='<option value="">Error loading models</option>'}
}

// Tab: Channels
async function renderChannels(t){
  const isAdmin=S.role==='admin';
  const tenantCh=t.channels||[];
  // Load system channels
  let sysCh=[];
  try{const d=await api('GET','/api/portal/system/channels');sysCh=d.channels||[]}catch(e){}
  // Map tenant channels with system info
  const addBtn=isAdmin?`<button class="btn-o" onclick="openAddSystemChannel()">+ Add Channel</button>`:'';
  const cnt=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><div><h3 style="font-size:1rem;font-weight:700">Channels</h3><p style="font-size:.8rem;color:var(--d)">${tenantCh.length} / ${fmt(t.max_channels)} channels connected</p></div>${addBtn}</div>`;
  // Active channels
  let html=cnt;
  if(tenantCh.length>0){
    const rows=tenantCh.map(c=>{const sys=sysCh.find(s=>s.name===c.channel_type)||{};
      const del=isAdmin?`<button class="btn-r" onclick="removeChannel('${c.name}')">Remove</button>`:'';
      const status=sys.configured?'<span class="badge running">Configured</span>':'<span class="badge plan">Pending</span>';
      return `<tr><td style="text-transform:capitalize;font-weight:500">${sys.display_name||c.channel_type||'-'}</td><td>${c.name||c.channel_type||'-'}</td><td>${status}</td><td>${del}</td></tr>`}).join('');
    html+=`<table class="dt"><thead><tr><th>Type</th><th>Name</th><th>Status</th>${isAdmin?'<th>Actions</th>':''}</tr></thead><tbody>${rows}</tbody></table>`;
  } else {
    html+=`<div class="empty"><div class="empty-icon">(( ))</div><h4>No channels connected</h4><p>Connect a messaging platform to start receiving messages.</p></div>`;
  }
  // Available system channels
  if(isAdmin && sysCh.length>0){
    html+=`<div style="margin-top:24px"><h3 style="font-size:1rem;font-weight:700;margin-bottom:12px">Available Channels (${sysCh.length})</h3>`;
    const cats=[...new Set(sysCh.map(c=>c.category))];
    cats.forEach(cat=>{
      const chs=sysCh.filter(c=>c.category===cat);
      html+=`<div style="margin-bottom:12px"><div style="font-size:.8rem;font-weight:600;color:var(--d);text-transform:uppercase;margin-bottom:6px">${cat} (${chs.length})</div>`;
      html+=`<div style="display:flex;flex-wrap:wrap;gap:8px">`;
      chs.forEach(c=>{const connected=tenantCh.some(tc=>tc.channel_type===c.name);
        const badge=connected?'running':c.configured?'plan':'stopped';
        const label=connected?'Connected':c.configured?'Available':'Not Configured';
        html+=`<div style="padding:6px 12px;background:var(--bg2);border:1px solid var(--b);border-radius:8px;font-size:.8rem;display:flex;align-items:center;gap:6px"><span style="font-weight:500">${c.display_name}</span><span class="badge ${badge}" style="font-size:.65rem;padding:2px 6px">${label}</span>`;if(!connected&&isAdmin)html+=`<button class="btn-g" style="padding:2px 8px;font-size:.7rem" onclick="addSystemChannel('${c.name}','${c.display_name}')">Add</button>`;html+=`</div>`;
      });html+=`</div></div>`;
    });html+=`</div>`;
  }
  return html;
}
function openAddSystemChannel(){CTab='channels';renderDetailBody()}
async function addSystemChannel(type,name){const body={channel_type:type,name:name};const d=await api('POST','/api/portal/tenants/'+D.id+'/channels',body);if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}

// Tab: Usage
function renderUsage(t){
  const pct=t.max_messages_per_day>=INF?0:Math.min(100,Math.round(t.messages_today/t.max_messages_per_day*100));
  return `<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:20px"><h3 style="font-size:1rem;font-weight:700">Usage</h3><span class="badge plan" style="padding:6px 12px;font-size:.8rem">Current Period</span></div>
  <div class="cards" style="grid-template-columns:repeat(3,1fr)">
    <div class="card"><div class="card-label">Messages Today</div><div class="card-val" style="font-size:2rem">${t.messages_today}</div></div>
    <div class="card"><div class="card-label">Daily Limit</div><div class="card-val" style="font-size:2rem">${fmt(t.max_messages_per_day)}</div></div>
    <div class="card"><div class="card-label">Members</div><div class="card-val" style="font-size:2rem">${(t.members||[]).length} / ${fmt(t.max_members)}</div></div>
  </div>
  <div class="sbox"><h3>Daily Message Usage</h3>
    <div class="bar" style="height:20px;margin-top:12px;background:var(--bg3);border-radius:8px;overflow:hidden"><div style="height:100%;background:${pct>80?'var(--r)':pct>50?'#f59e0b':'var(--g)'};border-radius:8px;width:${pct}%;transition:width .3s"></div></div>
    <p style="font-size:.8rem;color:var(--d);margin-top:8px">${t.max_messages_per_day>=INF?'Unlimited':pct+'% used ('+t.messages_today+' / '+t.max_messages_per_day+')'}</p>
  </div>`;
}

// Tab: Members
function renderMembersTab(t,isAdmin){
  const addBtn=isAdmin?`<button class="btn-o" onclick="openModal('addMemberModal')">+ Add Member</button>`:'';
  const header=`<div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:16px"><h3 style="font-size:1rem;font-weight:700">Members</h3>${addBtn}</div>`;
  const rows=(t.members||[]).map(m=>{
    const roleHtml=isAdmin?`<select class="role-sel" onchange="changeRole('${m.email}',this.value)">${ROLES.map(r=>`<option value="${r.toLowerCase()}"${m.role===r.toLowerCase()?' selected':''}>${r}</option>`).join('')}</select>`:`<span class="badge plan">${m.role.charAt(0).toUpperCase()+m.role.slice(1)}</span>`;
    const actions=isAdmin?`<button class="btn-r" onclick="removeMember('${m.email}')">Remove</button>`:'';
    return `<tr><td style="font-weight:500">${m.email}</td><td>${roleHtml}</td><td style="color:var(--d)">${fmtDate(m.added_at)}</td><td>${actions}</td></tr>`;
  }).join('');
  return header+`<table class="dt"><thead><tr><th>Email</th><th>Role</th><th>Joined</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// Tenant Actions
async function doRestart(){if(!D)return;const d=await api('POST',`/api/portal/tenants/${D.id}/restart`);if(d.ok){await loadT();D=await api('GET','/api/portal/tenants/'+D.id);renderDetailPage()}else{alert(d.error||'Failed')}}
async function doStop(){if(!D||!confirm('Stop this tenant?'))return;const d=await api('POST',`/api/portal/tenants/${D.id}/stop`);if(d.ok){await loadT();D=await api('GET','/api/portal/tenants/'+D.id);renderDetailPage()}else{alert(d.error||'Failed')}}
async function doDeleteTenant(){if(!D||!confirm('Delete tenant "'+D.name+'"? This cannot be undone.'))return;const d=await api('DELETE','/api/portal/tenants/'+D.id);if(d.ok){await loadT();showPage('tenants')}else{alert(d.error||'Failed')}}

// Config Actions
async function saveConfig(){if(!D)return;const body={provider:document.getElementById('cfgProvider').value,model:document.getElementById('cfgModel').value,temperature:parseFloat(document.getElementById('cfgTemp').value)||0.7};const key=document.getElementById('cfgApiKey').value.trim();if(key)body.api_key=key;const d=await api('PUT',`/api/portal/tenants/${D.id}/config`,body);if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody();alert('Config saved!')}else{alert(d.error||'Failed')}}

// Channel Actions
async function addChannel(){const ct=document.getElementById('acType').value,nm=document.getElementById('acName').value.trim();if(!ct){alert('Channel type is required');return}const body={channel_type:ct};if(nm)body.name=nm;const d=await api('POST',`/api/portal/tenants/${D.id}/channels`,body);if(d.ok){closeModal('addChannelModal');document.getElementById('acName').value='';D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
async function removeChannel(name){if(!confirm('Remove channel "'+name+'"?'))return;const d=await api('DELETE',`/api/portal/tenants/${D.id}/channels`,{name});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}

// Member Actions
async function changeRole(email,role){const d=await api('PUT',`/api/portal/tenants/${D.id}/members/role`,{email,role});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
async function removeMember(email){if(!confirm('Remove '+email+'?'))return;const d=await api('DELETE',`/api/portal/tenants/${D.id}/members`,{email});if(d.ok){D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}
function openModal(id){document.getElementById(id).classList.add('show')}
function closeModal(id){document.getElementById(id).classList.remove('show')}
async function doAddMember(){const e=document.getElementById('amEmail').value.trim(),n=document.getElementById('amName').value.trim(),r=document.getElementById('amRole').value,p=document.getElementById('amPass').value;if(!e){alert('Email is required');return}const body={email:e,role:r};if(n)body.display_name=n;if(p)body.password=p;const d=await api('POST',`/api/portal/tenants/${D.id}/members`,body);if(d.ok){closeModal('addMemberModal');document.getElementById('amEmail').value='';document.getElementById('amName').value='';document.getElementById('amPass').value='';D=await api('GET','/api/portal/tenants/'+D.id);renderDetailBody()}else{alert(d.error||'Failed')}}

// All Members Page
async function renderMembers(){
  const d=await api('GET','/api/portal/members');const ms=d.members||[];
  const rows=ms.map(m=>`<tr><td style="font-weight:500">${m.display_name||m.email}</td><td style="color:var(--d)">${m.email}</td><td><span class="badge plan">${m.role}</span></td><td>${m.has_password?'Yes':'No'}</td><td>${(m.tenants||[]).map(t=>t.name).join(', ')||'-'}</td><td style="color:var(--d);font-size:.8rem">${m.last_login||'Never'}</td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Members: <span class="sv">${ms.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Tenants</th><th>Last Login</th></tr></thead><tbody>${rows}</tbody></table>`;
}

// Users Page
async function renderUsers(){
  const d=await api('GET','/api/portal/users');const us=d.users||[];
  const rows=us.map(u=>`<tr><td style="font-weight:500">${u.display_name||u.email}</td><td style="color:var(--d)">${u.email}</td><td><span class="badge ${u.role==='admin'?'running':'plan'}">${u.role}</span></td><td><span class="badge plan">${u.plan_id||'none'}</span></td><td>${u.tenant_count||0} / ${fmt(u.max_tenants)}</td><td>${u.has_password?'Yes':'No'}</td><td style="color:var(--d);font-size:.8rem">${u.last_login?fmtDate(u.last_login):'Never'}</td><td><button class="btn-r" onclick="deleteUser('${u.email}')">Delete</button></td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Users: <span class="sv">${us.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Plan</th><th>Tenants</th><th>Password</th><th>Last Login</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
async function openCreateUserModal(){const d=await api('GET','/api/portal/plans');const plans=d.plans||[];const sel=document.getElementById('cuPlan');sel.innerHTML=plans.map(p=>`<option value="${p.id}"${p.is_default?' selected':''}>${p.name} (${p.price_label||'Free'})</option>`).join('');openModal('createUserModal')}
async function doCreateUser(){const email=document.getElementById('cuEmail').value.trim(),name=document.getElementById('cuName').value.trim(),pass=document.getElementById('cuPass').value,role=document.getElementById('cuRole').value,plan=document.getElementById('cuPlan').value;if(!email){alert('Email is required');return}const body={email,role,plan_id:plan};if(name)body.display_name=name;if(pass)body.password=pass;const d=await api('POST','/api/portal/users',body);if(d.ok){closeModal('createUserModal');document.getElementById('cuEmail').value='';document.getElementById('cuName').value='';document.getElementById('cuPass').value='';renderUsers()}else{alert(d.error||'Failed')}}
async function deleteUser(email){if(!confirm('Delete user "'+email+'"?'))return;const d=await api('DELETE','/api/portal/users/'+encodeURIComponent(email));if(d.ok)renderUsers();else alert(d.error||'Failed')}

// Plans Page
async function renderPlans(){
  const d=await api('GET','/api/portal/plans');const ps=d.plans||[];
  const rows=ps.map(p=>`<tr><td style="font-weight:500">${p.name}${p.is_default?' <span class="badge running" style="font-size:.7rem">Default</span>':''}</td><td>${fmt(p.max_messages_per_day)}</td><td>${fmt(p.max_channels)}</td><td>${fmt(p.max_members)}</td><td>${fmt(p.max_tenants)}</td><td>${p.price_label||'-'}</td><td><button class="btn-r" onclick="deletePlan('${p.id}')">Delete</button></td></tr>`).join('');
  document.getElementById('mainContent').innerHTML=`<div class="sr"><span class="sl">Total Plans: <span class="sv">${ps.length}</span></span></div><table class="dt"><thead><tr><th>Name</th><th>Msg/Day</th><th>Channels</th><th>Members</th><th>Tenants</th><th>Price</th><th>Actions</th></tr></thead><tbody>${rows}</tbody></table>`;
}
async function doCreatePlan(){const name=document.getElementById('cpName').value.trim();if(!name){alert('Plan name is required');return}const body={name,max_messages_per_day:parseInt(document.getElementById('cpMsg').value)||500,max_channels:parseInt(document.getElementById('cpCh').value)||5,max_members:parseInt(document.getElementById('cpMem').value)||10,max_tenants:parseInt(document.getElementById('cpTen').value)||5,price_label:document.getElementById('cpPrice').value.trim()};const d=await api('POST','/api/portal/plans',body);if(d.ok){closeModal('createPlanModal');document.getElementById('cpName').value='';renderPlans()}else{alert(d.error||'Failed')}}
async function deletePlan(id){if(!confirm('Delete plan "'+id+'"?'))return;const d=await api('DELETE','/api/portal/plans/'+encodeURIComponent(id));if(d.ok)renderPlans();else alert(d.error||'Failed')}

// Create Tenant (self-service)
function openCreateTenantModal(){openModal('createTenantModal')}
async function doCreateMyTenant(){const name=document.getElementById('ctName').value.trim();if(!name){alert('Tenant name is required');return}const body={name,provider:document.getElementById('ctProvider').value,model:document.getElementById('ctModel').value};const d=await api('POST','/api/portal/my/tenants',body);if(d.ok){closeModal('createTenantModal');document.getElementById('ctName').value='';await loadT();showPage('tenants')}else{alert(d.error||'Failed')}}

// Init + Permalink
window.addEventListener('popstate',function(e){if(e.state&&e.state.page==='detail'&&e.state.id){openDetail(e.state.id)}else{showPage('tenants')}});
(function(){const s=localStorage.getItem('ps');if(s){try{S=JSON.parse(s);
  // Check if URL has tenant ID (permalink)
  const m=location.pathname.match(/\/portal\/([a-f0-9-]+)/i);
  if(m){showDash().then(()=>openDetail(m[1]))}else{showDash()}
}catch(e){localStorage.removeItem('ps')}}})();
</script>
</body></html>"##;
