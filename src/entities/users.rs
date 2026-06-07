use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub disabled: bool,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::invite_codes::Entity")]
    InviteCodes,
    #[sea_orm(has_many = "super::tunnels::Entity")]
    Tunnels,
}

impl Related<super::invite_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InviteCodes.def()
    }
}

impl Related<super::tunnels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tunnels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
