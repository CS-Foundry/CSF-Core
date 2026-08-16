use anyhow::{bail, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    db::{access_keys as keys_db, buckets as buckets_db},
    garage::GarageClient,
    log_error, log_info,
    models::{AccessKeyCreatedResponse, AccessKeyResponse, CreateAccessKeyRequest},
};

fn into_response(model: entity::entities::bucket_access_keys::Model) -> AccessKeyResponse {
    AccessKeyResponse {
        id: model.id,
        bucket_id: model.bucket_id,
        name: model.name,
        garage_key_id: model.garage_key_id,
        permissions: model.permissions,
        expires_at: model.expires_at,
        last_rotated_at: model.last_rotated_at,
        created_at: model.created_at,
    }
}

pub async fn create_key(
    db: &DatabaseConnection,
    garage: &GarageClient,
    bucket_id: Uuid,
    req: CreateAccessKeyRequest,
) -> Result<Option<AccessKeyCreatedResponse>> {
    let Some(bucket) = buckets_db::get_by_id(db, bucket_id).await? else {
        return Ok(None);
    };
    let Some(garage_bucket_id) = &bucket.garage_bucket_id else {
        bail!("bucket has no garage_bucket_id, cannot create access key");
    };

    let permissions = req.permissions.unwrap_or_else(|| "readwrite".to_string());

    let garage_key = match garage.create_key(&req.name).await {
        Ok(key) => key,
        Err(e) => {
            log_error!("services::access_key", &format!("garage create_key failed name={} err={}", req.name, e));
            bail!("failed to create access key in garage: {}", e);
        }
    };

    if let Err(e) = garage
        .allow_bucket_key(garage_bucket_id, &garage_key.access_key_id, &permissions)
        .await
    {
        log_error!("services::access_key", &format!("garage allow_bucket_key failed key_id={} err={}", garage_key.access_key_id, e));
        let _ = garage.delete_key(&garage_key.access_key_id).await;
        bail!("failed to grant bucket access: {}", e);
    }

    let model = keys_db::insert(
        db,
        bucket_id,
        &req.name,
        &garage_key.access_key_id,
        &permissions,
        req.expires_at,
    )
    .await?;

    log_info!("services::access_key", &format!("access key created id={} bucket_id={}", model.id, bucket_id));

    Ok(Some(AccessKeyCreatedResponse {
        key: into_response(model),
        secret_access_key: garage_key.secret_access_key,
    }))
}

pub async fn list_keys(
    db: &DatabaseConnection,
    bucket_id: Uuid,
) -> Result<Vec<AccessKeyResponse>> {
    let rows = keys_db::list_for_bucket(db, bucket_id).await?;
    Ok(rows.into_iter().map(into_response).collect())
}

pub async fn rotate_key(
    db: &DatabaseConnection,
    garage: &GarageClient,
    bucket_id: Uuid,
    key_id: Uuid,
) -> Result<Option<AccessKeyCreatedResponse>> {
    let Some(existing) = keys_db::get_by_id(db, key_id).await? else {
        return Ok(None);
    };
    if existing.bucket_id != bucket_id {
        return Ok(None);
    }

    let created = create_key(
        db,
        garage,
        bucket_id,
        CreateAccessKeyRequest {
            name: existing.name.clone(),
            permissions: Some(existing.permissions.clone()),
            expires_at: existing.expires_at,
        },
    )
    .await?;

    let Some(created) = created else {
        return Ok(None);
    };

    if let Err(e) = garage.delete_key(&existing.garage_key_id).await {
        log_error!("services::access_key", &format!("garage delete_key on rotate failed old_key_id={} err={}", existing.garage_key_id, e));
    }
    keys_db::delete(db, key_id).await?;
    keys_db::touch_rotated(db, created.key.id).await?;

    log_info!("services::access_key", &format!("access key rotated old_id={} new_id={}", key_id, created.key.id));

    Ok(Some(created))
}

pub async fn delete_key(
    db: &DatabaseConnection,
    garage: &GarageClient,
    bucket_id: Uuid,
    key_id: Uuid,
) -> Result<bool> {
    let Some(existing) = keys_db::get_by_id(db, key_id).await? else {
        return Ok(false);
    };
    if existing.bucket_id != bucket_id {
        return Ok(false);
    }

    garage.delete_key(&existing.garage_key_id).await?;
    keys_db::delete(db, key_id).await?;

    log_info!("services::access_key", &format!("access key deleted id={}", key_id));
    Ok(true)
}
