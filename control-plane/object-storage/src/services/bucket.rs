use anyhow::{bail, Result};
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    db::buckets as db,
    garage::GarageClient,
    log_error, log_info,
    models::{BucketResponse, CreateBucketRequest, UpdateBucketRequest},
};

pub async fn create_bucket(
    db_conn: &DatabaseConnection,
    garage: &GarageClient,
    req: CreateBucketRequest,
) -> Result<BucketResponse> {
    let global_alias = format!("{}-{}", req.name, Uuid::new_v4().simple());

    let garage_bucket = match garage.create_bucket(&global_alias).await {
        Ok(bucket) => bucket,
        Err(e) => {
            log_error!("services::bucket", &format!("garage create_bucket failed name={} err={}", req.name, e));
            bail!("failed to create bucket in garage: {}", e);
        }
    };

    if req.quota_max_size.is_some() || req.quota_max_objects.is_some() {
        if let Err(e) = garage
            .update_bucket_quotas(&garage_bucket.id, req.quota_max_size, req.quota_max_objects)
            .await
        {
            log_error!("services::bucket", &format!("garage update_bucket_quotas failed bucket_id={} err={}", garage_bucket.id, e));
            let _ = garage.delete_bucket(&garage_bucket.id).await;
            bail!("failed to apply bucket quota: {}", e);
        }
    }

    let model = db::insert(db_conn, &req, &global_alias, &garage_bucket.id).await?;
    log_info!("services::bucket", &format!("bucket created id={} garage_bucket_id={}", model.id, garage_bucket.id));

    Ok(db::into_response(model))
}

pub async fn get_bucket(
    db_conn: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<BucketResponse>> {
    Ok(db::get_by_id(db_conn, id).await?.map(db::into_response))
}

pub async fn list_buckets(
    db_conn: &DatabaseConnection,
    resource_group_id: Option<Uuid>,
    organization_id: Option<Uuid>,
) -> Result<Vec<BucketResponse>> {
    let rows = db::list(db_conn, resource_group_id, organization_id).await?;
    Ok(rows.into_iter().map(db::into_response).collect())
}

pub async fn update_bucket(
    db_conn: &DatabaseConnection,
    garage: &GarageClient,
    id: Uuid,
    req: UpdateBucketRequest,
) -> Result<Option<BucketResponse>> {
    let Some(existing) = db::get_by_id(db_conn, id).await? else {
        return Ok(None);
    };

    if req.quota_max_size.is_some() || req.quota_max_objects.is_some() {
        let Some(garage_bucket_id) = &existing.garage_bucket_id else {
            bail!("bucket has no garage_bucket_id, cannot update quota");
        };
        garage
            .update_bucket_quotas(garage_bucket_id, req.quota_max_size, req.quota_max_objects)
            .await?;
    }

    let updated = db::update(db_conn, id, &req).await?.map(db::into_response);
    Ok(updated)
}

pub async fn delete_bucket(
    db_conn: &DatabaseConnection,
    garage: &GarageClient,
    id: Uuid,
) -> Result<bool> {
    let Some(existing) = db::get_by_id(db_conn, id).await? else {
        return Ok(false);
    };

    if let Some(garage_bucket_id) = &existing.garage_bucket_id {
        garage.delete_bucket(garage_bucket_id).await?;
    }

    db::delete(db_conn, id).await?;
    log_info!("services::bucket", &format!("bucket deleted id={}", id));
    Ok(true)
}
