pub mod admin;
pub mod auth;
pub mod dashboard;
pub mod health;
pub mod tunnels;
pub mod types;

use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(health::healthz))
        .route("/api/session", get(auth::session))
        .route("/api/login", post(auth::login))
        .route("/api/register", post(auth::register))
        .route("/api/logout", post(auth::logout))
        .route("/api/password", post(auth::change_password))
        .route("/api/dashboard/summary", get(dashboard::summary))
        .route(
            "/api/tunnels",
            get(dashboard::tunnels).post(tunnels::create),
        )
        .route(
            "/api/tunnels/{id}",
            get(tunnels::get)
                .patch(tunnels::update)
                .delete(tunnels::delete),
        )
        .route("/api/tunnels/{id}/frpc", get(tunnels::preview_frpc))
        .route("/tunnels/{id}/frpc.toml", get(tunnels::download_frpc))
        .route("/api/admin/config", get(admin::config))
        .route(
            "/api/admin/invites",
            get(admin::invites).post(admin::create_invite),
        )
        .route("/api/admin/users", get(admin::users))
        .route("/api/admin/users/{id}/disable", post(admin::disable_user))
        .route("/api/admin/users/{id}/enable", post(admin::enable_user))
        .route(
            "/api/admin/users/{id}/reset-password",
            post(admin::reset_user_password),
        )
        .route("/api/admin/tunnels", get(admin::all_tunnels))
        .route("/api/admin/tunnels/{id}", delete(admin::delete_tunnel))
}
