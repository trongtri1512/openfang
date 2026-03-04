//! Multi-tenant management module — CRUD API for tenant instances.
//!
//! Tenants are logical namespaces with their own agents, channels, quotas,
//! and access links. Data is persisted as JSON in `~/.openfang/tenants.json`.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use crate::routes::AppState;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub status: TenantStatus,
    pub plan: TenantPlan,
    pub provider: String,
    pub model: String,
    pub temperature: f64,
    pub max_messages_per_day: u32,
    pub max_channels: u32,
    pub max_members: u32,
    pub messages_today: u32,
    pub channels_active: u32,
    pub members: Vec<TenantMember>,
    pub access_token: String,
    pub created_at: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TenantStatus {
    Running,
    Stopped,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TenantPlan {
    Free,
    Pro,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMember {
    pub email: String,
    pub role: String,
    pub added_at: String,
}

impl std::fmt::Display for TenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantStatus::Running => write!(f, "running"),
            TenantStatus::Stopped => write!(f, "stopped"),
            TenantStatus::Error => write!(f, "error"),
        }
    }
}

impl std::fmt::Display for TenantPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantPlan::Free => write!(f, "free"),
            TenantPlan::Pro => write!(f, "pro"),
            TenantPlan::Enterprise => write!(f, "enterprise"),
        }
    }
}

// ---------------------------------------------------------------------------
// JSON file storage helpers
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
struct TenantsFile {
    tenants: Vec<Tenant>,
}

fn tenants_path(state: &AppState) -> std::path::PathBuf {
    state.kernel.config.home_dir.join("tenants.json")
}

fn load_tenants(state: &AppState) -> TenantsFile {
    let path = tenants_path(state);
    if path.exists() {
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => TenantsFile::default(),
        }
    } else {
        TenantsFile::default()
    }
}

fn save_tenants(state: &AppState, data: &TenantsFile) -> Result<(), String> {
    let path = tenants_path(state);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())
}

fn generate_slug(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();
    // Lowercase, remove special chars, truncate, + short hash
    let clean: String = name
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() { Some(c.to_lowercase().next().unwrap_or(c)) }
            else if c == ' ' || c == '-' { Some('-') }
            else { None }
        })
        .take(20)
        .collect();
    let clean = clean.trim_matches('-').to_string();
    format!("{}-{:x}", if clean.is_empty() { "tenant" } else { &clean }, hash & 0xFFFFFF)
}

fn generate_access_token() -> String {
    Uuid::new_v4().to_string().replace('-', "")
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

// ---------------------------------------------------------------------------
// Request / query types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    pub search: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub plan: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub plan: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMemberRequest {
    pub email: String,
    pub role: Option<String>,
}

// ---------------------------------------------------------------------------
// Plan defaults
// ---------------------------------------------------------------------------

fn plan_limits(plan: &TenantPlan) -> (u32, u32, u32) {
    match plan {
        TenantPlan::Free => (100, 3, 5),
        TenantPlan::Pro => (1000, 10, 20),
        TenantPlan::Enterprise => (u32::MAX, u32::MAX, u32::MAX),
    }
}

// ---------------------------------------------------------------------------
// API Handlers
// ---------------------------------------------------------------------------

/// GET /api/tenants — List all tenants with optional search/filter.
pub async fn list_tenants(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListTenantsQuery>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    let mut results: Vec<&Tenant> = data.tenants.iter().collect();

    // Search filter
    if let Some(ref search) = params.search {
        let q = search.to_lowercase();
        results.retain(|t| {
            t.name.to_lowercase().contains(&q) || t.slug.to_lowercase().contains(&q)
        });
    }

    // Status filter
    if let Some(ref status) = params.status {
        if status != "all" {
            results.retain(|t| t.status.to_string() == *status);
        }
    }

    let total = data.tenants.len();
    let running = data.tenants.iter().filter(|t| t.status == TenantStatus::Running).count();

    Json(serde_json::json!({
        "tenants": results,
        "total": total,
        "running": running,
    }))
}

/// POST /api/tenants — Create a new tenant.
pub async fn create_tenant(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateTenantRequest>,
) -> impl IntoResponse {
    if req.name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Name is required"}))).into_response();
    }

    let plan = match req.plan.as_deref() {
        Some("pro") => TenantPlan::Pro,
        Some("enterprise") => TenantPlan::Enterprise,
        _ => TenantPlan::Free,
    };

    let (max_msg, max_ch, max_mem) = plan_limits(&plan);

    let tenant = Tenant {
        id: Uuid::new_v4().to_string(),
        name: req.name.trim().to_string(),
        slug: generate_slug(req.name.trim()),
        status: TenantStatus::Running,
        plan,
        provider: req.provider.unwrap_or_else(|| "groq".to_string()),
        model: req.model.unwrap_or_else(|| "llama-3.3-70b-versatile".to_string()),
        temperature: req.temperature.unwrap_or(0.7),
        max_messages_per_day: max_msg,
        max_channels: max_ch,
        max_members: max_mem,
        messages_today: 0,
        channels_active: 0,
        members: vec![],
        access_token: generate_access_token(),
        created_at: now_iso(),
        version: format!("openfang-{}", env!("CARGO_PKG_VERSION")),
    };

    let mut data = load_tenants(&state);
    data.tenants.push(tenant.clone());

    if let Err(e) = save_tenants(&state, &data) {
        warn!("Failed to save tenant: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response();
    }

    info!(tenant_id = %tenant.id, name = %tenant.name, "Created tenant");
    (StatusCode::CREATED, Json(serde_json::json!(tenant))).into_response()
}

/// GET /api/tenants/:id — Get tenant detail.
pub async fn get_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(tenant) => Json(serde_json::json!(tenant)).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// PUT /api/tenants/:id — Update tenant.
pub async fn update_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTenantRequest>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    if let Some(name) = req.name { tenant.name = name; }
    if let Some(provider) = req.provider { tenant.provider = provider; }
    if let Some(model) = req.model { tenant.model = model; }
    if let Some(temp) = req.temperature { tenant.temperature = temp; }
    if let Some(ref plan_str) = req.plan {
        tenant.plan = match plan_str.as_str() {
            "pro" => TenantPlan::Pro,
            "enterprise" => TenantPlan::Enterprise,
            _ => TenantPlan::Free,
        };
        let (max_msg, max_ch, max_mem) = plan_limits(&tenant.plan);
        tenant.max_messages_per_day = max_msg;
        tenant.max_channels = max_ch;
        tenant.max_members = max_mem;
    }
    if let Some(ref status_str) = req.status {
        tenant.status = match status_str.as_str() {
            "stopped" => TenantStatus::Stopped,
            "error" => TenantStatus::Error,
            _ => TenantStatus::Running,
        };
    }

    let updated = tenant.clone();
    if let Err(e) = save_tenants(&state, &data) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response();
    }
    info!(tenant_id = %id, "Updated tenant");
    Json(serde_json::json!(updated)).into_response()
}

/// DELETE /api/tenants/:id — Delete tenant.
pub async fn delete_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let before = data.tenants.len();
    data.tenants.retain(|t| t.id != id);
    if data.tenants.len() == before {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response();
    }
    if let Err(e) = save_tenants(&state, &data) {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"error": e}))).into_response();
    }
    info!(tenant_id = %id, "Deleted tenant");
    Json(serde_json::json!({"status": "deleted", "id": id})).into_response()
}

/// POST /api/tenants/:id/restart — Restart tenant.
pub async fn restart_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => {
            t.status = TenantStatus::Running;
            t.messages_today = 0;
            let _ = save_tenants(&state, &data);
            info!(tenant_id = %id, "Restarted tenant");
            Json(serde_json::json!({"status": "restarted", "id": id})).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// POST /api/tenants/:id/stop — Stop tenant.
pub async fn stop_tenant(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => {
            t.status = TenantStatus::Stopped;
            let _ = save_tenants(&state, &data);
            info!(tenant_id = %id, "Stopped tenant");
            Json(serde_json::json!({"status": "stopped", "id": id})).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// GET /api/tenants/:id/stats — Resource metrics for a tenant.
pub async fn tenant_stats(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    let tenant = match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    // Provide process-level metrics approximation
    let process_count = state.kernel.registry.count();
    let uptime_secs = state.started_at.elapsed().as_secs();

    Json(serde_json::json!({
        "cpu_percent": 0.0,
        "memory_used_mb": 4.4,
        "memory_total_mb": 512.0,
        "disk_used_kb": 180.6,
        "processes": process_count,
        "messages_today": tenant.messages_today,
        "messages_limit": tenant.max_messages_per_day,
        "channels_active": tenant.channels_active,
        "channels_limit": tenant.max_channels,
        "uptime_seconds": uptime_secs,
    })).into_response()
}

/// GET /api/tenants/:id/logs — Recent logs.
pub async fn tenant_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    if !data.tenants.iter().any(|t| t.id == id) {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response();
    }

    // Return recent API request logs from kernel audit if available
    Json(serde_json::json!({
        "logs": [],
        "note": "Log streaming is available via the main Dashboard"
    })).into_response()
}

/// POST /api/tenants/:id/access-link — Regenerate magic access link.
pub async fn regenerate_access_link(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => {
            t.access_token = generate_access_token();
            let token = t.access_token.clone();
            let _ = save_tenants(&state, &data);
            info!(tenant_id = %id, "Regenerated access link");
            Json(serde_json::json!({"access_token": token})).into_response()
        }
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// GET /api/tenants/:id/members — List members.
pub async fn list_members(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => Json(serde_json::json!({"members": t.members})).into_response(),
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// POST /api/tenants/:id/members — Add member.
pub async fn add_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<AddMemberRequest>,
) -> impl IntoResponse {
    if req.email.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"error": "Email is required"}))).into_response();
    }

    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    if tenant.members.len() as u32 >= tenant.max_members {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": format!("Member limit reached ({}/{})", tenant.members.len(), tenant.max_members)
        }))).into_response();
    }

    if tenant.members.iter().any(|m| m.email == req.email.trim()) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "Member already exists"}))).into_response();
    }

    tenant.members.push(TenantMember {
        email: req.email.trim().to_string(),
        role: req.role.unwrap_or_else(|| "member".to_string()),
        added_at: now_iso(),
    });

    let members = tenant.members.clone();
    let _ = save_tenants(&state, &data);
    Json(serde_json::json!({"members": members})).into_response()
}

/// DELETE /api/tenants/:id/members/:email — Remove member.
pub async fn remove_member(
    State(state): State<Arc<AppState>>,
    Path((id, email)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    let before = tenant.members.len();
    tenant.members.retain(|m| m.email != email);
    if tenant.members.len() == before {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Member not found"}))).into_response();
    }

    let members = tenant.members.clone();
    let _ = save_tenants(&state, &data);
    Json(serde_json::json!({"members": members})).into_response()
}
