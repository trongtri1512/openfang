//! BizClaw Portal — standalone HTTP server for tenant management.
//!
//! This is a completely independent service from OpenFang.
//! It communicates with OpenFang only via HTTP API proxies.
//!
//! # Environment Variables
//! - `PORTAL_LISTEN` — bind address (default: `0.0.0.0:4201`)
//! - `PORTAL_DATABASE_URL` — PostgreSQL connection string (optional, falls back to JSON file)
//! - `OPENFANG_API_URL` — OpenFang API base URL (default: `http://openfang:4200`)
//! - `PORTAL_ADMIN_KEY` — admin password for super-admin login
//! - `OPENFANG_API_KEY` — fallback admin key

mod models;
mod db;
mod handlers;
mod html;
mod channels;

use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,portal=debug".parse().unwrap()),
        )
        .init();

    let listen = std::env::var("PORTAL_LISTEN").unwrap_or_else(|_| "0.0.0.0:4201".into());

    info!("🚀 BizClaw Portal starting...");
    info!("   Listen: {listen}");

    // Create portal state
    let state = Arc::new(db::PortalState::new());

    // Initialize default data (seed plans if empty)
    db::init_data(&state);

    let openfang_url = state.openfang_api_url.clone();
    info!("   OpenFang API: {openfang_url}");
    info!("   Database: {}", if state.db_pool.is_some() { "PostgreSQL" } else { "JSON file" });

    // Build router
    let app = axum::Router::new()
        // Portal HTML pages
        .route("/", axum::routing::get(handlers::portal_page))
        .route("/{id}", axum::routing::get(handlers::portal_page_with_id))
        // Auth
        .route("/api/portal/login", axum::routing::post(handlers::portal_login))
        .route("/api/portal/me", axum::routing::get(handlers::portal_me))
        // Tenants
        .route("/api/portal/tenants", axum::routing::get(handlers::portal_tenants))
        .route(
            "/api/portal/tenants/{id}",
            axum::routing::get(handlers::portal_tenant_detail)
                .delete(handlers::portal_delete_tenant),
        )
        .route("/api/portal/tenants/{id}/config", axum::routing::put(handlers::portal_update_config))
        .route("/api/portal/tenants/{id}/restart", axum::routing::post(handlers::portal_restart))
        .route("/api/portal/tenants/{id}/stop", axum::routing::post(handlers::portal_stop))
        .route("/api/portal/tenants/{id}/clone", axum::routing::post(handlers::portal_clone_tenant))
        .route("/api/portal/tenants/{id}/conversations", axum::routing::get(handlers::portal_conversations))
        // Members
        .route("/api/portal/tenants/{id}/members", axum::routing::post(handlers::portal_add_member).delete(handlers::portal_remove_member))
        .route("/api/portal/tenants/{id}/members/role", axum::routing::put(handlers::portal_update_role))
        .route("/api/portal/tenants/{id}/members/password", axum::routing::put(handlers::portal_set_password))
        .route("/api/portal/members", axum::routing::get(handlers::portal_all_members))
        // Channels
        .route("/api/portal/tenants/{id}/channels", axum::routing::post(handlers::portal_add_channel).delete(handlers::portal_remove_channel))
        .route("/api/portal/tenants/{id}/channels/config", axum::routing::put(handlers::portal_update_channel_config))
        // Agent
        .route("/api/portal/tenants/{id}/agent", axum::routing::put(handlers::portal_update_agent))
        .route("/api/portal/tenants/{id}/chat", axum::routing::post(handlers::portal_chat))
        // Self-service
        .route("/api/portal/my/tenants", axum::routing::post(handlers::portal_create_my_tenant))
        // Users CRUD (admin)
        .route("/api/portal/users", axum::routing::get(handlers::portal_list_users).post(handlers::portal_create_user))
        .route("/api/portal/users/{email}", axum::routing::put(handlers::portal_update_user).delete(handlers::portal_delete_user))
        // Plans CRUD (admin)
        .route("/api/portal/plans", axum::routing::get(handlers::portal_list_plans).post(handlers::portal_create_plan))
        .route("/api/portal/plans/{id}", axum::routing::put(handlers::portal_update_plan).delete(handlers::portal_delete_plan))
        // Automation: Workflows (proxy to OpenFang)
        .route("/api/portal/workflows", axum::routing::get(handlers::portal_list_workflows).post(handlers::portal_create_workflow))
        .route("/api/portal/workflows/{id}", axum::routing::delete(handlers::portal_delete_workflow))
        .route("/api/portal/workflows/{id}/run", axum::routing::post(handlers::portal_run_workflow))
        .route("/api/portal/workflows/{id}/runs", axum::routing::get(handlers::portal_workflow_runs))
        // Automation: Scheduler / Cron Jobs (proxy to OpenFang)
        .route("/api/portal/scheduler", axum::routing::get(handlers::portal_list_cron_jobs).post(handlers::portal_create_cron_job))
        .route("/api/portal/scheduler/{id}", axum::routing::put(handlers::portal_toggle_cron_job).delete(handlers::portal_delete_cron_job))
        // System API proxies (calls OpenFang via HTTP)
        .route("/api/portal/system/agents", axum::routing::get(handlers::portal_system_agents))
        .route("/api/portal/system/channels", axum::routing::get(handlers::portal_system_channels))
        .route("/api/portal/system/providers", axum::routing::get(handlers::portal_system_providers))
        .route("/api/portal/system/models", axum::routing::get(handlers::portal_system_models))
        .route("/api/portal/system/skills", axum::routing::get(handlers::portal_system_skills))
        .route("/api/portal/system/hands", axum::routing::get(handlers::portal_system_hands))
        // Write proxies → push config to OpenFang
        .route("/api/portal/system/channels/{name}/configure", axum::routing::post(handlers::portal_system_channel_configure).delete(handlers::portal_system_channel_remove))
        .route("/api/portal/system/channels/reload", axum::routing::post(handlers::portal_system_channels_reload))
        .route("/api/portal/system/skills/install", axum::routing::post(handlers::portal_system_skill_install))
        .route("/api/portal/system/skills/uninstall", axum::routing::post(handlers::portal_system_skill_uninstall))
        .route("/api/portal/system/hands/{hand_id}/activate", axum::routing::post(handlers::portal_system_hand_activate))
        .route("/api/portal/system/hands/{hand_id}", axum::routing::get(handlers::portal_system_hand_detail))
        .route("/api/portal/system/providers/{name}/key", axum::routing::post(handlers::portal_system_provider_key))
        .route("/api/portal/system/providers/{name}/test", axum::routing::post(handlers::portal_system_provider_test))
        // Diagnostic: test OpenFang API connectivity
        .route("/api/portal/system/test", axum::routing::get(handlers::portal_system_test))
        // Independent Channel Instances (multi-channel support)
        .route("/api/portal/channel-instances", axum::routing::get(handlers::channel_instance_list).post(handlers::channel_instance_create))
        .route("/api/portal/channel-instances/{id}", axum::routing::get(handlers::channel_instance_detail).put(handlers::channel_instance_update).delete(handlers::channel_instance_delete))
        .route("/api/portal/channel-instances/{id}/test", axum::routing::post(handlers::channel_instance_test))
        .route("/api/portal/channel-instances/{id}/webhook", axum::routing::post(handlers::channel_instance_set_webhook))
        // Channel webhook receivers (incoming messages from Telegram/Zalo/etc.)
        .route("/webhook/ch/{id}", axum::routing::get(handlers::channel_webhook_verify).post(handlers::channel_webhook_receive))
        .with_state(state);

    // Add CORS
    let app = app.layer(
        tower_http::cors::CorsLayer::permissive()
    );

    info!("✅ Portal ready at http://{listen}");

    let listener = tokio::net::TcpListener::bind(&listen).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
