//! Data models for BizClaw Portal — fully independent from OpenFang core.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Core data models
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
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub channels: Vec<TenantChannel>,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(default)]
    pub hands: Vec<String>,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub webhook_url: Option<String>,
    #[serde(default)]
    pub agent_name: Option<String>,
    #[serde(default)]
    pub archetype: Option<String>,
    #[serde(default)]
    pub vibe: Option<String>,
    #[serde(default)]
    pub greeting_style: Option<String>,
    #[serde(default)]
    pub tool_profile: Option<String>,
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

/// An independent channel instance managed by Portal (not OpenFang).
/// Allows multiple instances of the same channel type (e.g., 3 Telegram bots).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInstance {
    pub id: String,
    pub tenant_id: String,
    pub channel_type: String,
    pub display_name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
    pub webhook_path: String,
    pub created_at: String,
    #[serde(default)]
    pub last_message_at: Option<String>,
    #[serde(default)]
    pub message_count: u64,
    #[serde(default)]
    pub status: ChannelInstanceStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ChannelInstanceStatus {
    #[default]
    Pending,
    Active,
    Error,
    Disabled,
}

impl std::fmt::Display for ChannelInstanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelInstanceStatus::Pending => write!(f, "pending"),
            ChannelInstanceStatus::Active => write!(f, "active"),
            ChannelInstanceStatus::Error => write!(f, "error"),
            ChannelInstanceStatus::Disabled => write!(f, "disabled"),
        }
    }
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
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub last_login: Option<String>,
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
// Global Portal Users & Service Plans
// ---------------------------------------------------------------------------

/// A global portal user (independent of any tenant).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalUser {
    pub email: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub password_hash: Option<String>,
    pub role: String,
    #[serde(default)]
    pub plan_id: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub last_login: Option<String>,
    #[serde(default = "default_max_tenants")]
    pub max_tenants: u32,
}

fn default_max_tenants() -> u32 { 3 }

/// A configurable service plan with quotas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePlan {
    pub id: String,
    pub name: String,
    pub max_messages_per_day: u32,
    pub max_channels: u32,
    pub max_members: u32,
    pub max_tenants: u32,
    #[serde(default)]
    pub price_label: String,
    #[serde(default)]
    pub is_default: bool,
}

// ---------------------------------------------------------------------------
// Independent Portal Feature Models
// ---------------------------------------------------------------------------

/// Knowledge Base document for RAG.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDoc {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub chunks: u32,
    #[serde(default)]
    pub size_bytes: u64,
    pub created_at: String,
}

/// Tool configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalTool {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub builtin: bool,
}

/// Skill from Skills Market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalSkill {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub installed: bool,
    #[serde(default)]
    pub builtin: bool,
}

/// Agent template for Gallery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub system_prompt: String,
}

/// Agent-to-agent delegation rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub id: String,
    #[serde(default)]
    pub from_agent: String,
    #[serde(default)]
    pub to_agent: String,
    #[serde(default)]
    pub link_type: String,
    #[serde(default)]
    pub enabled: bool,
}

/// Kanban task/card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KanbanTask {
    pub id: String,
    #[serde(default)]
    pub tenant_id: String,
    pub title: String,
    #[serde(default)]
    pub agent: String,
    #[serde(default)]
    pub status: String,      // inbox, in_progress, review, done
    pub created_at: String,
}

/// LLM call trace log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    pub id: String,
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub prompt_tokens: u32,
    #[serde(default)]
    pub completion_tokens: u32,
    #[serde(default)]
    pub latency_ms: u32,
    #[serde(default)]
    pub cost: f64,
    pub created_at: String,
}

/// System activity event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub id: String,
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub event_type: String,
    #[serde(default)]
    pub message: String,
    pub created_at: String,
}

/// Portal-managed API key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalApiKey {
    pub id: String,
    #[serde(default)]
    pub tenant_id: String,
    pub name: String,
    #[serde(default)]
    pub key_hash: String,
    #[serde(default)]
    pub key_prefix: String,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Aggregate data file
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PortalData {
    #[serde(default)]
    pub users: Vec<PortalUser>,
    #[serde(default)]
    pub plans: Vec<ServicePlan>,
    #[serde(default)]
    pub tenants: Vec<Tenant>,
    #[serde(default)]
    pub channel_instances: Vec<ChannelInstance>,
    // Independent feature data
    #[serde(default)]
    pub knowledge_docs: Vec<KnowledgeDoc>,
    #[serde(default)]
    pub tools: Vec<PortalTool>,
    #[serde(default)]
    pub skills: Vec<PortalSkill>,
    #[serde(default)]
    pub gallery: Vec<AgentTemplate>,
    #[serde(default)]
    pub delegations: Vec<Delegation>,
    #[serde(default)]
    pub kanban_tasks: Vec<KanbanTask>,
    #[serde(default)]
    pub traces: Vec<LlmTrace>,
    #[serde(default)]
    pub activity: Vec<ActivityEvent>,
    #[serde(default)]
    pub api_keys: Vec<PortalApiKey>,
}

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionPayload {
    pub email: String,
    pub role: String,
    pub tenant_ids: Vec<String>,
    pub exp: i64,
}

// ---------------------------------------------------------------------------
// Request types
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

#[derive(Debug, Deserialize)]
pub struct PortalUpdateConfigRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PortalAddChannelRequest {
    pub channel_type: String,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PortalRemoveChannelRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct PortalUpdateChannelConfigRequest {
    pub channel_name: String,
    #[serde(default)]
    pub channel_instance_id: Option<String>,
    pub config: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct PortalUpdateAgentRequest {
    pub system_prompt: Option<String>,
    pub skills: Option<Vec<String>>,
    pub hands: Option<Vec<String>>,
    pub language: Option<String>,
    pub webhook_url: Option<String>,
    pub agent_name: Option<String>,
    pub archetype: Option<String>,
    pub vibe: Option<String>,
    pub greeting_style: Option<String>,
    pub tool_profile: Option<String>,
    #[serde(default)]
    pub deploy: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct PortalCreateUserRequest {
    pub email: String,
    pub password: Option<String>,
    pub role: Option<String>,
    pub display_name: Option<String>,
    pub plan_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PortalUpdateUserRequest {
    pub role: Option<String>,
    pub plan_id: Option<String>,
    pub password: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PortalCreatePlanRequest {
    pub name: String,
    pub max_messages_per_day: u32,
    pub max_channels: u32,
    pub max_members: u32,
    pub max_tenants: u32,
    #[serde(default)]
    pub price_label: String,
}

#[derive(Debug, Deserialize)]
pub struct PortalCreateMyTenantRequest {
    pub name: String,
    pub provider: Option<String>,
    pub model: Option<String>,
}

// ---------------------------------------------------------------------------
// Channel Instance request types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct CreateChannelInstanceRequest {
    pub tenant_id: String,
    pub channel_type: String,
    pub display_name: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelInstanceRequest {
    pub display_name: Option<String>,
    pub enabled: Option<bool>,
    pub config: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

pub fn seed_defaults(data: &mut PortalData) -> bool {
    if !data.plans.is_empty() { return false; }
    data.plans = vec![
        ServicePlan { id: "free".into(), name: "Free".into(), max_messages_per_day: 100, max_channels: 3, max_members: 5, max_tenants: 2, price_label: "Free".into(), is_default: true },
        ServicePlan { id: "pro".into(), name: "Pro".into(), max_messages_per_day: 1000, max_channels: 10, max_members: 20, max_tenants: 10, price_label: "$29/mo".into(), is_default: false },
        ServicePlan { id: "enterprise".into(), name: "Enterprise".into(), max_messages_per_day: u32::MAX, max_channels: u32::MAX, max_members: u32::MAX, max_tenants: u32::MAX, price_label: "Contact us".into(), is_default: false },
    ];
    true
}

pub fn generate_slug(name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    let hash = hasher.finish();
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

pub fn generate_access_token() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

pub fn hash_password(password: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    "openfang_salt_v1".hash(&mut hasher);
    password.hash(&mut hasher);
    let h1 = hasher.finish();
    let mut hasher2 = DefaultHasher::new();
    h1.hash(&mut hasher2);
    password.hash(&mut hasher2);
    let h2 = hasher2.finish();
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(format!("{:016x}{:016x}", h1, h2))
}

pub fn verify_password(password: &str, stored_hash: &str) -> bool {
    hash_password(password) == stored_hash
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
