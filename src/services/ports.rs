use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use crate::{
    entities::tunnels,
    error::{AppError, AppResult},
};

pub async fn validate_remote_port_available(
    db: &DatabaseConnection,
    remote_port: i32,
    min: i32,
    max: i32,
) -> AppResult<i32> {
    if remote_port < min || remote_port > max {
        return Err(AppError::BadRequest("远程端口不在允许范围内".to_owned()));
    }
    let exists = tunnels::Entity::find()
        .filter(tunnels::Column::RemotePort.eq(remote_port))
        .one(db)
        .await?
        .is_some();
    if exists {
        return Err(AppError::BadRequest("远程端口已被占用".to_owned()));
    }
    Ok(remote_port)
}

pub async fn allocate_remote_port(db: &DatabaseConnection, min: i32, max: i32) -> AppResult<i32> {
    let used = tunnels::Entity::find()
        .filter(tunnels::Column::RemotePort.gte(min))
        .filter(tunnels::Column::RemotePort.lte(max))
        .all(db)
        .await?
        .into_iter()
        .filter_map(|tunnel| tunnel.remote_port)
        .collect::<std::collections::HashSet<_>>();

    (min..=max)
        .find(|port| !used.contains(port))
        .ok_or_else(|| AppError::BadRequest("没有可用远程端口".to_owned()))
}
