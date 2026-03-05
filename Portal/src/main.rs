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
    info!("   Database: {}", if state.pool.is_some() { "PostgreSQL" } else { "JSON file" });

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
        // Self-service
        .route("/api/portal/my/tenants", axum::routing::post(handlers::portal_create_my_tenant))
        // Users CRUD (admin)
        .route("/api/portal/users", axum::routing::get(handlers::portal_list_users).post(handlers::portal_create_user))
        .route("/api/portal/users/{email}", axum::routing::put(handlers::portal_update_user).delete(handlers::portal_delete_user))
        // Plans CRUD (admin)
        .route("/api/portal/plans", axum::routing::get(handlers::portal_list_plans).post(handlers::portal_create_plan))
        .route("/api/portal/plans/{id}", axum::routing::put(handlers::portal_update_plan).delete(handlers::portal_delete_plan))
        // System API proxies (calls OpenFang via HTTP)
        .route("/api/portal/system/channels", axum::routing::get(handlers::portal_system_channels))
        .route("/api/portal/system/providers", axum::routing::get(handlers::portal_system_providers))
        .route("/api/portal/system/models", axum::routing::get(handlers::portal_system_models))
        .route("/api/portal/system/skills", axum::routing::get(handlers::portal_system_skills))
        .route("/api/portal/system/hands", axum::routing::get(handlers::portal_system_hands))
        .with_state(state);

    // Add CORS
    let app = app.layer(
        tower_http::cors::CorsLayer::permissive()
    );

    info!("✅ Portal ready at http://{listen}");

    let listener = tokio::net::TcpListener::bind(&listen).await.expect("Failed to bind");
    axum::serve(listener, app).await.expect("Server error");
}
