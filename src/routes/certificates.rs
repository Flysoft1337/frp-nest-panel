use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::CurrentUser,
    entities::{certificates, tunnels},
    error::{AppError, AppResult},
    routes::types::{CertificateResponse, OkResponse},
    services::certificates as certificate_service,
    state::AppState,
};

#[derive(Deserialize)]
pub struct CertificateForm {
    name: String,
    certificate_pem: String,
    private_key_pem: String,
}

pub async fn list(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> AppResult<impl IntoResponse> {
    let items = certificates::Entity::find()
        .filter(certificates::Column::UserId.eq(user.id))
        .all(&state.db)
        .await?
        .into_iter()
        .map(CertificateResponse::from)
        .collect::<Vec<_>>();
    Ok(Json(items))
}

pub async fn create(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Json(form): Json<CertificateForm>,
) -> AppResult<impl IntoResponse> {
    let name = certificate_name(&form.name)?;
    let parsed =
        certificate_service::parse_and_validate(&form.certificate_pem, &form.private_key_pem)?;
    let certificate_id = Uuid::new_v4();
    let (cert_path, key_path) = certificate_service::write_certificate_files(
        user.id,
        certificate_id,
        &form.certificate_pem,
        &form.private_key_pem,
    )
    .await
    .map_err(|error| AppError::BadRequest(format!("保存证书失败: {error}")))?;

    let cert = certificates::ActiveModel {
        id: Set(certificate_id),
        user_id: Set(user.id),
        name: Set(name),
        domains_json: Set(
            serde_json::to_string(&parsed.domains).unwrap_or_else(|_| "[]".to_owned())
        ),
        cert_path: Set(cert_path),
        key_path: Set(key_path),
        not_before: Set(parsed.not_before),
        not_after: Set(parsed.not_after),
        fingerprint_sha256: Set(parsed.fingerprint_sha256),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(&state.db)
    .await?;

    Ok(Json(CertificateResponse::from(cert)))
}

pub async fn delete(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<Uuid>,
) -> AppResult<impl IntoResponse> {
    let Some(cert) = certificates::Entity::find_by_id(id).one(&state.db).await? else {
        return Err(AppError::NotFound);
    };
    if cert.user_id != user.id {
        return Err(AppError::Forbidden);
    }
    let used = tunnels::Entity::find()
        .filter(tunnels::Column::CertificateId.eq(id))
        .one(&state.db)
        .await?
        .is_some();
    if used {
        return Err(AppError::BadRequest("证书正在被隧道使用".to_owned()));
    }
    certificates::Entity::delete_by_id(id)
        .exec(&state.db)
        .await?;
    certificate_service::remove_certificate_files(user.id, id)
        .await
        .map_err(|error| AppError::BadRequest(format!("删除证书文件失败: {error}")))?;
    Ok(Json(OkResponse { ok: true }))
}

fn certificate_name(value: &str) -> AppResult<String> {
    let value = value.trim();
    if !(1..=64).contains(&value.len()) {
        return Err(AppError::BadRequest(
            "证书名称长度必须为 1-64 位".to_owned(),
        ));
    }
    Ok(value.to_owned())
}
