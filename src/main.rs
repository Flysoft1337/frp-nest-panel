mod auth;
mod config;
mod entities;
mod error;
mod routes;
mod services;
mod spa;
mod state;

use std::sync::{atomic::AtomicBool, Arc};

use axum::{routing::get, Router};
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, Database, EntityTrait};
use tokio::{net::TcpListener, sync::RwLock};
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tower_sessions::cookie::{time::Duration, Key};
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

use crate::{
    config::Config,
    entities::users,
    services::{frps, password},
    state::AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "frp_nest_panel=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let db = Database::connect(&config.database_url).await?;
    Migrator::up(&db, None).await?;
    ensure_initial_admin(&db, &config).await?;
    let frps_config = frps::load_runtime_config(&config).await?;

    let state = AppState {
        config: config.clone(),
        db,
        frps: Arc::new(RwLock::new(frps_config.clone())),
        frps_restarting: Arc::new(AtomicBool::new(false)),
    };

    let mut key_bytes = [0_u8; 64];
    for (index, byte) in config.session_secret.as_bytes().iter().take(64).enumerate() {
        key_bytes[index] = *byte;
    }
    let key = Key::from(&key_bytes);
    let session_layer = SessionManagerLayer::new(MemoryStore::default())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_signed(key);

    let app = Router::new()
        .merge(routes::router())
        .nest_service("/assets", ServeDir::new("frontend/dist/assets"))
        .route_service("/icons.svg", ServeFile::new("frontend/dist/icons.svg"))
        .route_service("/favicon.svg", ServeFile::new("frontend/dist/favicon.svg"))
        .fallback(get(spa::index))
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let panel_tls = crate::services::panel_tls::load_config()
        .await
        .unwrap_or_default();
    if panel_tls.enabled {
        let tls_app = app.clone();
        let tls_bind = panel_tls.bind.clone();
        tokio::spawn(async move {
            let Ok(config) = axum_server::tls_rustls::RustlsConfig::from_pem_file(
                crate::services::panel_tls::PANEL_TLS_CERT_PATH,
                crate::services::panel_tls::PANEL_TLS_KEY_PATH,
            )
            .await
            else {
                tracing::error!("failed to load panel TLS certificate");
                return;
            };
            let Ok(addr) = tls_bind.parse() else {
                tracing::error!(bind = %tls_bind, "invalid panel TLS bind address");
                return;
            };
            tracing::info!(bind = %tls_bind, "frp-nest-panel HTTPS starting");
            if let Err(error) = axum_server::bind_rustls(addr, config)
                .serve(tls_app.into_make_service())
                .await
            {
                tracing::error!(%error, "panel HTTPS server stopped");
            }
        });
    }

    let listener = TcpListener::bind(config.app_bind).await?;
    tracing::info!(
        bind = %config.app_bind,
        frps_server_addr = %frps_config.server_addr,
        frps_bind_port = frps_config.bind_port,
        remote_port_min = frps_config.remote_port_min,
        remote_port_max = frps_config.remote_port_max,
        user_max_tunnels = config.user_max_tunnels,
        "frp-nest-panel starting"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ensure_initial_admin(
    db: &sea_orm::DatabaseConnection,
    config: &Config,
) -> anyhow::Result<()> {
    if users::Entity::find_by_id(Uuid::nil())
        .one(db)
        .await?
        .is_some()
    {
        return Ok(());
    }

    users::ActiveModel {
        id: Set(Uuid::nil()),
        username: Set(config.initial_admin_username.clone()),
        password_hash: Set(password::hash_password(&config.initial_admin_password)?),
        role: Set("admin".to_owned()),
        disabled: Set(false),
        created_at: Set(chrono::Utc::now().fixed_offset()),
        max_tunnels: Set(None),
    }
    .insert(db)
    .await?;
    Ok(())
}
