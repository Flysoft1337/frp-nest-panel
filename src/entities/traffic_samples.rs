use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "traffic_samples")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub tunnel_id: Option<Uuid>,
    pub protocol: String,
    pub proxy_name: String,
    pub traffic_in: i64,
    pub traffic_out: i64,
    pub sampled_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tunnels::Entity",
        from = "Column::TunnelId",
        to = "super::tunnels::Column::Id"
    )]
    Tunnel,
}

impl Related<super::tunnels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tunnel.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
