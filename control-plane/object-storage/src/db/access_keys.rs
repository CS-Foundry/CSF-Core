use anyhow::{Context, Result};
use chrono::Utc;
use entity::entities::bucket_access_keys;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub async fn insert(
    db: &DatabaseConnection,
    bucket_id: Uuid,
    name: &str,
    garage_key_id: &str,
    permissions: &str,
    expires_at: Option<chrono::NaiveDateTime>,
) -> Result<bucket_access_keys::Model> {
    let model = bucket_access_keys::ActiveModel {
        id: Set(Uuid::new_v4()),
        bucket_id: Set(bucket_id),
        name: Set(name.to_string()),
        garage_key_id: Set(garage_key_id.to_string()),
        permissions: Set(permissions.to_string()),
        expires_at: Set(expires_at),
        last_rotated_at: Set(None),
        created_at: Set(Utc::now().naive_utc()),
    };

    model
        .insert(db)
        .await
        .context("failed to insert bucket access key")
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Option<bucket_access_keys::Model>> {
    bucket_access_keys::Entity::find_by_id(id)
        .one(db)
        .await
        .context("failed to get bucket access key")
}

pub async fn list_for_bucket(
    db: &DatabaseConnection,
    bucket_id: Uuid,
) -> Result<Vec<bucket_access_keys::Model>> {
    bucket_access_keys::Entity::find()
        .filter(bucket_access_keys::Column::BucketId.eq(bucket_id))
        .all(db)
        .await
        .context("failed to list bucket access keys")
}

pub async fn touch_rotated(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    if let Some(existing) = get_by_id(db, id).await? {
        let mut model: bucket_access_keys::ActiveModel = existing.into();
        model.last_rotated_at = Set(Some(Utc::now().naive_utc()));
        model
            .update(db)
            .await
            .context("failed to update bucket access key rotation timestamp")?;
    }
    Ok(())
}

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<()> {
    bucket_access_keys::Entity::delete_by_id(id)
        .exec(db)
        .await
        .context("failed to delete bucket access key")?;
    Ok(())
}
