use std::sync::{atomic::AtomicBool, Arc};

use sea_orm::DatabaseConnection;
use tokio::sync::RwLock;

use crate::{config::Config, services::frps::FrpsRuntimeConfig};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabaseConnection,
    pub frps: Arc<RwLock<FrpsRuntimeConfig>>,
    pub frps_restarting: Arc<AtomicBool>,
}
