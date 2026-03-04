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
                        // Use highest role found across tenants
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

    // Update last_login for all matched members
    for tenant in &mut data.tenants {
        for member in &mut tenant.members {
            if member.email.to_lowercase() == email {
                member.last_login = Some(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true));
            }
        }
    }
    let _ = save_tenants(&state, &data);

    // Create session token
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
pub async fn portal_me(
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    match extract_session(&headers) {
        Some(session) => Json(serde_json::json!({
            "email": session.email,
            "role": session.role,
            "tenant_ids": session.tenant_ids,
        })).into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        ).into_response(),
    }
}

/// GET /api/portal/tenants — List tenants (filtered by role).
pub async fn portal_tenants(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        ).into_response(),
    };

    let data = load_tenants(&state);
    let tenants: Vec<serde_json::Value> = data.tenants.iter()
        .filter(|t| {
            // Admin sees all, member sees only assigned
            session.role == "admin" || session.tenant_ids.contains(&t.id)
        })
        .map(|t| serde_json::json!({
            "id": t.id,
            "name": t.name,
            "slug": t.slug,
            "status": t.status,
            "plan": t.plan,
            "provider": t.provider,
            "model": t.model,
            "messages_today": t.messages_today,
            "max_messages_per_day": t.max_messages_per_day,
            "channels_active": t.channels_active,
            "members_count": t.members.len(),
            "created_at": t.created_at,
        }))
        .collect();

    Json(serde_json::json!({"tenants": tenants})).into_response()
}

/// GET /api/portal/tenants/:id — Tenant detail (if authorized).
pub async fn portal_tenant_detail(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        ).into_response(),
    };

    if session.role != "admin" && !session.tenant_ids.contains(&id) {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "You don't have access to this tenant"})),
        ).into_response();
    }

    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => {
            // Members list omits password_hash for security
            let members: Vec<serde_json::Value> = t.members.iter().map(|m| serde_json::json!({
                "email": m.email,
                "role": m.role,
                "display_name": m.display_name,
                "added_at": m.added_at,
                "last_login": m.last_login,
                "has_password": m.password_hash.is_some(),
            })).collect();

            Json(serde_json::json!({
                "id": t.id,
                "name": t.name,
                "slug": t.slug,
                "status": t.status,
                "plan": t.plan,
                "provider": t.provider,
                "model": t.model,
                "temperature": t.temperature,
                "messages_today": t.messages_today,
                "max_messages_per_day": t.max_messages_per_day,
                "max_channels": t.max_channels,
                "max_members": t.max_members,
                "channels": t.channels,
                "members": members,
                "created_at": t.created_at,
            })).into_response()
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tenant not found"})),
        ).into_response(),
    }
}

/// GET /api/portal/members — Global members list (admin only).
pub async fn portal_all_members(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let session = match extract_session(&headers) {
        Some(s) => s,
        None => return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        ).into_response(),
    };

    if session.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ).into_response();
    }

    let data = load_tenants(&state);
    let mut members_map: std::collections::HashMap<String, serde_json::Value> = std::collections::HashMap::new();

    for tenant in &data.tenants {
        for member in &tenant.members {
            let key = member.email.to_lowercase();
            let entry = members_map.entry(key.clone()).or_insert_with(|| serde_json::json!({
                "email": member.email,
                "display_name": member.display_name,
                "role": member.role,
                "has_password": member.password_hash.is_some(),
                "last_login": member.last_login,
                "tenants": [],
            }));
            if let Some(arr) = entry["tenants"].as_array_mut() {
                arr.push(serde_json::json!({
                    "id": tenant.id,
                    "name": tenant.name,
                    "role": member.role,
                }));
            }
            // Upgrade role to highest
            if member.role == "admin" {
                entry["role"] = serde_json::json!("admin");
            }
        }
    }

    let members: Vec<serde_json::Value> = members_map.into_values().collect();
    Json(serde_json::json!({"members": members})).into_response()
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
        None => return (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid or expired session"})),
        ).into_response(),
    };

    if session.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ).into_response();
    }

    if req.password.len() < 4 {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Password must be at least 4 characters"})),
        ).into_response();
    }

    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Tenant not found"})),
        ).into_response(),
    };

    let member = match tenant.members.iter_mut().find(|m| m.email.to_lowercase() == req.email.to_lowercase()) {
        Some(m) => m,
        None => return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Member not found"})),
        ).into_response(),
    };

    member.password_hash = Some(hash_password(&req.password));
    if let Some(name) = req.display_name {
        member.display_name = Some(name);
    }

    let _ = save_tenants(&state, &data);
    info!(email = %req.email, tenant = %id, "Set portal password");

    Json(serde_json::json!({"ok": true})).into_response()
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
<style>
*{margin:0;padding:0;box-sizing:border-box}
:root{--bg:#0f0f23;--surface:#1a1a2e;--surface2:#16213e;--border:#2a2a4a;--text:#e0e0e0;--text-dim:#8888aa;--primary:#6c63ff;--primary-hover:#5a52d5;--accent:#00d4aa;--danger:#ff4757}
body{font-family:'Segoe UI',system-ui,sans-serif;background:var(--bg);color:var(--text);min-height:100vh}
.login-container{display:flex;align-items:center;justify-content:center;min-height:100vh;padding:20px}
.login-card{background:var(--surface);border:1px solid var(--border);border-radius:12px;padding:40px;width:100%;max-width:400px}
.login-card h1{font-size:1.5rem;margin-bottom:8px;display:flex;align-items:center;gap:8px}
.login-card p{color:var(--text-dim);font-size:0.85rem;margin-bottom:24px}
.form-group{margin-bottom:16px}
.form-group label{display:block;font-size:0.8rem;color:var(--text-dim);margin-bottom:4px}
.form-group input{width:100%;padding:10px 14px;background:var(--surface2);border:1px solid var(--border);border-radius:8px;color:var(--text);font-size:0.9rem;outline:none}
.form-group input:focus{border-color:var(--primary)}
.btn-primary{width:100%;padding:12px;background:var(--primary);color:#fff;border:none;border-radius:8px;font-size:0.95rem;font-weight:600;cursor:pointer;transition:background 0.2s}
.btn-primary:hover{background:var(--primary-hover)}
.btn-primary:disabled{opacity:0.5;cursor:not-allowed}
.error-msg{color:var(--danger);font-size:0.8rem;margin-top:8px;display:none}
/* Dashboard */
.dashboard{display:none;height:100vh;overflow:hidden}
.dash-header{background:var(--surface);border-bottom:1px solid var(--border);padding:12px 20px;display:flex;align-items:center;justify-content:space-between}
.dash-header h1{font-size:1rem;display:flex;align-items:center;gap:8px}
.dash-header .user-info{display:flex;align-items:center;gap:12px;font-size:0.85rem}
.dash-header .badge{background:var(--primary);color:#fff;padding:2px 8px;border-radius:4px;font-size:0.7rem;text-transform:uppercase}
.btn-ghost{background:none;border:1px solid var(--border);color:var(--text);padding:6px 12px;border-radius:6px;cursor:pointer;font-size:0.8rem}
.btn-ghost:hover{border-color:var(--primary);color:var(--primary)}
.dash-body{display:flex;height:calc(100vh - 50px)}
.sidebar{width:260px;background:var(--surface);border-right:1px solid var(--border);overflow-y:auto;flex-shrink:0;padding:12px}
.sidebar h3{font-size:0.75rem;color:var(--text-dim);text-transform:uppercase;padding:8px 12px;letter-spacing:0.5px}
.tenant-item{padding:10px 12px;border-radius:8px;cursor:pointer;font-size:0.85rem;margin-bottom:2px;display:flex;align-items:center;gap:8px;transition:background 0.15s}
.tenant-item:hover{background:var(--surface2)}
.tenant-item.active{background:var(--primary);color:#fff}
.tenant-item .dot{width:8px;height:8px;border-radius:50%;flex-shrink:0}
.tenant-item .dot.running{background:var(--accent)}
.tenant-item .dot.stopped{background:var(--text-dim)}
.content{flex:1;overflow-y:auto;padding:24px}
.stat-grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px;margin-bottom:24px}
.stat-card{background:var(--surface);border:1px solid var(--border);border-radius:10px;padding:16px}
.stat-card .label{font-size:0.75rem;color:var(--text-dim);margin-bottom:4px}
.stat-card .value{font-size:1.4rem;font-weight:700}
.members-table{width:100%;border-collapse:collapse;font-size:0.85rem}
.members-table th,.members-table td{padding:10px 12px;text-align:left;border-bottom:1px solid var(--border)}
.members-table th{color:var(--text-dim);font-weight:600;font-size:0.75rem;text-transform:uppercase}
.members-table tr:hover{background:var(--surface2)}
.role-badge{padding:2px 8px;border-radius:4px;font-size:0.7rem;text-transform:uppercase}
.role-badge.admin{background:#6c63ff33;color:#6c63ff}
.role-badge.member{background:#00d4aa33;color:#00d4aa}
@media(max-width:768px){.sidebar{display:none}.stat-grid{grid-template-columns:1fr}}
</style>
</head>
<body>
<!-- Login -->
<div class="login-container" id="loginView">
  <div class="login-card">
    <h1>🔐 Portal Login</h1>
    <p>Sign in with your member credentials to access your tenants.</p>
    <div class="form-group">
      <label>Email</label>
      <input type="email" id="loginEmail" placeholder="your@email.com" autocomplete="email">
    </div>
    <div class="form-group">
      <label>Password</label>
      <input type="password" id="loginPass" placeholder="••••••••" autocomplete="current-password"
        onkeydown="if(event.key==='Enter')doLogin()">
    </div>
    <button class="btn-primary" id="loginBtn" onclick="doLogin()">Sign In</button>
    <div class="error-msg" id="loginError"></div>
  </div>
</div>

<!-- Dashboard -->
<div class="dashboard" id="dashView">
  <div class="dash-header">
    <h1>🏢 OpenFang Portal</h1>
    <div class="user-info">
      <span id="userName"></span>
      <span class="badge" id="userRole"></span>
      <button class="btn-ghost" onclick="doLogout()">Logout</button>
    </div>
  </div>
  <div class="dash-body">
    <div class="sidebar">
      <h3>Tenants</h3>
      <div id="tenantList"></div>
    </div>
    <div class="content" id="contentArea">
      <div style="text-align:center;padding:60px;color:var(--text-dim)">
        <h2 style="margin-bottom:8px">👈 Select a tenant</h2>
        <p>Choose a tenant from the sidebar to view details.</p>
      </div>
    </div>
  </div>
</div>

<script>
let session=null;
let tenants=[];
let currentTenant=null;

function api(method,path,body){
  const opts={method,headers:{'Content-Type':'application/json'}};
  if(session)opts.headers['Authorization']='Bearer '+session.token;
  if(body)opts.body=JSON.stringify(body);
  return fetch(path,opts).then(r=>r.json());
}

async function doLogin(){
  const email=document.getElementById('loginEmail').value.trim();
  const pass=document.getElementById('loginPass').value;
  const err=document.getElementById('loginError');
  err.style.display='none';
  if(!email||!pass){err.textContent='Please fill in all fields';err.style.display='block';return}
  document.getElementById('loginBtn').disabled=true;
  try{
    const data=await api('POST','/api/portal/login',{email,password:pass});
    if(data.error){err.textContent=data.error;err.style.display='block';return}
    session=data;
    localStorage.setItem('portal_session',JSON.stringify(data));
    showDashboard();
  }catch(e){err.textContent='Connection error';err.style.display='block'}
  finally{document.getElementById('loginBtn').disabled=false}
}

function doLogout(){
  session=null;localStorage.removeItem('portal_session');
  document.getElementById('loginView').style.display='flex';
  document.getElementById('dashView').style.display='none';
}

async function showDashboard(){
  document.getElementById('loginView').style.display='none';
  document.getElementById('dashView').style.display='block';
  document.getElementById('userName').textContent=session.display_name||session.email;
  document.getElementById('userRole').textContent=session.role;
  await loadTenants();
}

async function loadTenants(){
  const data=await api('GET','/api/portal/tenants');
  tenants=data.tenants||[];
  const list=document.getElementById('tenantList');
  list.innerHTML=tenants.map(t=>`
    <div class="tenant-item" onclick="selectTenant('${t.id}')">
      <span class="dot ${t.status==='running'?'running':'stopped'}"></span>
      <span>${t.name}</span>
    </div>
  `).join('');
}

async function selectTenant(id){
  const data=await api('GET','/api/portal/tenants/'+id);
  if(data.error){alert(data.error);return}
  currentTenant=data;
  document.querySelectorAll('.tenant-item').forEach(el=>{
    el.classList.toggle('active',el.onclick.toString().includes(id))
  });
  renderTenantDetail(data);
}

function renderTenantDetail(t){
  const c=document.getElementById('contentArea');
  const membersHtml=t.members?t.members.map(m=>`
    <tr>
      <td>${m.display_name||m.email}</td>
      <td>${m.email}</td>
      <td><span class="role-badge ${m.role}">${m.role}</span></td>
      <td>${m.has_password?'✅':'❌'}</td>
      <td>${m.last_login||'Never'}</td>
    </tr>
  `).join(''):'';

  c.innerHTML=`
    <h2 style="margin-bottom:4px">${t.name}</h2>
    <p style="color:var(--text-dim);font-size:0.85rem;margin-bottom:20px">${t.slug} · ${t.plan}</p>
    <div class="stat-grid">
      <div class="stat-card"><div class="label">Provider</div><div class="value" style="font-size:1rem">${t.provider}</div></div>
      <div class="stat-card"><div class="label">Model</div><div class="value" style="font-size:1rem">${t.model}</div></div>
      <div class="stat-card"><div class="label">Messages Today</div><div class="value">${t.messages_today}</div></div>
      <div class="stat-card"><div class="label">Daily Limit</div><div class="value">${t.max_messages_per_day>=4294967295?'∞':t.max_messages_per_day}</div></div>
      <div class="stat-card"><div class="label">Channels</div><div class="value">${(t.channels||[]).length}</div></div>
      <div class="stat-card"><div class="label">Members</div><div class="value">${(t.members||[]).length}</div></div>
    </div>
    <h3 style="margin-bottom:12px">Members</h3>
    <table class="members-table">
      <thead><tr><th>Name</th><th>Email</th><th>Role</th><th>Password</th><th>Last Login</th></tr></thead>
      <tbody>${membersHtml}</tbody>
    </table>
  `;
}

// Auto-login from stored session
(function(){
  const stored=localStorage.getItem('portal_session');
  if(stored){
    try{session=JSON.parse(stored);showDashboard()}
    catch(e){localStorage.removeItem('portal_session')}
  }
})();
</script>
</body></html>"##;
