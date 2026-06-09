use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection};
use serde_json::{json, Value};
use uuid::Uuid;

use crate::entities::{audit_logs, users};

pub struct AuditEvent<'a> {
    pub actor: Option<&'a users::Model>,
    pub action: &'a str,
    pub resource_type: &'a str,
    pub resource_id: Option<Uuid>,
    pub resource_name: Option<String>,
    pub outcome: &'a str,
    pub message: Option<String>,
    pub metadata: Option<Value>,
}

pub async fn record(db: &DatabaseConnection, event: AuditEvent<'_>) {
    let metadata_json = event.metadata.map(|value| value.to_string());
    let result = audit_logs::ActiveModel {
        id: Set(Uuid::new_v4()),
        actor_user_id: Set(event.actor.map(|user| user.id)),
        actor_username: Set(event.actor.map(|user| user.username.clone())),
        actor_role: Set(event.actor.map(|user| user.role.clone())),
        action: Set(event.action.to_owned()),
        resource_type: Set(event.resource_type.to_owned()),
        resource_id: Set(event.resource_id),
        resource_name: Set(event.resource_name),
        outcome: Set(event.outcome.to_owned()),
        message: Set(event.message),
        metadata_json: Set(metadata_json),
        ip_address: Set(None),
        user_agent: Set(None),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(db)
    .await;

    if let Err(error) = result {
        tracing::warn!(%error, action = event.action, "failed to write audit log");
    }
}

pub fn metadata(value: Value) -> Option<Value> {
    match value {
        Value::Object(map) if map.is_empty() => None,
        other => Some(other),
    }
}

pub fn changed_fields(fields: Vec<&'static str>) -> Option<Value> {
    metadata(json!({ "changed_fields": fields }))
}
