use anyhow::{Context, Result};
use chrono::Utc;
use entity::entities::buckets;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

use crate::models::{BucketResponse, CreateBucketRequest, UpdateBucketRequest};

pub fn into_response(model: buckets::Model) -> BucketResponse {
    BucketResponse {
        id: model.id,
        name: model.name,
        global_alias: model.global_alias,
        exposure: model.exposure,
        quota_max_size: model.quota_max_size,
        quota_max_objects: model.quota_max_objects,
        status: model.status,
        organization_id: model.organization_id,
        resource_group_id: model.resource_group_id,
        created_at: model.created_at,
        updated_at: model.updated_at,
    }
}

pub async fn insert(
    db: &DatabaseConnection,
    req: &CreateBucketRequest,
    global_alias: &str,
    garage_bucket_id: &str,
    master_key_id: &str,
    master_key_secret_encrypted: Vec<u8>,
) -> Result<buckets::Model> {
    let model = buckets::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name.clone()),
        garage_bucket_id: Set(Some(garage_bucket_id.to_string())),
        global_alias: Set(global_alias.to_string()),
        exposure: Set(req
            .exposure
            .clone()
            .unwrap_or_else(|| "internal".to_string())),
        quota_max_size: Set(req.quota_max_size),
        quota_max_objects: Set(req.quota_max_objects),
        status: Set("active".to_string()),
        master_key_id: Set(Some(master_key_id.to_string())),
        master_key_secret_encrypted: Set(Some(master_key_secret_encrypted)),
        organization_id: Set(req.organization_id),
        resource_group_id: Set(req.resource_group_id),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(None),
    };

    model.insert(db).await.context("failed to insert bucket")
}

pub async fn get_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<buckets::Model>> {
    buckets::Entity::find_by_id(id)
        .one(db)
        .await
        .context("failed to get bucket")
}

pub async fn list(
    db: &DatabaseConnection,
    resource_group_id: Option<Uuid>,
    organization_id: Option<Uuid>,
) -> Result<Vec<buckets::Model>> {
    let mut query = buckets::Entity::find();

    if let Some(rg_id) = resource_group_id {
        query = query.filter(buckets::Column::ResourceGroupId.eq(rg_id));
    }

    if let Some(org_id) = organization_id {
        query = query.filter(buckets::Column::OrganizationId.eq(org_id));
    }

    query.all(db).await.context("failed to list buckets")
}

pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    req: &UpdateBucketRequest,
) -> Result<Option<buckets::Model>> {
    let Some(existing) = get_by_id(db, id).await? else {
        return Ok(None);
    };

    let mut model: buckets::ActiveModel = existing.into();

    if let Some(exposure) = &req.exposure {
        model.exposure = Set(exposure.clone());
    }
    if req.quota_max_size.is_some() {
        model.quota_max_size = Set(req.quota_max_size);
    }
    if req.quota_max_objects.is_some() {
        model.quota_max_objects = Set(req.quota_max_objects);
    }
    model.updated_at = Set(Some(Utc::now().naive_utc()));

    let updated = model.update(db).await.context("failed to update bucket")?;
    Ok(Some(updated))
}

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    buckets::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("failed to delete bucket")?;
    Ok(())
}
