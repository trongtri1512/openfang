//! All portal API handlers — independent from OpenFang core.
//!
//! Uses `PortalState` for database access and `reqwest` for OpenFang system API proxies.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use std::sync::Arc;
use tracing::{info, warn};

use crate::db::{PortalState, load_data, save_data};
use crate::models::*;

const SESSION_SECRET: &str = "openfang_portal_v1";
const SESSION_EXPIRY_SECS: i64 = 86400;

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

fn is_admin_or_owner(session: &SessionPayload, data: &PortalData, tenant_id: &str) -> bool {
    if session.role == "admin" { return true; }
    let email = session.email.to_lowercase();
    data.tenants.iter().find(|t| t.id == tenant_id)
        .map(|t| t.members.iter().any(|m| m.email.to_lowercase() == email && (m.role == "owner" || m.role == "manager")))
        .unwrap_or(false)
}

// ─── HTML Page ───────────────────────────────────────────────────────────────
pub async fn portal_page() -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], axum::response::Html(crate::html::PORTAL_HTML))
}
pub async fn portal_page_with_id() -> impl IntoResponse {
    ([(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")], axum::response::Html(crate::html::PORTAL_HTML))
}

// ─── Auth ────────────────────────────────────────────────────────────────────
pub async fn portal_login(State(state): State<Arc<PortalState>>, Json(req): Json<PortalLoginRequest>) -> impl IntoResponse {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() || req.password.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email and password are required"}))).into_response();
    }
    // Super Admin: password == PORTAL_ADMIN_KEY or OPENFANG_API_KEY
    let admin_key = std::env::var("PORTAL_ADMIN_KEY")
        .or_else(|_| std::env::var("OPENFANG_API_KEY"))
        .unwrap_or_default();
    if !admin_key.is_empty() && req.password == admin_key {
        let payload = SessionPayload { email: email.clone(), role: "admin".into(), tenant_ids: vec![], exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS };
        let token = create_session_token(&payload);
        info!(email = %email, "Super admin portal login via API key");
        return Json(serde_json::json!({"token":token,"email":email,"role":"admin","display_name":"System Admin","expires_in":SESSION_EXPIRY_SECS})).into_response();
    }
    let mut data = load_data(&state);
    if seed_defaults(&mut data) { let _ = save_data(&state, &data); }

    // 1) Check global users
    if let Some(user) = data.users.iter().find(|u| u.email.to_lowercase() == email) {
        if let Some(hash) = &user.password_hash {
            if verify_password(&req.password, hash) {
                let tenant_ids: Vec<String> = data.tenants.iter()
                    .filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == email))
                    .map(|t| t.id.clone()).collect();
                let role = user.role.clone();
                let display_name = user.display_name.clone().unwrap_or_else(|| email.clone());
                if let Some(u) = data.users.iter_mut().find(|u| u.email.to_lowercase() == email) {
                    u.last_login = Some(now_iso());
                }
                let _ = save_data(&state, &data);
                let payload = SessionPayload { email: email.clone(), role: role.clone(), tenant_ids: if role == "admin" { vec![] } else { tenant_ids }, exp: chrono::Utc::now().timestamp() + SESSION_EXPIRY_SECS };
                let token = create_session_token(&payload);
                info!(email = %email, role = %role, "Portal user login");
                return Json(serde_json::json!({"token":token,"email":email,"role":role,"display_name":display_name,"expires_in":SESSION_EXPIRY_SECS})).into_response();
            }
        }
    }
    // 2) Fallback: scan tenant members
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
            if member.email.to_lowercase() == email { member.last_login = Some(now_iso()); }
        }
    }
    let _ = save_data(&state, &data);
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

// ─── Tenants ─────────────────────────────────────────────────────────────────
pub async fn portal_tenants(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let email = session.email.to_lowercase();
    let tenants: Vec<serde_json::Value> = data.tenants.iter()
        .filter(|t| session.role == "admin" || session.tenant_ids.contains(&t.id) || t.members.iter().any(|m| m.email.to_lowercase() == email))
        .map(|t| serde_json::json!({"id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,"provider":t.provider,"model":t.model,"messages_today":t.messages_today,"max_messages_per_day":t.max_messages_per_day,"channels_active":t.channels_active,"members_count":t.members.len(),"created_at":t.created_at,"version":t.version}))
        .collect();
    Json(serde_json::json!({"tenants":tenants})).into_response()
}

pub async fn portal_tenant_detail(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let email = session.email.to_lowercase();
    match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => {
            let is_member = t.members.iter().any(|m| m.email.to_lowercase() == email);
            if session.role != "admin" && !session.tenant_ids.contains(&id) && !is_member {
                return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
            }
            let members: Vec<serde_json::Value> = t.members.iter().map(|m| serde_json::json!({"email":m.email,"role":m.role,"display_name":m.display_name,"added_at":m.added_at,"last_login":m.last_login,"has_password":m.password_hash.is_some()})).collect();
            Json(serde_json::json!({"id":t.id,"name":t.name,"slug":t.slug,"status":t.status,"plan":t.plan,"provider":t.provider,"model":t.model,"temperature":t.temperature,"messages_today":t.messages_today,"max_messages_per_day":t.max_messages_per_day,"max_channels":t.max_channels,"max_members":t.max_members,"channels":t.channels,"members":members,"created_at":t.created_at,"version":t.version,"access_token":t.access_token,"api_key":t.api_key,"system_prompt":t.system_prompt,"skills":t.skills,"hands":t.hands,"language":t.language,"webhook_url":t.webhook_url})).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

pub async fn portal_all_members(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let data = load_data(&state);
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

// ─── Member management ──────────────────────────────────────────────────────
pub async fn portal_set_password(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<SetPasswordRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    if req.password.len() < 4 { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Password must be at least 4 characters"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) { Some(m) => m, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response() };
    member.password_hash = Some(hash_password(&req.password));
    if let Some(name) = req.display_name { member.display_name = Some(name); }
    let _ = save_data(&state, &data);
    info!(email = %req.email, tenant = %id, "Set portal password");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_update_role(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<UpdateRoleRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let valid_roles = ["owner", "manager", "contributor", "viewer", "admin", "member"];
    let new_role = req.role.to_lowercase();
    if !valid_roles.contains(&new_role.as_str()) { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Invalid role"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) { Some(m) => m, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response() };
    member.role = new_role.clone();
    let _ = save_data(&state, &data);
    info!(email = %req.email, tenant = %id, role = %new_role, "Updated member role via portal");
    Json(serde_json::json!({"ok":true,"role":new_role})).into_response()
}

pub async fn portal_add_member(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<AddMemberRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    if email.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email is required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if tenant.members.iter().any(|m| m.email.to_lowercase() == email) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"Member already exists"}))).into_response();
    }
    let pw_hash = req.password.as_ref().filter(|p| p.len() >= 4).map(|p| hash_password(p));
    tenant.members.push(TenantMember { email: email.clone(), role: req.role.to_lowercase(), display_name: req.display_name.clone(), added_at: now_iso(), last_login: None, password_hash: pw_hash });
    let _ = save_data(&state, &data);
    info!(email = %email, tenant = %id, role = %req.role, "Added member via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_remove_member(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<RemoveMemberRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let before = tenant.members.len();
    tenant.members.retain(|m| m.email.to_lowercase() != email);
    if tenant.members.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Member not found"}))).into_response(); }
    let _ = save_data(&state, &data);
    info!(email = %email, tenant = %id, "Removed member via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Config, Actions ─────────────────────────────────────────────────────────
pub async fn portal_update_config(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalUpdateConfigRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if let Some(provider) = req.provider { tenant.provider = provider; }
    if let Some(model) = req.model { tenant.model = model; }
    if let Some(temp) = req.temperature { tenant.temperature = temp; }
    if let Some(key) = req.api_key { tenant.api_key = Some(key); }
    let updated = tenant.clone();
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, "Updated tenant config via portal");
    Json(serde_json::json!({"ok":true,"tenant":updated})).into_response()
}

pub async fn portal_restart(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.status = TenantStatus::Running; t.messages_today = 0; let _ = save_data(&state, &data); info!(tenant_id = %id, "Restarted tenant via portal"); Json(serde_json::json!({"ok":true,"status":"running"})).into_response() }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

pub async fn portal_stop(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => { t.status = TenantStatus::Stopped; let _ = save_data(&state, &data); info!(tenant_id = %id, "Stopped tenant via portal"); Json(serde_json::json!({"ok":true,"status":"stopped"})).into_response() }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(),
    }
}

pub async fn portal_delete_tenant(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let before = data.tenants.len();
    data.tenants.retain(|t| t.id != id);
    if data.tenants.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response(); }
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, "Deleted tenant via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Channels ────────────────────────────────────────────────────────────────
pub async fn portal_add_channel(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalAddChannelRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if tenant.channels.len() as u32 >= tenant.max_channels { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Channel limit reached"}))).into_response(); }
    let channel = TenantChannel { name: req.name.unwrap_or_else(|| req.channel_type.clone()), channel_type: req.channel_type, enabled: true, config: serde_json::json!({}), added_at: now_iso() };
    tenant.channels.push(channel);
    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, "Added channel via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_remove_channel(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalRemoveChannelRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let before = tenant.channels.len();
    tenant.channels.retain(|c| c.name != req.name);
    if tenant.channels.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel not found"}))).into_response(); }
    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, "Removed channel via portal");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_update_channel_config(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalUpdateChannelConfigRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let channel = match tenant.channels.iter_mut().find(|c| c.name == req.channel_name) { Some(c) => c, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel not found"}))).into_response() };
    let channel_type = channel.channel_type.clone();
    if let (Some(existing), Some(new_obj)) = (channel.config.as_object_mut(), req.config.as_object()) {
        for (k, v) in new_obj { existing.insert(k.clone(), v.clone()); }
    } else { channel.config = req.config; }
    let final_config = channel.config.clone();
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, channel = %req.channel_name, "Updated per-tenant channel config");

    // Push channel config to OpenFang so it actually starts working
    // OpenFang expects: {"fields": {"bot_token": "...", ...}}
    let openfang_body = serde_json::json!({"fields": final_config});
    let openfang_url = format!("{}/api/channels/{}/configure", state.openfang_api_url, channel_type);
    let client = reqwest::Client::new();
    let mut openfang_req = client.post(&openfang_url).json(&openfang_body);
    if !state.openfang_api_key.is_empty() {
        openfang_req = openfang_req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match openfang_req.send().await {
        Ok(resp) => {
            let status = resp.status();
            if status.is_success() {
                info!(channel_type = %channel_type, "Pushed channel config to OpenFang successfully");
                // Also reload channels on OpenFang
                let reload_url = format!("{}/api/channels/reload", state.openfang_api_url);
                let mut reload_req = client.post(&reload_url);
                if !state.openfang_api_key.is_empty() {
                    reload_req = reload_req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
                }
                let _ = reload_req.send().await;
            } else {
                let body = resp.text().await.unwrap_or_default();
                tracing::warn!(channel_type = %channel_type, status = %status, body = %body, "Failed to push channel config to OpenFang");
                return Json(serde_json::json!({"ok":true, "warning": format!("Saved locally but OpenFang returned {}: {}", status, body)})).into_response();
            }
        },
        Err(e) => {
            tracing::warn!(channel_type = %channel_type, error = %e, "Failed to connect to OpenFang for channel config");
            return Json(serde_json::json!({"ok":true, "warning": format!("Saved locally but could not reach OpenFang: {}", e)})).into_response();
        }
    }

    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Users CRUD ──────────────────────────────────────────────────────────────
pub async fn portal_list_users(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let data = load_data(&state);
    let users: Vec<serde_json::Value> = data.users.iter().map(|u| {
        let tenant_count = data.tenants.iter().filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == u.email.to_lowercase())).count();
        serde_json::json!({"email":u.email,"display_name":u.display_name,"role":u.role,"plan_id":u.plan_id,"created_at":u.created_at,"last_login":u.last_login,"max_tenants":u.max_tenants,"tenant_count":tenant_count,"has_password":u.password_hash.is_some()})
    }).collect();
    Json(serde_json::json!({"users":users})).into_response()
}

pub async fn portal_create_user(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(req): Json<PortalCreateUserRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let email = req.email.trim().to_lowercase();
    if email.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Email is required"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_defaults(&mut data) { let _ = save_data(&state, &data); data = load_data(&state); }
    if data.users.iter().any(|u| u.email.to_lowercase() == email) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"User already exists"}))).into_response();
    }
    let role = req.role.unwrap_or_else(|| "user".into());
    let plan_id = req.plan_id.clone().or_else(|| data.plans.iter().find(|p| p.is_default).map(|p| p.id.clone()));
    let max_t = plan_id.as_ref().and_then(|pid| data.plans.iter().find(|p| p.id == *pid)).map(|p| p.max_tenants).unwrap_or(3);
    let user = PortalUser { email: email.clone(), display_name: req.display_name, password_hash: req.password.filter(|p| p.len() >= 4).map(|p| hash_password(&p)), role, plan_id, created_at: now_iso(), last_login: None, max_tenants: max_t };
    data.users.push(user);
    let _ = save_data(&state, &data);
    info!(email = %email, "Created portal user");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_update_user(State(state): State<Arc<PortalState>>, Path(user_email): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalUpdateUserRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let target = user_email.to_lowercase();
    let mut data = load_data(&state);
    let user = match data.users.iter_mut().find(|u| u.email.to_lowercase() == target) { Some(u) => u, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"User not found"}))).into_response() };
    if let Some(name) = req.display_name { user.display_name = Some(name); }
    if let Some(role) = req.role { user.role = role; }
    if let Some(plan_id) = req.plan_id.clone() {
        user.plan_id = Some(plan_id.clone());
        if let Some(plan) = data.plans.iter().find(|p| p.id == plan_id) { user.max_tenants = plan.max_tenants; }
    }
    if let Some(pw) = req.password.filter(|p| p.len() >= 4) { user.password_hash = Some(hash_password(&pw)); }
    let _ = save_data(&state, &data);
    info!(email = %target, "Updated portal user");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_delete_user(State(state): State<Arc<PortalState>>, Path(user_email): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let target = user_email.to_lowercase();
    let mut data = load_data(&state);
    if !data.users.iter().any(|u| u.email.to_lowercase() == target) {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"User not found"}))).into_response();
    }
    let member_tenants: Vec<String> = data.tenants.iter()
        .filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == target))
        .map(|t| t.name.clone()).collect();
    for tenant in data.tenants.iter_mut() { tenant.members.retain(|m| m.email.to_lowercase() != target); }
    data.users.retain(|u| u.email.to_lowercase() != target);
    let _ = save_data(&state, &data);
    info!(email = %target, removed_from = ?member_tenants, "Deleted portal user and removed from tenants");
    Json(serde_json::json!({"ok":true,"removed_from_tenants":member_tenants})).into_response()
}

// ─── Plans CRUD ──────────────────────────────────────────────────────────────
pub async fn portal_list_plans(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" && session.role != "user" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Login required"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_defaults(&mut data) { let _ = save_data(&state, &data); }
    Json(serde_json::json!({"plans":data.plans})).into_response()
}

pub async fn portal_create_plan(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(req): Json<PortalCreatePlanRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_data(&state);
    let id = req.name.trim().to_lowercase().replace(' ', "-");
    if data.plans.iter().any(|p| p.id == id) { return (StatusCode::CONFLICT, Json(serde_json::json!({"error":"Plan already exists"}))).into_response(); }
    let plan = ServicePlan { id: id.clone(), name: req.name.trim().into(), max_messages_per_day: req.max_messages_per_day, max_channels: req.max_channels, max_members: req.max_members, max_tenants: req.max_tenants, price_label: req.price_label, is_default: false };
    data.plans.push(plan);
    let _ = save_data(&state, &data);
    info!(plan_id = %id, "Created service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_update_plan(State(state): State<Arc<PortalState>>, Path(plan_id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalCreatePlanRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_data(&state);
    let plan = match data.plans.iter_mut().find(|p| p.id == plan_id) { Some(p) => p, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Plan not found"}))).into_response() };
    plan.name = req.name.trim().into();
    plan.max_messages_per_day = req.max_messages_per_day;
    plan.max_channels = req.max_channels;
    plan.max_members = req.max_members;
    plan.max_tenants = req.max_tenants;
    plan.price_label = req.price_label;
    let _ = save_data(&state, &data);
    info!(plan_id = %plan_id, "Updated service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

pub async fn portal_delete_plan(State(state): State<Arc<PortalState>>, Path(plan_id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if session.role != "admin" { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin access required"}))).into_response(); }
    let mut data = load_data(&state);
    let before = data.plans.len();
    data.plans.retain(|p| p.id != plan_id);
    if data.plans.len() == before { return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Plan not found"}))).into_response(); }
    let _ = save_data(&state, &data);
    info!(plan_id = %plan_id, "Deleted service plan");
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Self-Service Tenant Creation ────────────────────────────────────────────
pub async fn portal_create_my_tenant(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(req): Json<PortalCreateMyTenantRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    if req.name.trim().is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Tenant name is required"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_defaults(&mut data) { let _ = save_data(&state, &data); data = load_data(&state); }
    let user = data.users.iter().find(|u| u.email.to_lowercase() == session.email.to_lowercase());
    let (max_msg, max_ch, max_mem, max_t) = if let Some(u) = user {
        let plan = u.plan_id.as_ref().and_then(|pid| data.plans.iter().find(|p| p.id == *pid));
        match plan { Some(p) => (p.max_messages_per_day, p.max_channels, p.max_members, u.max_tenants), None => (100, 3, 5, 3) }
    } else if session.role == "admin" { (u32::MAX, u32::MAX, u32::MAX, u32::MAX) } else { (100, 3, 5, 2) };
    let current_count = data.tenants.iter().filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == session.email.to_lowercase() && m.role == "owner")).count() as u32;
    if session.role != "admin" && current_count >= max_t {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":format!("Tenant limit reached ({}/{}). Please upgrade your plan.", current_count, max_t)}))).into_response();
    }
    let plan = if let Some(u) = user {
        match u.plan_id.as_deref() { Some("pro") => TenantPlan::Pro, Some("enterprise") => TenantPlan::Enterprise, _ => TenantPlan::Free }
    } else { TenantPlan::Free };
    let slug = generate_slug(req.name.trim());
    let tenant = Tenant {
        id: uuid::Uuid::new_v4().to_string(), name: req.name.trim().to_string(), slug, status: TenantStatus::Running, plan,
        provider: req.provider.unwrap_or_else(|| "groq".into()), model: req.model.unwrap_or_else(|| "llama-3.3-70b-versatile".into()), temperature: 0.7,
        max_messages_per_day: max_msg, max_channels: max_ch, max_members: max_mem, messages_today: 0, channels_active: 0,
        members: vec![TenantMember { email: session.email.clone(), role: "owner".into(), display_name: user.and_then(|u| u.display_name.clone()), added_at: now_iso(), last_login: None, password_hash: None }],
        access_token: generate_access_token(), created_at: now_iso(), version: format!("bizclaw-portal-{}", env!("CARGO_PKG_VERSION")),
        api_key: None, channels: vec![], system_prompt: String::new(), skills: vec![], hands: vec![], language: String::new(), webhook_url: None,
        agent_name: None, archetype: None, vibe: None, greeting_style: None, tool_profile: None,
    };
    let tid = tenant.id.clone();
    data.tenants.push(tenant);
    let _ = save_data(&state, &data);
    info!(email = %session.email, tenant_id = %tid, "User created tenant via portal");
    Json(serde_json::json!({"ok":true,"tenant_id":tid})).into_response()
}

// ─── Agent Config ────────────────────────────────────────────────────────────
pub async fn portal_update_agent(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<PortalUpdateAgentRequest>) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    if let Some(prompt) = req.system_prompt { tenant.system_prompt = prompt; }
    if let Some(skills) = req.skills { tenant.skills = skills; }
    if let Some(hands) = req.hands { tenant.hands = hands; }
    if let Some(lang) = req.language { tenant.language = lang; }
    if let Some(wh) = req.webhook_url { tenant.webhook_url = if wh.is_empty() { None } else { Some(wh) }; }
    if let Some(name) = req.agent_name { tenant.agent_name = if name.is_empty() { None } else { Some(name) }; }
    if let Some(v) = req.archetype { tenant.archetype = if v.is_empty() { None } else { Some(v) }; }
    if let Some(v) = req.vibe { tenant.vibe = if v.is_empty() { None } else { Some(v) }; }
    if let Some(v) = req.greeting_style { tenant.greeting_style = if v.is_empty() { None } else { Some(v) }; }
    if let Some(v) = req.tool_profile { tenant.tool_profile = if v.is_empty() { None } else { Some(v) }; }
    let _ = save_data(&state, &data);
    info!(tenant_id = %id, "Updated agent config via portal");

    // ── Auto-deploy agent on OpenFang ──────────────────────────────────────
    let deploy = req.deploy.unwrap_or(false);
    if !deploy {
        return Json(serde_json::json!({"ok":true,"deployed":false})).into_response();
    }

    let data = load_data(&state);
    let tenant = match data.tenants.iter().find(|t| t.id == id) { Some(t) => t, None => return Json(serde_json::json!({"ok":true,"deployed":false,"error":"Tenant disappeared"})).into_response() };

    let agent_name = tenant.agent_name.clone().unwrap_or_else(|| format!("portal-{}", tenant.slug));
    let provider = &tenant.provider;
    let model = &tenant.model;
    let sys_prompt = tenant.system_prompt.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
    let profile = tenant.tool_profile.as_deref().unwrap_or("full");

    let skills_toml = if tenant.skills.is_empty() { String::new() } else {
        format!("\nskills = [{}]", tenant.skills.iter().map(|s| format!("\"{}\"", s)).collect::<Vec<_>>().join(", "))
    };

    let manifest_toml = format!(
        "name = \"{}\"\nversion = \"0.1.0\"\ndescription = \"Auto-managed by Portal for tenant: {}\"\nmodule = \"builtin:chat\"\nprofile = \"{}\"\n\n[model]\nprovider = \"{}\"\nmodel = \"{}\"\ntemperature = {}\nsystem_prompt = \"{}\"\n{}",
        agent_name.replace('"', ""), tenant.name.replace('"', ""), profile, provider, model, tenant.temperature, sys_prompt, skills_toml,
    );

    let client = reqwest::Client::new();
    let auth_hdr = if !state.openfang_api_key.is_empty() { Some(format!("Bearer {}", state.openfang_api_key)) } else { None };

    // Step 1: Find and kill existing agent with same name
    let list_url = format!("{}/api/agents", state.openfang_api_url);
    let mut list_req = client.get(&list_url);
    if let Some(ref a) = auth_hdr { list_req = list_req.header("Authorization", a.clone()); }
    if let Ok(resp) = list_req.send().await {
        if let Ok(agents) = resp.json::<Vec<serde_json::Value>>().await {
            for agent in &agents {
                if agent.get("name").and_then(|n| n.as_str()) == Some(&agent_name) {
                    if let Some(aid) = agent.get("id").and_then(|i| i.as_str()) {
                        let kill_url = format!("{}/api/agents/{}", state.openfang_api_url, aid);
                        let mut kill_req = client.delete(&kill_url);
                        if let Some(ref a) = auth_hdr { kill_req = kill_req.header("Authorization", a.clone()); }
                        let _ = kill_req.send().await;
                        info!(tenant_id = %id, agent_id = %aid, "Killed existing portal-managed agent");
                    }
                }
            }
        }
    }

    // Step 2: Create new agent
    let spawn_url = format!("{}/api/agents", state.openfang_api_url);
    let spawn_body = serde_json::json!({"manifest_toml": manifest_toml});
    let mut spawn_req = client.post(&spawn_url).json(&spawn_body);
    if let Some(ref a) = auth_hdr { spawn_req = spawn_req.header("Authorization", a.clone()); }

    match spawn_req.send().await {
        Ok(resp) => {
            let st = resp.status();
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if st.is_success() {
                let agent_id = body.get("agent_id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                info!(tenant_id = %id, agent_id = %agent_id, "Auto-deployed agent on OpenFang");

                // Step 3: Set agent identity
                let identity = serde_json::json!({
                    "archetype": tenant.archetype.as_deref().unwrap_or("assistant"),
                    "vibe": tenant.vibe.as_deref().unwrap_or("professional"),
                    "greeting_style": tenant.greeting_style.as_deref().unwrap_or("warm"),
                });
                let id_url = format!("{}/api/agents/{}/identity", state.openfang_api_url, agent_id);
                let mut id_req = client.put(&id_url).json(&identity);
                if let Some(ref a) = auth_hdr { id_req = id_req.header("Authorization", a.clone()); }
                let _ = id_req.send().await;

                Json(serde_json::json!({"ok":true,"deployed":true,"agent_id":agent_id,"agent_name":agent_name})).into_response()
            } else {
                let err = body.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                warn!(tenant_id = %id, error = %err, "Failed to deploy agent on OpenFang");
                Json(serde_json::json!({"ok":true,"deployed":false,"deploy_error":err})).into_response()
            }
        }
        Err(e) => {
            warn!(tenant_id = %id, error = %e, "Failed to connect to OpenFang for agent deploy");
            Json(serde_json::json!({"ok":true,"deployed":false,"deploy_error":format!("Connection failed: {e}")})).into_response()
        }
    }
}

// ─── Chat with Agent ─────────────────────────────────────────────────────────
pub async fn portal_chat(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(req): Json<serde_json::Value>) -> impl IntoResponse {
    let _session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let tenant = match data.tenants.iter().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let agent_name = tenant.agent_name.clone().unwrap_or_else(|| format!("portal-{}", tenant.slug));
    let message = req.get("message").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if message.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Message is required"}))).into_response(); }

    let client = reqwest::Client::new();
    let auth = if !state.openfang_api_key.is_empty() { Some(format!("Bearer {}", state.openfang_api_key)) } else { None };

    // Find agent ID by name
    let list_url = format!("{}/api/agents", state.openfang_api_url);
    let mut list_req = client.get(&list_url);
    if let Some(ref a) = auth { list_req = list_req.header("Authorization", a.clone()); }
    let agent_id = match list_req.send().await {
        Ok(resp) => {
            let agents: Vec<serde_json::Value> = resp.json().await.unwrap_or_default();
            agents.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some(&agent_name))
                .and_then(|a| a.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
        }
        Err(_) => None,
    };
    let agent_id = match agent_id {
        Some(id) => id,
        None => return Json(serde_json::json!({"error":format!("Agent '{}' not found. Deploy the agent first.", agent_name)})).into_response(),
    };

    // Send message
    let msg_url = format!("{}/api/agents/{}/message", state.openfang_api_url, agent_id);
    let mut msg_req = client.post(&msg_url).json(&serde_json::json!({"message": message}));
    if let Some(ref a) = auth { msg_req = msg_req.header("Authorization", a.clone()); }

    match msg_req.send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            Json(body).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({"error":format!("Failed to reach agent: {e}")})).into_response()
        }
    }
}

// ─── Conversations (proxy to OpenFang agent session) ─────────────────────────
pub async fn portal_conversations(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let _session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let tenant = match data.tenants.iter().find(|t| t.id == id) { Some(t) => t, None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let agent_name = tenant.agent_name.clone().unwrap_or_else(|| format!("portal-{}", tenant.slug));

    let client = reqwest::Client::new();
    let auth = if !state.openfang_api_key.is_empty() { Some(format!("Bearer {}", state.openfang_api_key)) } else { None };

    // Find agent ID by name
    let list_url = format!("{}/api/agents", state.openfang_api_url);
    let mut list_req = client.get(&list_url);
    if let Some(ref a) = auth { list_req = list_req.header("Authorization", a.clone()); }
    let agent_id = match list_req.send().await {
        Ok(resp) => {
            let agents: Vec<serde_json::Value> = resp.json().await.unwrap_or_default();
            agents.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some(&agent_name))
                .and_then(|a| a.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
        }
        Err(_) => None,
    };
    let agent_id = match agent_id {
        Some(id) => id,
        None => return Json(serde_json::json!({"conversations":[],"error":"Agent not deployed yet"})).into_response(),
    };

    // Get agent session
    let sess_url = format!("{}/api/agents/{}/session", state.openfang_api_url, agent_id);
    let mut sess_req = client.get(&sess_url);
    if let Some(ref a) = auth { sess_req = sess_req.header("Authorization", a.clone()); }

    match sess_req.send().await {
        Ok(resp) => {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            Json(body).into_response()
        }
        Err(e) => {
            Json(serde_json::json!({"conversations":[],"error":format!("Failed: {e}")})).into_response()
        }
    }
}

// ─── Clone Tenant ────────────────────────────────────────────────────────────
pub async fn portal_clone_tenant(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &id) { return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response(); }
    let mut data = load_data(&state);
    let source = match data.tenants.iter().find(|t| t.id == id) { Some(t) => t.clone(), None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response() };
    let new_id = uuid::Uuid::new_v4().to_string();
    let clone = Tenant {
        id: new_id.clone(), name: format!("{} (Copy)", source.name), slug: format!("{}-copy", source.slug),
        status: TenantStatus::Stopped, messages_today: 0, channels_active: 0, channels: vec![],
        members: source.members.clone(), access_token: generate_access_token(), created_at: now_iso(), ..source
    };
    data.tenants.push(clone);
    let _ = save_data(&state, &data);
    info!(source = %id, clone = %new_id, "Cloned tenant via portal");
    Json(serde_json::json!({"ok":true,"tenant_id":new_id})).into_response()
}

// ─── System API Proxies (calls OpenFang via HTTP) ────────────────────────────
async fn proxy_get(state: &PortalState, path: &str) -> impl IntoResponse {
    let url = format!("{}{}", state.openfang_api_url, path);
    info!(url = %url, "Proxy GET request");
    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if !state.openfang_api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status();
            match resp.json::<serde_json::Value>().await {
                Ok(json) => { info!(url = %url, status = %status, "Proxy GET success"); Json(json).into_response() },
                Err(e) => { tracing::warn!(url = %url, error = %e, "Proxy parse error"); (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Parse error: {e}")}))).into_response() },
            }
        },
        Err(e) => { tracing::warn!(url = %url, error = %e, "Proxy connection error"); (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Proxy error: {e}")}))).into_response() },
    }
}

pub async fn portal_system_agents(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/agents").await.into_response()
}

pub async fn portal_system_channels(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/channels").await.into_response()
}

pub async fn portal_system_providers(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/providers").await.into_response()
}

pub async fn portal_system_models(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, query: axum::extract::Query<std::collections::HashMap<String, String>>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let qs = if query.is_empty() { String::new() } else { format!("?{}", query.iter().map(|(k,v)| format!("{k}={v}")).collect::<Vec<_>>().join("&")) };
    proxy_get(&state, &format!("/api/models{qs}")).await.into_response()
}

pub async fn portal_system_skills(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_skills(&mut data) { let _ = save_data(&state, &data); }
    Json(serde_json::json!({"skills": data.skills})).into_response()
}

pub async fn portal_system_hands(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/hands").await.into_response()
}

// ─── Scheduler (Cron Jobs) ────────────────────────────────────────────────────
pub async fn portal_list_cron_jobs(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/schedules").await.into_response()
}

pub async fn portal_create_cron_job(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, "/api/schedules", body).await.into_response()
}

pub async fn portal_toggle_cron_job(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Path(id): Path<String>, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_put(&state, &format!("/api/schedules/{}", id), body).await.into_response()
}

pub async fn portal_delete_cron_job(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Path(id): Path<String>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_delete(&state, &format!("/api/schedules/{}", id)).await.into_response()
}

// ─── Write Proxies (push config to OpenFang) ─────────────────────────────────

async fn proxy_post(state: &PortalState, path: &str, body: serde_json::Value) -> impl IntoResponse {
    let url = format!("{}{}", state.openfang_api_url, path);
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    if !state.openfang_api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match req.send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => Json(json).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Parse error: {e}")}))).into_response(),
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Proxy error: {e}")}))).into_response(),
    }
}

async fn proxy_delete(state: &PortalState, path: &str) -> impl IntoResponse {
    let url = format!("{}{}", state.openfang_api_url, path);
    let client = reqwest::Client::new();
    let mut req = client.delete(&url);
    if !state.openfang_api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match req.send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => Json(json).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Parse error: {e}")}))).into_response(),
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Proxy error: {e}")}))).into_response(),
    }
}

/// Configure a channel on OpenFang: POST /api/channels/{name}/configure
pub async fn portal_system_channel_configure(State(state): State<Arc<PortalState>>, Path(name): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, &format!("/api/channels/{}/configure", name), body).await.into_response()
}

/// Remove a channel from OpenFang: DELETE /api/channels/{name}/configure
pub async fn portal_system_channel_remove(State(state): State<Arc<PortalState>>, Path(name): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_delete(&state, &format!("/api/channels/{}/configure", name)).await.into_response()
}

/// Reload channels on OpenFang: POST /api/channels/reload
pub async fn portal_system_channels_reload(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, "/api/channels/reload", serde_json::json!({})).await.into_response()
}

/// Install a skill: POST /api/portal/system/skills/install
pub async fn portal_system_skill_install(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let skill_id = body["id"].as_str().unwrap_or("").to_string();
    let mut data = load_data(&state);
    if let Some(skill) = data.skills.iter_mut().find(|s| s.id == skill_id) {
        skill.installed = true;
        let _ = save_data(&state, &data);
        Json(serde_json::json!({"ok":true, "installed": true})).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Skill not found"}))).into_response()
    }
}

/// Uninstall a skill: POST /api/portal/system/skills/uninstall
pub async fn portal_system_skill_uninstall(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let skill_id = body["id"].as_str().unwrap_or("").to_string();
    let mut data = load_data(&state);
    if let Some(skill) = data.skills.iter_mut().find(|s| s.id == skill_id) {
        skill.installed = false;
        let _ = save_data(&state, &data);
        Json(serde_json::json!({"ok":true, "installed": false})).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Skill not found"}))).into_response()
    }
}

/// Activate a hand on OpenFang: POST /api/hands/{id}/activate
pub async fn portal_system_hand_activate(State(state): State<Arc<PortalState>>, Path(hand_id): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, &format!("/api/hands/{}/activate", hand_id), body).await.into_response()
}

/// Get hand details: GET /api/hands/{id}
pub async fn portal_system_hand_detail(State(state): State<Arc<PortalState>>, Path(hand_id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, &format!("/api/hands/{}", hand_id)).await.into_response()
}

/// Set provider API key: POST /api/providers/{name}/key
pub async fn portal_system_provider_key(State(state): State<Arc<PortalState>>, Path(name): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, &format!("/api/providers/{}/key", name), body).await.into_response()
}

/// Test provider: POST /api/providers/{name}/test
pub async fn portal_system_provider_test(State(state): State<Arc<PortalState>>, Path(name): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, &format!("/api/providers/{}/test", name), serde_json::json!({})).await.into_response()
}

async fn proxy_put(state: &PortalState, path: &str, body: serde_json::Value) -> impl IntoResponse {
    let url = format!("{}{}", state.openfang_api_url, path);
    let client = reqwest::Client::new();
    let mut req = client.put(&url).json(&body);
    if !state.openfang_api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match req.send().await {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => Json(json).into_response(),
            Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Parse error: {e}")}))).into_response(),
        },
        Err(e) => (StatusCode::BAD_GATEWAY, Json(serde_json::json!({"error": format!("Proxy error: {e}")}))).into_response(),
    }
}

// ─── Workflows (proxy to OpenFang) ───────────────────────────────────────────

pub async fn portal_list_workflows(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, "/api/workflows").await.into_response()
}

pub async fn portal_create_workflow(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, "/api/workflows", body).await.into_response()
}

pub async fn portal_run_workflow(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_post(&state, &format!("/api/workflows/{}/run", id), body).await.into_response()
}

pub async fn portal_workflow_runs(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_get(&state, &format!("/api/workflows/{}/runs", id)).await.into_response()
}

pub async fn portal_delete_workflow(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    proxy_delete(&state, &format!("/api/workflows/{}", id)).await.into_response()
}

/// Diagnostic: test OpenFang API connectivity
pub async fn portal_system_test(State(state): State<Arc<PortalState>>) -> impl IntoResponse {
    let url = format!("{}/api/providers", state.openfang_api_url);
    let has_key = !state.openfang_api_key.is_empty();
    let client = reqwest::Client::builder().timeout(std::time::Duration::from_secs(10)).build().unwrap_or_default();
    let mut req = client.get(&url);
    if has_key {
        req = req.header("Authorization", format!("Bearer {}", state.openfang_api_key));
    }
    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_else(|e| format!("read error: {e}"));
            Json(serde_json::json!({
                "openfang_api_url": state.openfang_api_url,
                "has_api_key": has_key,
                "test_url": url,
                "status": status,
                "connected": true,
                "response_preview": if body.len() > 500 { format!("{}...", &body[..500]) } else { body }
            })).into_response()
        },
        Err(e) => {
            Json(serde_json::json!({
                "openfang_api_url": state.openfang_api_url,
                "has_api_key": has_key,
                "test_url": url,
                "connected": false,
                "error": format!("{e}")
            })).into_response()
        }
    }
}

// ─── Independent Channel Instances (Multi-Channel Support) ───────────────────

/// List all channel instances, optionally filtered by tenant_id
pub async fn channel_instance_list(
    State(state): State<Arc<PortalState>>,
    headers: axum::http::HeaderMap,
    query: axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let instances: Vec<&ChannelInstance> = if let Some(tid) = query.get("tenant_id") {
        if !is_admin_or_owner(&session, &data, tid) {
            return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
        }
        data.channel_instances.iter().filter(|c| c.tenant_id == *tid).collect()
    } else if session.role == "admin" {
        data.channel_instances.iter().collect()
    } else {
        // Return only instances for tenants user owns
        let user_tenant_ids: Vec<&str> = data.tenants.iter()
            .filter(|t| t.members.iter().any(|m| m.email.to_lowercase() == session.email.to_lowercase()))
            .map(|t| t.id.as_str()).collect();
        data.channel_instances.iter().filter(|c| user_tenant_ids.contains(&c.tenant_id.as_str())).collect()
    };
    Json(serde_json::json!({"channel_instances": instances})).into_response()
}

/// Create a new channel instance
pub async fn channel_instance_create(
    State(state): State<Arc<PortalState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CreateChannelInstanceRequest>,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data_check = load_data(&state);
    if !is_admin_or_owner(&session, &data_check, &req.tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Admin or Owner access required"}))).into_response();
    }
    if req.display_name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Display name is required"}))).into_response();
    }
    let valid_types = ["telegram", "zalo", "discord", "slack", "whatsapp", "facebook", "email", "web"];
    if !valid_types.contains(&req.channel_type.as_str()) {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":format!("Invalid channel type. Supported: {:?}", valid_types)}))).into_response();
    }

    let mut data = load_data(&state);
    let id = uuid::Uuid::new_v4().to_string();
    let webhook_path = format!("/webhook/ch/{}", &id[..8]);
    let instance = ChannelInstance {
        id: id.clone(),
        tenant_id: req.tenant_id.clone(),
        channel_type: req.channel_type.clone(),
        display_name: req.display_name.trim().to_string(),
        enabled: true,
        config: req.config.unwrap_or(serde_json::json!({})),
        webhook_path: webhook_path.clone(),
        created_at: now_iso(),
        last_message_at: None,
        message_count: 0,
        status: ChannelInstanceStatus::Pending,
    };
    data.channel_instances.push(instance);
    let _ = save_data(&state, &data);
    info!(id = %id, tenant = %req.tenant_id, channel_type = %req.channel_type, "Created channel instance");
    Json(serde_json::json!({"ok":true, "id": id, "webhook_path": webhook_path})).into_response()
}

/// Get channel instance detail
pub async fn channel_instance_detail(
    State(state): State<Arc<PortalState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let instance = match data.channel_instances.iter().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel instance not found"}))).into_response(),
    };
    if !is_admin_or_owner(&session, &data, &instance.tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }
    Json(serde_json::json!(instance)).into_response()
}

/// Update a channel instance (display_name, enabled, config)
pub async fn channel_instance_update(
    State(state): State<Arc<PortalState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(req): Json<UpdateChannelInstanceRequest>,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let mut data = load_data(&state);
    let instance = match data.channel_instances.iter_mut().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel instance not found"}))).into_response(),
    };
    if !is_admin_or_owner(&session, &load_data(&state), &instance.tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }
    if let Some(name) = req.display_name { instance.display_name = name; }
    if let Some(enabled) = req.enabled { instance.enabled = enabled; }
    if let Some(config) = req.config {
        if let (Some(existing), Some(new_obj)) = (instance.config.as_object_mut(), config.as_object()) {
            for (k, v) in new_obj { existing.insert(k.clone(), v.clone()); }
        } else {
            instance.config = config;
        }
    }
    let _ = save_data(&state, &data);
    info!(id = %id, "Updated channel instance");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// Delete a channel instance
pub async fn channel_instance_delete(
    State(state): State<Arc<PortalState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let mut data = load_data(&state);
    let tenant_id = match data.channel_instances.iter().find(|c| c.id == id) {
        Some(c) => c.tenant_id.clone(),
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel instance not found"}))).into_response(),
    };
    if !is_admin_or_owner(&session, &data, &tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }

    // If Telegram, delete webhook before removing
    let instance = data.channel_instances.iter().find(|c| c.id == id).unwrap();
    if instance.channel_type == "telegram" {
        if let Some(token) = instance.config.get("bot_token").and_then(|v| v.as_str()) {
            let _ = crate::channels::telegram::delete_webhook(token).await;
        }
    }

    data.channel_instances.retain(|c| c.id != id);
    let _ = save_data(&state, &data);
    info!(id = %id, "Deleted channel instance");
    Json(serde_json::json!({"ok":true})).into_response()
}

/// Test channel instance connectivity (verify bot token, etc.)
pub async fn channel_instance_test(
    State(state): State<Arc<PortalState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let instance = match data.channel_instances.iter().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel instance not found"}))).into_response(),
    };
    if !is_admin_or_owner(&session, &data, &instance.tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }

    match instance.channel_type.as_str() {
        "telegram" => {
            let token = match instance.config.get("bot_token").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return Json(serde_json::json!({"ok":false, "error":"Missing bot_token in config"})).into_response(),
            };
            match crate::channels::telegram::get_bot_info(token).await {
                Ok(bot_info) => {
                    // Auto-update status to active
                    let mut data = load_data(&state);
                    if let Some(inst) = data.channel_instances.iter_mut().find(|c| c.id == id) {
                        inst.status = ChannelInstanceStatus::Active;
                        let _ = save_data(&state, &data);
                    }
                    Json(serde_json::json!({"ok":true, "bot_info": bot_info, "status": "active"})).into_response()
                }
                Err(e) => {
                    let mut data = load_data(&state);
                    if let Some(inst) = data.channel_instances.iter_mut().find(|c| c.id == id) {
                        inst.status = ChannelInstanceStatus::Error;
                        let _ = save_data(&state, &data);
                    }
                    Json(serde_json::json!({"ok":false, "error": e, "status": "error"})).into_response()
                }
            }
        }
        _ => Json(serde_json::json!({"ok":false, "error": format!("Test not implemented for channel type: {}", instance.channel_type)})).into_response(),
    }
}

/// Set webhook URL for a channel instance (Telegram: calls setWebhook)
pub async fn channel_instance_set_webhook(
    State(state): State<Arc<PortalState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let session = match extract_session(&headers) { Some(s) => s, None => return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response() };
    let data = load_data(&state);
    let instance = match data.channel_instances.iter().find(|c| c.id == id) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Channel instance not found"}))).into_response(),
    };
    if !is_admin_or_owner(&session, &data, &instance.tenant_id) {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({"error":"Access denied"}))).into_response();
    }

    // The base_url comes from the request body or we auto-detect it
    let base_url = body.get("base_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if base_url.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"base_url is required (e.g. https://portal.example.com)"}))).into_response();
    }

    match instance.channel_type.as_str() {
        "telegram" => {
            let token = match instance.config.get("bot_token").and_then(|v| v.as_str()) {
                Some(t) => t,
                None => return Json(serde_json::json!({"ok":false, "error":"Missing bot_token in config"})).into_response(),
            };
            let webhook_url = format!("{}{}", base_url.trim_end_matches('/'), instance.webhook_path);
            match crate::channels::telegram::set_webhook(token, &webhook_url).await {
                Ok(()) => {
                    let mut data = load_data(&state);
                    if let Some(inst) = data.channel_instances.iter_mut().find(|c| c.id == id) {
                        inst.status = ChannelInstanceStatus::Active;
                        let _ = save_data(&state, &data);
                    }
                    Json(serde_json::json!({"ok":true, "webhook_url": webhook_url})).into_response()
                }
                Err(e) => Json(serde_json::json!({"ok":false, "error": e})).into_response(),
            }
        }
        _ => Json(serde_json::json!({"ok":false, "error": "Webhook not supported for this channel type yet"})).into_response(),
    }
}

// ─── Webhook Receivers (incoming messages from channels) ─────────────────────

/// Handle incoming webhook from any channel (POST /webhook/ch/{id})
pub async fn channel_webhook_receive(
    State(state): State<Arc<PortalState>>,
    Path(short_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let data = load_data(&state);
    let webhook_path = format!("/webhook/ch/{}", short_id);

    let instance = match data.channel_instances.iter().find(|c| c.webhook_path == webhook_path && c.enabled) {
        Some(c) => c.clone(),
        None => {
            warn!(webhook_path = %webhook_path, "Webhook received for unknown channel instance");
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Unknown channel"}))).into_response();
        }
    };

    let tenant = match data.tenants.iter().find(|t| t.id == instance.tenant_id) {
        Some(t) => t.clone(),
        None => {
            warn!(tenant_id = %instance.tenant_id, "Webhook: tenant not found for channel instance");
            return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tenant not found"}))).into_response();
        }
    };

    info!(channel_id = %instance.id, channel_type = %instance.channel_type, tenant = %tenant.name, "Webhook received");

    match instance.channel_type.as_str() {
        "telegram" => {
            let msg = match crate::channels::telegram::parse_webhook(&body) {
                Some(m) => m,
                None => {
                    // Telegram sends various update types (inline, callback, etc.) — ignore non-text
                    return Json(serde_json::json!({"ok":true, "skipped":true})).into_response();
                }
            };

            info!(sender = %msg.sender_id, text_len = msg.text.len(), "Telegram message from: {}", msg.sender_name.as_deref().unwrap_or("?"));

            // Route to OpenFang Agent
            let agent_name = tenant.agent_name.clone().unwrap_or_else(|| format!("portal-{}", tenant.slug));
            let client = reqwest::Client::new();
            let auth = if !state.openfang_api_key.is_empty() { Some(format!("Bearer {}", state.openfang_api_key)) } else { None };

            // Find agent ID
            let list_url = format!("{}/api/agents", state.openfang_api_url);
            let mut list_req = client.get(&list_url);
            if let Some(ref a) = auth { list_req = list_req.header("Authorization", a.clone()); }
            let agent_id = match list_req.send().await {
                Ok(resp) => {
                    let agents: Vec<serde_json::Value> = resp.json().await.unwrap_or_default();
                    agents.iter().find(|a| a.get("name").and_then(|n| n.as_str()) == Some(&agent_name))
                        .and_then(|a| a.get("id").and_then(|i| i.as_str()).map(|s| s.to_string()))
                }
                Err(_) => None,
            };

            let agent_id = match agent_id {
                Some(id) => id,
                None => {
                    warn!(agent_name = %agent_name, "Agent not deployed, cannot process webhook message");
                    // Reply on Telegram that agent is not ready
                    let _ = crate::channels::telegram::send_reply(&instance.config, &msg.chat_id, "⚠️ Agent chưa được deploy. Vui lòng liên hệ admin.").await;
                    return Json(serde_json::json!({"ok":true, "error":"Agent not deployed"})).into_response();
                }
            };

            // Send message to agent
            let msg_url = format!("{}/api/agents/{}/message", state.openfang_api_url, agent_id);
            let mut msg_req = client.post(&msg_url).json(&serde_json::json!({"message": msg.text}));
            if let Some(ref a) = auth { msg_req = msg_req.header("Authorization", a.clone()); }

            let response_text = match msg_req.send().await {
                Ok(resp) => {
                    let body: serde_json::Value = resp.json().await.unwrap_or_default();
                    body.get("response").and_then(|v| v.as_str()).unwrap_or("(no response)").to_string()
                }
                Err(e) => format!("❌ Lỗi kết nối: {}", e),
            };

            // Send reply back to Telegram
            let result = crate::channels::telegram::send_reply(&instance.config, &msg.chat_id, &response_text).await;

            // Update message count
            let mut data = load_data(&state);
            if let Some(inst) = data.channel_instances.iter_mut().find(|c| c.id == instance.id) {
                inst.message_count += 1;
                inst.last_message_at = Some(now_iso());
            }
            let _ = save_data(&state, &data);

            if result.success {
                Json(serde_json::json!({"ok":true})).into_response()
            } else {
                Json(serde_json::json!({"ok":false, "error": result.error})).into_response()
            }
        }
        _ => {
            Json(serde_json::json!({"ok":false, "error": format!("Channel type {} not yet supported", instance.channel_type)})).into_response()
        }
    }
}

/// Handle webhook verification (GET /webhook/ch/{id}) — needed by some platforms
pub async fn channel_webhook_verify(
    Path(short_id): Path<String>,
) -> impl IntoResponse {
    // Telegram doesn't need GET verification, but we return OK for health checks
    Json(serde_json::json!({"ok":true, "channel_id": short_id})).into_response()
}

// ─── Independent Portal Features (local data, no OpenFang dependency) ────────

use crate::models::{KnowledgeDoc, PortalTool, PortalSkill, AgentTemplate, Delegation, KanbanTask, LlmTrace, ActivityEvent, PortalApiKey};

/// Seed default tools if empty.
fn seed_tools(data: &mut crate::models::PortalData) -> bool {
    if !data.tools.is_empty() { return false; }
    let defaults = vec![
        ("shell", "🖥️", "Execute system commands (sandboxed)"),
        ("web_search", "🔍", "Search the web for information"),
        ("http_request", "🌐", "Make HTTP API calls"),
        ("execute_code", "💻", "Run Python/JavaScript code"),
        ("read_file", "📄", "Read file contents"),
        ("write_file", "✏️", "Write file contents"),
        ("list_dir", "📁", "List directory contents"),
        ("rag_search", "📚", "Search Knowledge Base"),
        ("memory_recall", "🧠", "Recall from memory"),
        ("calculator", "🔢", "Mathematical calculations"),
        ("datetime", "🕐", "Get current date/time"),
        ("json_parse", "📋", "Parse JSON data"),
        ("text_transform", "🔤", "Transform text (upper, lower, etc.)"),
    ];
    for (name, icon, desc) in defaults {
        data.tools.push(PortalTool { name: name.to_string(), icon: icon.to_string(), desc: desc.to_string(), enabled: true, builtin: true });
    }
    true
}

/// Seed default skills if empty.
fn seed_skills(data: &mut crate::models::PortalData) -> bool {
    if !data.skills.is_empty() { return false; }
    let defaults = vec![
        ("rust-expert", "Rust Expert", "🦀", "coding", "Rust expert: ownership, async, lifetimes, error handling"),
        ("python-analyst", "Python Analyst", "🐍", "data", "Data analysis with pandas, numpy, matplotlib"),
        ("web-developer", "Web Developer", "🌐", "coding", "Full-stack: HTML, CSS, JS, React, Node.js"),
        ("security-auditor", "Security Auditor", "🔒", "security", "Vulnerability scanning, code audit, OWASP"),
        ("business-analyst", "Business Analyst", "💼", "business", "Market research, financial analysis, strategy"),
        ("content-writer", "Content Writer", "✍️", "writing", "SEO writing, blog posts, marketing copy"),
        ("devops-engineer", "DevOps Engineer", "⚙️", "coding", "Docker, CI/CD, Kubernetes, cloud infra"),
        ("data-scientist", "Data Scientist", "📊", "data", "ML, statistics, data visualization, prediction"),
        ("qa-tester", "QA Tester", "🧪", "coding", "Test automation, bug reporting, quality assurance"),
        ("research-assistant", "Research Assistant", "🔬", "research", "Academic research, literature review, summarization"),
    ];
    for (id, name, icon, cat, desc) in defaults {
        data.skills.push(PortalSkill { id: id.to_string(), name: name.to_string(), icon: icon.to_string(), category: cat.to_string(), description: desc.to_string(), version: "1.0.0".to_string(), installed: false, builtin: true });
    }
    true
}

/// Seed default gallery templates if empty.
fn seed_gallery(data: &mut crate::models::PortalData) -> bool {
    if !data.gallery.is_empty() { return false; }
    let defaults = vec![
        ("cs-agent", "Customer Support", "🎧", "Support", "Friendly customer service agent with FAQ knowledge"),
        ("sales-agent", "Sales Assistant", "💰", "Sales", "Lead qualification, product recommendations, follow-ups"),
        ("hr-agent", "HR Assistant", "👤", "HR", "Employee onboarding, policy Q&A, leave management"),
        ("marketing-agent", "Marketing Planner", "📢", "Marketing", "Campaign planning, content calendar, social media"),
        ("it-helpdesk", "IT Helpdesk", "🖥️", "IT", "Technical support, troubleshooting, ticket management"),
        ("finance-agent", "Finance Advisor", "📊", "Finance", "Budget analysis, expense tracking, financial reports"),
        ("legal-agent", "Legal Assistant", "⚖️", "Legal", "Contract review, compliance checks, legal research"),
        ("project-manager", "Project Manager", "📋", "Management", "Task tracking, timeline management, status reports"),
    ];
    for (id, name, icon, cat, desc) in defaults {
        data.gallery.push(AgentTemplate { id: id.to_string(), name: name.to_string(), icon: icon.to_string(), category: cat.to_string(), description: desc.to_string(), system_prompt: String::new() });
    }
    true
}

// ─── Knowledge Base ──────────────────────────────────────────────────────────

pub async fn portal_knowledge_list(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let docs = &data.knowledge_docs;
    let total_chunks: u32 = docs.iter().map(|d| d.chunks).sum();
    Json(serde_json::json!({"ok":true, "documents": docs, "total_docs": docs.len(), "total_chunks": total_chunks})).into_response()
}

pub async fn portal_knowledge_upload(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let content = body["content"].as_str().unwrap_or("").to_string();
    let name = body["name"].as_str().unwrap_or("Untitled").to_string();
    if content.is_empty() { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error":"Content is required"}))).into_response(); }
    let chunks = (content.len() / 500).max(1) as u32; // ~500 chars per chunk
    let size_bytes = content.len() as u64;
    let doc = KnowledgeDoc {
        id: uuid::Uuid::new_v4().to_string(), tenant_id: String::new(), name, content, chunks, size_bytes,
        created_at: crate::models::now_iso(),
    };
    let mut data = load_data(&state);
    let id = doc.id.clone();
    data.knowledge_docs.push(doc);
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true, "id": id})).into_response()
}

pub async fn portal_knowledge_delete(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    data.knowledge_docs.retain(|d| d.id != id);
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Tools ───────────────────────────────────────────────────────────────────

pub async fn portal_tools_list(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_tools(&mut data) { let _ = save_data(&state, &data); }
    Json(serde_json::json!({"tools": data.tools})).into_response()
}

pub async fn portal_tools_toggle(State(state): State<Arc<PortalState>>, Path(name): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_tools(&mut data) { /* seeded */ }
    let enabled = body["enabled"].as_bool().unwrap_or(false);
    if let Some(tool) = data.tools.iter_mut().find(|t| t.name == name) {
        tool.enabled = enabled;
        let _ = save_data(&state, &data);
        Json(serde_json::json!({"ok":true, "name": name, "enabled": enabled})).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Tool not found"}))).into_response()
    }
}

// ─── LLM Traces ──────────────────────────────────────────────────────────────

pub async fn portal_traces(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    Json(serde_json::json!({"traces": data.traces})).into_response()
}

// ─── Cost Tracking (aggregated from traces) ──────────────────────────────────

pub async fn portal_cost(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let mut model_map: std::collections::HashMap<String, (u32, u32, f64)> = std::collections::HashMap::new();
    for t in &data.traces {
        let entry = model_map.entry(t.model.clone()).or_insert((0, 0, 0.0));
        entry.0 += 1; // requests
        entry.1 += t.prompt_tokens + t.completion_tokens; // tokens
        entry.2 += t.cost; // cost
    }
    let total_cost: f64 = model_map.values().map(|v| v.2).sum();
    let models: Vec<serde_json::Value> = model_map.iter().map(|(model, (reqs, tokens, cost))| {
        let pct = if total_cost > 0.0 { cost / total_cost * 100.0 } else { 0.0 };
        serde_json::json!({"model": model, "requests": reqs, "tokens": tokens, "cost": cost, "percentage": pct})
    }).collect();
    Json(serde_json::json!({"models": models, "total_cost": total_cost})).into_response()
}

// ─── Activity Feed ───────────────────────────────────────────────────────────

pub async fn portal_activity(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    Json(serde_json::json!({"events": data.activity})).into_response()
}

pub async fn portal_activity_clear(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    data.activity.clear();
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── API Keys ────────────────────────────────────────────────────────────────

pub async fn portal_apikeys_list(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let keys: Vec<serde_json::Value> = data.api_keys.iter().map(|k| serde_json::json!({"id": k.id, "name": k.name, "prefix": k.key_prefix, "created_at": k.created_at})).collect();
    Json(serde_json::json!({"keys": keys})).into_response()
}

pub async fn portal_apikeys_create(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let name = body["name"].as_str().unwrap_or("Unnamed").to_string();
    let raw_key = format!("pk_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    let prefix = format!("{}...", &raw_key[..12]);
    let key = PortalApiKey {
        id: uuid::Uuid::new_v4().to_string(), tenant_id: String::new(), name, key_hash: raw_key.clone(), key_prefix: prefix,
        created_at: crate::models::now_iso(),
    };
    let mut data = load_data(&state);
    data.api_keys.push(key);
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true, "key": raw_key, "id": data.api_keys.last().map(|k| k.id.clone()).unwrap_or_default()})).into_response()
}

pub async fn portal_apikeys_delete(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    data.api_keys.retain(|k| k.id != id);
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Usage & Quotas (computed from PortalData) ───────────────────────────────

pub async fn portal_usage(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let tokens_used: u32 = data.traces.iter().map(|t| t.prompt_tokens + t.completion_tokens).sum();
    Json(serde_json::json!({
        "agents_used": data.tenants.len(), "agents_max": 10,
        "tokens_used": tokens_used, "tokens_max": 100000,
        "requests_used": data.traces.len(), "requests_max": 10000,
        "apikeys_used": data.api_keys.len(), "apikeys_max": 5
    })).into_response()
}

// ─── Org Map (computed from tenants) ─────────────────────────────────────────

pub async fn portal_orgmap(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let nodes: Vec<serde_json::Value> = data.tenants.iter().map(|t| {
        serde_json::json!({"id": t.id, "name": t.agent_name.as_deref().unwrap_or(&t.name), "icon": "🤖", "role": format!("{}", t.status), "type": "agent"})
    }).collect();
    Json(serde_json::json!({"nodes": nodes})).into_response()
}

// ─── Kanban Board ────────────────────────────────────────────────────────────

pub async fn portal_kanban(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    Json(serde_json::json!({"tasks": data.kanban_tasks})).into_response()
}

pub async fn portal_kanban_update(State(state): State<Arc<PortalState>>, Path(id): Path<String>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    if let Some(task) = data.kanban_tasks.iter_mut().find(|t| t.id == id) {
        if let Some(status) = body["status"].as_str() { task.status = status.to_string(); }
        if let Some(title) = body["title"].as_str() { task.title = title.to_string(); }
        let _ = save_data(&state, &data);
        Json(serde_json::json!({"ok":true})).into_response()
    } else {
        (StatusCode::NOT_FOUND, Json(serde_json::json!({"error":"Task not found"}))).into_response()
    }
}

// ─── Gallery (Agent Templates) ───────────────────────────────────────────────

pub async fn portal_gallery(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let mut data = load_data(&state);
    if seed_gallery(&mut data) { let _ = save_data(&state, &data); }
    Json(serde_json::json!({"templates": data.gallery})).into_response()
}

// ─── Config File ─────────────────────────────────────────────────────────────

pub async fn portal_configfile(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    // Return a summary of portal config as editable text
    let data = load_data(&state);
    let config = format!(
        "# Portal Configuration\n\n[portal]\ntenants_count = {}\nusers_count = {}\nchannels_count = {}\ntools_count = {}\nskills_count = {}\n\n[defaults]\nmax_messages_per_day = 1000\nmax_channels = 10\nmax_members = 50\n",
        data.tenants.len(), data.users.len(), data.channel_instances.len(), data.tools.len(), data.skills.len()
    );
    Json(serde_json::json!({"content": config})).into_response()
}

pub async fn portal_configfile_save(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let _content = body["content"].as_str().unwrap_or("");
    // Config is read-only for now (computed from PortalData)
    Json(serde_json::json!({"ok":true})).into_response()
}

// ─── Orchestration (Delegation) ──────────────────────────────────────────────

pub async fn portal_orchestration(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let data = load_data(&state);
    let links = vec![
        serde_json::json!({"name":"delegate","desc":"Agent-to-agent delegate","enabled":true}),
        serde_json::json!({"name":"handoff","desc":"Agent-to-agent handoff","enabled":true}),
        serde_json::json!({"name":"broadcast","desc":"Agent-to-agent broadcast","enabled":true}),
        serde_json::json!({"name":"escalate","desc":"Agent-to-agent escalate","enabled":true}),
    ];
    Json(serde_json::json!({"delegations": data.delegations, "links": links})).into_response()
}

pub async fn portal_orchestration_create(State(state): State<Arc<PortalState>>, headers: axum::http::HeaderMap, Json(body): Json<serde_json::Value>) -> impl IntoResponse {
    if extract_session(&headers).is_none() { return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"Unauthorized"}))).into_response(); }
    let from = body["from_agent"].as_str().unwrap_or("").to_string();
    let to = body["to_agent"].as_str().unwrap_or("").to_string();
    let link_type = body["link_type"].as_str().unwrap_or("delegate").to_string();
    let delegation = Delegation { id: uuid::Uuid::new_v4().to_string(), from_agent: from, to_agent: to, link_type, enabled: true };
    let mut data = load_data(&state);
    data.delegations.push(delegation);
    let _ = save_data(&state, &data);
    Json(serde_json::json!({"ok":true})).into_response()
}
