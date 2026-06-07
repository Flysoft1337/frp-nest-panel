use std::sync::Arc;

use minijinja::Environment;
use sea_orm::DatabaseConnection;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: DatabaseConnection,
    pub templates: Arc<Environment<'static>>,
}
