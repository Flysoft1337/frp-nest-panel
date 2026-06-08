use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "certificates")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub domains_json: String,
    pub cert_path: String,
    pub key_path: String,
    pub not_before: DateTimeWithTimeZone,
    pub not_after: DateTimeWithTimeZone,
    pub fingerprint_sha256: String,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::tunnels::Entity")]
    Tunnels,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::tunnels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tunnels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
