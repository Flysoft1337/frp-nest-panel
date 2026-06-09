use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "tunnels")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub protocol: String,
    pub local_host: String,
    pub local_port: i32,
    pub remote_port: Option<i32>,
    pub custom_domain: Option<String>,
    pub tls_mode: Option<String>,
    pub certificate_id: Option<Uuid>,
    pub use_encryption: bool,
    pub use_compression: bool,
    pub bandwidth_limit: Option<String>,
    pub bandwidth_limit_mode: Option<String>,
    pub proxy_protocol_version: Option<String>,
    pub locations: Option<String>,
    pub host_header_rewrite: Option<String>,
    pub updated_at: DateTimeWithTimeZone,
    pub config_changed_at: DateTimeWithTimeZone,
    pub last_config_viewed_at: Option<DateTimeWithTimeZone>,
    pub last_config_downloaded_at: Option<DateTimeWithTimeZone>,
    pub config_version: i32,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::certificates::Entity",
        from = "Column::CertificateId",
        to = "super::certificates::Column::Id"
    )]
    Certificate,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    User,
}

impl Related<super::certificates::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Certificate.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
