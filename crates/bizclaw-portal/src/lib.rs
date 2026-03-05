//! BizClaw Portal — fully independent tenant management portal.
//!
//! This crate is completely separated from OpenFang core.
//! It has its own database (PORTAL_DATABASE_URL), its own data models,
//! and communicates with OpenFang only via HTTP API proxies.
//!
//! # Usage
//! ```ignore
//! let portal_state = bizclaw_portal::db::PortalState::new();
//! let portal_router = bizclaw_portal::portal_routes(portal_state);
//! let app = axum::Router::new().merge(portal_router);
//! ```

pub mod models;
pub mod db;
pub mod handlers;
pub mod html;

use std::sync::Arc;

/// Build the complete Portal router with all routes.
/// This returns an axum::Router that can be `.merge()`-ed into the main server.
pub fn portal_routes(state: db::PortalState) -> axum::Router {
    let state = Arc::new(state);

    // Initialize data with default plans
    db::init_data(&state);

    axum::Router::new()
        // Portal HTML pages
        .route("/portal/", axum::routing::get(handlers::portal_page))
        .route("/portal/{id}", axum::routing::get(handlers::portal_page_with_id))
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
        // System API proxies
        .route("/api/portal/system/channels", axum::routing::get(handlers::portal_system_channels))
        .route("/api/portal/system/providers", axum::routing::get(handlers::portal_system_providers))
        .route("/api/portal/system/models", axum::routing::get(handlers::portal_system_models))
        .route("/api/portal/system/skills", axum::routing::get(handlers::portal_system_skills))
        .route("/api/portal/system/hands", axum::routing::get(handlers::portal_system_hands))
        .with_state(state)
}
