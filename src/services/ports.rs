use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    entities::tunnels,
    error::{AppError, AppResult},
};

pub async fn allocate_remote_port(db: &DatabaseConnection, min: i32, max: i32) -> AppResult<i32> {
    let used = tunnels::Entity::find()
        .filter(tunnels::Column::RemotePort.gte(min))
        .filter(tunnels::Column::RemotePort.lte(max))
        .all(db)
        .await?
        .into_iter()
        .map(|tunnel| tunnel.remote_port)
        .collect::<std::collections::HashSet<_>>();

    (min..=max)
        .find(|port| !used.contains(port))
        .ok_or_else(|| AppError::BadRequest("没有可用远程端口".to_owned()))
}
