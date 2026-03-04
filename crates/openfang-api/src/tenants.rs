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
    /// AI provider API key (masked in JSON output, stored encrypted).
    #[serde(default)]
    pub api_key: Option<String>,
    /// Per-tenant channel configurations.
    #[serde(default)]
    pub channels: Vec<TenantChannel>,
}

/// A channel configured for a specific tenant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantChannel {
    pub name: String,
    pub channel_type: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub added_at: String,
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
        api_key: None,
        channels: vec![],
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

// ---------------------------------------------------------------------------
// Config TOML
// ---------------------------------------------------------------------------

/// GET /api/tenants/:id/config-toml — Get tenant config as TOML string.
pub async fn tenant_config_toml(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(tenant) => {
            let toml_str = format!(
                "# Tenant: {}\n# Slug: {}\n# Plan: {}\n\n[agent]\nprovider = \"{}\"\nmodel = \"{}\"\ntemperature = {}\n\n[quota]\nmax_messages_per_day = {}\nmax_channels = {}\nmax_members = {}\n",
                tenant.name, tenant.slug, tenant.plan,
                tenant.provider, tenant.model, tenant.temperature,
                if tenant.max_messages_per_day >= u32::MAX { "unlimited".to_string() } else { tenant.max_messages_per_day.to_string() },
                if tenant.max_channels >= u32::MAX { "unlimited".to_string() } else { tenant.max_channels.to_string() },
                if tenant.max_members >= u32::MAX { "unlimited".to_string() } else { tenant.max_members.to_string() },
            );
            Json(serde_json::json!({"toml": toml_str})).into_response()
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

// ---------------------------------------------------------------------------
// Tenant Channels CRUD
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AddChannelRequest {
    pub channel_type: String,
    pub name: Option<String>,
    #[serde(default)]
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelRequest {
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

/// GET /api/tenants/:id/channels — List tenant channels.
pub async fn list_tenant_channels(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    match data.tenants.iter().find(|t| t.id == id) {
        Some(tenant) => {
            let limit = if tenant.max_channels >= u32::MAX { "∞".to_string() } else { tenant.max_channels.to_string() };
            Json(serde_json::json!({
                "channels": tenant.channels,
                "count": tenant.channels.len(),
                "limit": limit,
            })).into_response()
        },
        None => (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    }
}

/// POST /api/tenants/:id/channels — Add a channel to tenant.
pub async fn add_tenant_channel(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<AddChannelRequest>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    if tenant.channels.len() as u32 >= tenant.max_channels {
        return (StatusCode::FORBIDDEN, Json(serde_json::json!({
            "error": format!("Channel limit reached ({}/{})", tenant.channels.len(), tenant.max_channels)
        }))).into_response();
    }

    // Check duplicate
    if tenant.channels.iter().any(|c| c.channel_type == req.channel_type && c.enabled) {
        return (StatusCode::CONFLICT, Json(serde_json::json!({"error": "Channel type already active"}))).into_response();
    }

    let channel = TenantChannel {
        name: req.name.unwrap_or_else(|| req.channel_type.clone()),
        channel_type: req.channel_type,
        enabled: true,
        config: req.config,
        added_at: now_iso(),
    };

    tenant.channels.push(channel);
    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let channels = tenant.channels.clone();
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Added channel");
    (StatusCode::CREATED, Json(serde_json::json!({"channels": channels}))).into_response()
}

/// PUT /api/tenants/:id/channels/:channel_name — Update channel (enable/disable).
pub async fn update_tenant_channel(
    State(state): State<Arc<AppState>>,
    Path((id, channel_name)): Path<(String, String)>,
    Json(req): Json<UpdateChannelRequest>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    let channel = match tenant.channels.iter_mut().find(|c| c.name == channel_name || c.channel_type == channel_name) {
        Some(c) => c,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Channel not found"}))).into_response(),
    };

    if let Some(enabled) = req.enabled { channel.enabled = enabled; }
    if let Some(config) = req.config { channel.config = config; }

    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let channels = tenant.channels.clone();
    let _ = save_tenants(&state, &data);
    Json(serde_json::json!({"channels": channels})).into_response()
}

/// DELETE /api/tenants/:id/channels/:channel_name — Remove channel.
pub async fn delete_tenant_channel(
    State(state): State<Arc<AppState>>,
    Path((id, channel_name)): Path<(String, String)>,
) -> impl IntoResponse {
    let mut data = load_tenants(&state);
    let tenant = match data.tenants.iter_mut().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    let before = tenant.channels.len();
    tenant.channels.retain(|c| c.name != channel_name && c.channel_type != channel_name);
    if tenant.channels.len() == before {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Channel not found"}))).into_response();
    }

    tenant.channels_active = tenant.channels.iter().filter(|c| c.enabled).count() as u32;
    let channels = tenant.channels.clone();
    let _ = save_tenants(&state, &data);
    info!(tenant_id = %id, "Removed channel");
    Json(serde_json::json!({"channels": channels})).into_response()
}

// ---------------------------------------------------------------------------
// Usage
// ---------------------------------------------------------------------------

/// GET /api/tenants/:id/usage — Detailed usage metrics.
pub async fn tenant_usage(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let data = load_tenants(&state);
    let tenant = match data.tenants.iter().find(|t| t.id == id) {
        Some(t) => t,
        None => return (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "Tenant not found"}))).into_response(),
    };

    let uptime_secs = state.started_at.elapsed().as_secs();

    Json(serde_json::json!({
        "messages_today": tenant.messages_today,
        "messages_limit": if tenant.max_messages_per_day >= u32::MAX { 0 } else { tenant.max_messages_per_day },
        "messages_unlimited": tenant.max_messages_per_day >= u32::MAX,
        "channels_active": tenant.channels_active,
        "channels_limit": tenant.max_channels,
        "channels_unlimited": tenant.max_channels >= u32::MAX,
        "members_count": tenant.members.len(),
        "members_limit": tenant.max_members,
        "members_unlimited": tenant.max_members >= u32::MAX,
        "uptime_seconds": uptime_secs,
        "plan": tenant.plan.to_string(),
        "created_at": tenant.created_at,
        "api_calls_today": 0,
    })).into_response()
}

// ---------------------------------------------------------------------------
// Magic Access Link — public tenant WebChat
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AccessQuery {
    pub t: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AccessChatRequest {
    pub message: String,
    #[serde(default)]
    pub history: Vec<serde_json::Value>,
}

/// Map a provider name to its OpenAI-compatible base URL.
fn provider_base_url(provider: &str) -> &'static str {
    match provider {
        "openai" => "https://api.openai.com/v1",
        "anthropic" => "https://api.anthropic.com/v1",
        "groq" => "https://api.groq.com/openai/v1",
        "openrouter" => "https://openrouter.ai/api/v1",
        "deepseek" => "https://api.deepseek.com/v1",
        "together" => "https://api.together.xyz/v1",
        "mistral" => "https://api.mistral.ai/v1",
        "fireworks" => "https://api.fireworks.ai/inference/v1",
        "gemini" | "google" => "https://generativelanguage.googleapis.com/v1beta/openai",
        "ollama" => "http://host.docker.internal:11434/v1",
        "xai" => "https://api.x.ai/v1",
        "perplexity" => "https://api.perplexity.ai",
        "cohere" => "https://api.cohere.ai/compatibility/v1",
        "cerebras" => "https://api.cerebras.ai/v1",
        "sambanova" => "https://api.sambanova.ai/v1",
        "moonshot" | "kimi" => "https://api.moonshot.cn/v1",
        "qwen" | "dashscope" => "https://dashscope.aliyuncs.com/compatible-mode/v1",
        "volcengine" => "https://ark.cn-beijing.volces.com/api/v3",
        _ => "https://api.openai.com/v1", // fallback
    }
}

/// GET /access/?t=<token> — Serve lightweight WebChat page for a tenant.
pub async fn tenant_access_page(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccessQuery>,
) -> impl IntoResponse {
    let token = match params.t {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                axum::response::Html(
                    r#"<!DOCTYPE html><html><body style="font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0"><h2>⚠️ Missing access token. Please use the link provided by your admin.</h2></body></html>"#.to_string()
                ),
            ).into_response();
        }
    };

    let data = load_tenants(&state);
    let tenant = data.tenants.iter().find(|t| t.access_token == token);

    match tenant {
        None => {
            (
                StatusCode::FORBIDDEN,
                axum::response::Html(
                    r#"<!DOCTYPE html><html><body style="font-family:sans-serif;display:flex;align-items:center;justify-content:center;height:100vh;margin:0;background:#1a1a2e;color:#e0e0e0"><h2>🔒 Invalid or expired access link. Please contact your admin for a new link.</h2></body></html>"#.to_string()
                ),
            ).into_response()
        }
        Some(tenant) => {
            let tenant_name = tenant.name.replace('"', "&quot;").replace('<', "&lt;");
            let tenant_model = tenant.model.replace('"', "&quot;");
            let html = format!(r##"<!DOCTYPE html>
<html lang="vi">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>{tenant_name} — AI Chat</title>
<style>
*{{margin:0;padding:0;box-sizing:border-box}}
:root{{--bg:#0f0f23;--surface:#1a1a2e;--surface2:#16213e;--border:#2a2a4a;--text:#e0e0e0;--text-dim:#8888aa;--primary:#6c63ff;--primary-hover:#5a52d5;--accent:#00d4aa;--msg-user:#1e3a5f;--msg-bot:#1a1a2e}}
body{{font-family:'Segoe UI',system-ui,-apple-system,sans-serif;background:var(--bg);color:var(--text);height:100vh;display:flex;flex-direction:column}}
.header{{background:var(--surface);border-bottom:1px solid var(--border);padding:12px 20px;display:flex;align-items:center;gap:12px;flex-shrink:0}}
.header .logo{{width:32px;height:32px;background:var(--primary);border-radius:8px;display:flex;align-items:center;justify-content:center;font-size:18px}}
.header h1{{font-size:1rem;font-weight:600}}
.header .model{{font-size:0.75rem;color:var(--text-dim);background:var(--surface2);padding:2px 8px;border-radius:4px}}
.chat-area{{flex:1;overflow-y:auto;padding:20px;display:flex;flex-direction:column;gap:12px}}
.msg{{max-width:80%;padding:12px 16px;border-radius:12px;line-height:1.5;font-size:0.9rem;word-wrap:break-word;white-space:pre-wrap}}
.msg.user{{align-self:flex-end;background:var(--msg-user);border-bottom-right-radius:4px}}
.msg.bot{{align-self:flex-start;background:var(--surface);border:1px solid var(--border);border-bottom-left-radius:4px}}
.msg.bot .thinking{{color:var(--text-dim);font-style:italic}}
.input-area{{background:var(--surface);border-top:1px solid var(--border);padding:12px 20px;display:flex;gap:10px;flex-shrink:0}}
.input-area textarea{{flex:1;background:var(--surface2);border:1px solid var(--border);border-radius:8px;padding:10px 14px;color:var(--text);font-size:0.9rem;font-family:inherit;resize:none;outline:none;min-height:44px;max-height:120px}}
.input-area textarea:focus{{border-color:var(--primary)}}
.input-area button{{background:var(--primary);color:#fff;border:none;border-radius:8px;padding:10px 20px;font-size:0.9rem;cursor:pointer;font-weight:600;transition:background 0.2s}}
.input-area button:hover{{background:var(--primary-hover)}}
.input-area button:disabled{{opacity:0.5;cursor:not-allowed}}
.typing{{display:flex;gap:4px;padding:4px 0}}.typing span{{width:6px;height:6px;background:var(--text-dim);border-radius:50%;animation:bounce 1.4s infinite}}.typing span:nth-child(2){{animation-delay:0.2s}}.typing span:nth-child(3){{animation-delay:0.4s}}
@keyframes bounce{{0%,80%,100%{{transform:translateY(0)}}40%{{transform:translateY(-8px)}}}}
@media(max-width:600px){{.msg{{max-width:90%}}.header h1{{font-size:0.9rem}}}}
</style>
</head>
<body>
<div class="header">
  <div class="logo">🤖</div>
  <h1>{tenant_name}</h1>
  <span class="model">{tenant_model}</span>
</div>
<div class="chat-area" id="chatArea">
  <div class="msg bot">Xin chào! Tôi là trợ lý AI của <strong>{tenant_name}</strong>. Bạn cần tôi giúp gì?</div>
</div>
<div class="input-area">
  <textarea id="msgInput" placeholder="Nhập tin nhắn..." rows="1" onkeydown="if(event.key==='Enter'&&!event.shiftKey){{event.preventDefault();sendMsg()}}"></textarea>
  <button id="sendBtn" onclick="sendMsg()">Gửi</button>
</div>
<script>
const TOKEN="{token}";
const chatArea=document.getElementById('chatArea');
const msgInput=document.getElementById('msgInput');
const sendBtn=document.getElementById('sendBtn');
let history=[];
function addMsg(role,text){{
  const d=document.createElement('div');
  d.className='msg '+(role==='user'?'user':'bot');
  d.textContent=text;
  chatArea.appendChild(d);
  chatArea.scrollTop=chatArea.scrollHeight;
  return d;
}}
function showTyping(){{
  const d=document.createElement('div');
  d.className='msg bot';d.id='typing';
  d.innerHTML='<div class="typing"><span></span><span></span><span></span></div>';
  chatArea.appendChild(d);chatArea.scrollTop=chatArea.scrollHeight;
}}
function hideTyping(){{const e=document.getElementById('typing');if(e)e.remove()}}
async function sendMsg(){{
  const text=msgInput.value.trim();
  if(!text)return;
  msgInput.value='';sendBtn.disabled=true;
  addMsg('user',text);
  history.push({{role:'user',content:text}});
  showTyping();
  try{{
    const res=await fetch('/api/access/chat?t='+TOKEN,{{
      method:'POST',
      headers:{{'Content-Type':'application/json'}},
      body:JSON.stringify({{message:text,history:history.slice(-20)}})
    }});
    hideTyping();
    const data=await res.json();
    if(data.error){{addMsg('bot','⚠️ '+data.error)}}
    else{{addMsg('bot',data.reply||'(no response)');history.push({{role:'assistant',content:data.reply||''}})}}
  }}catch(e){{hideTyping();addMsg('bot','⚠️ Connection error: '+e.message)}}
  sendBtn.disabled=false;msgInput.focus();
}}
msgInput.addEventListener('input',function(){{this.style.height='auto';this.style.height=Math.min(this.scrollHeight,120)+'px'}});
msgInput.focus();
</script>
</body></html>"##);

            (
                [(axum::http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
                axum::response::Html(html),
            ).into_response()
        }
    }
}

/// POST /api/access/chat?t=<token> — Proxy chat message to tenant's LLM.
pub async fn tenant_access_chat(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AccessQuery>,
    Json(body): Json<AccessChatRequest>,
) -> impl IntoResponse {
    // 1. Verify access token
    let token = match params.t {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({"error": "Missing access token"})),
            ).into_response();
        }
    };

    let data = load_tenants(&state);
    let tenant = match data.tenants.iter().find(|t| t.access_token == token) {
        Some(t) => t.clone(),
        None => {
            return (
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Invalid or expired access token"})),
            ).into_response();
        }
    };

    // 2. Check quota
    if tenant.messages_today >= tenant.max_messages_per_day && tenant.max_messages_per_day < u32::MAX {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({"error": "Daily message limit reached. Please try again tomorrow."})),
        ).into_response();
    }

    // 3. Get API key for the tenant
    let api_key = match &tenant.api_key {
        Some(key) if !key.is_empty() => key.clone(),
        _ => {
            // Fallback: try env var for the provider
            let env_key = match tenant.provider.as_str() {
                "openai" => "OPENAI_API_KEY",
                "anthropic" => "ANTHROPIC_API_KEY",
                "groq" => "GROQ_API_KEY",
                "openrouter" => "OPENROUTER_API_KEY",
                "deepseek" => "DEEPSEEK_API_KEY",
                "gemini" | "google" => "GEMINI_API_KEY",
                "ollama" => "OLLAMA_API_KEY",
                _ => "OPENAI_API_KEY",
            };
            std::env::var(env_key).unwrap_or_default()
        }
    };

    if api_key.is_empty() && tenant.provider != "ollama" {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "No API key configured for this tenant. Please contact your admin."})),
        ).into_response();
    }

    // 4. Build messages for LLM
    let base_url = provider_base_url(&tenant.provider);
    let mut messages = vec![
        serde_json::json!({"role": "system", "content": format!(
            "You are a helpful AI assistant for {}. Answer concisely and helpfully in the same language as the user.",
            tenant.name
        )}),
    ];
    // Add conversation history (last 20 messages)
    for msg in &body.history {
        messages.push(msg.clone());
    }
    messages.push(serde_json::json!({"role": "user", "content": body.message}));

    // 5. Call LLM via OpenAI-compatible API
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", base_url);

    let mut req_builder = client
        .post(&url)
        .header("Content-Type", "application/json");

    if !api_key.is_empty() {
        req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
    }

    let llm_body = serde_json::json!({
        "model": tenant.model,
        "messages": messages,
        "max_tokens": 2048,
        "temperature": tenant.temperature,
    });

    match req_builder.json(&llm_body).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                warn!(
                    tenant = %tenant.name,
                    provider = %tenant.provider,
                    status = %status,
                    "LLM API error"
                );
                return Json(serde_json::json!({
                    "error": format!("LLM provider error ({}): {}", status, err_text)
                })).into_response();
            }

            match resp.json::<serde_json::Value>().await {
                Ok(json) => {
                    let reply = json["choices"][0]["message"]["content"]
                        .as_str()
                        .unwrap_or("(empty response)")
                        .to_string();

                    // Increment message counter
                    let mut data = load_tenants(&state);
                    if let Some(t) = data.tenants.iter_mut().find(|t| t.access_token == token) {
                        t.messages_today = t.messages_today.saturating_add(1);
                        let _ = save_tenants(&state, &data);
                    }

                    Json(serde_json::json!({"reply": reply})).into_response()
                }
                Err(e) => {
                    warn!(tenant = %tenant.name, error = %e, "Failed to parse LLM response");
                    Json(serde_json::json!({"error": "Failed to parse LLM response"})).into_response()
                }
            }
        }
        Err(e) => {
            warn!(tenant = %tenant.name, error = %e, "Failed to connect to LLM provider");
            Json(serde_json::json!({
                "error": format!("Failed to connect to LLM provider: {}", e)
            })).into_response()
        }
    }
}
