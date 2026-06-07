pub mod admin;
pub mod auth;
pub mod dashboard;
pub mod health;
pub mod tunnels;

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(dashboard::home))
        .route("/healthz", get(health::healthz))
        .route("/login", get(auth::login_page).post(auth::login))
        .route("/register", get(auth::register_page).post(auth::register))
        .route("/logout", post(auth::logout))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/tunnels/new", get(tunnels::new_page).post(tunnels::create))
        .route("/tunnels/{id}/delete", post(tunnels::delete))
        .route("/tunnels/{id}/frpc.toml", get(tunnels::download_frpc))
        .route("/admin", get(admin::index))
        .route(
            "/admin/invites",
            get(admin::invites).post(admin::create_invite),
        )
        .route("/admin/users", get(admin::users))
        .route("/admin/users/{id}/disable", post(admin::disable_user))
        .route("/admin/users/{id}/enable", post(admin::enable_user))
        .route("/admin/tunnels", get(admin::all_tunnels))
}
