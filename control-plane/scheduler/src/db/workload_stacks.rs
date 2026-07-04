use chrono::Utc;
use entity::entities::workload_stacks;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use uuid::Uuid;

pub async fn create(
    db: &DatabaseConnection,
    resource_group_id: Uuid,
    name: &str,
    compose_source: &str,
) -> Result<workload_stacks::Model, sea_orm::DbErr> {
    let model = workload_stacks::ActiveModel {
        id: Set(Uuid::new_v4()),
        resource_group_id: Set(resource_group_id),
        name: Set(name.to_string()),
        compose_source: Set(Some(compose_source.to_string())),
        status: Set("pending".to_string()),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(None),
    };

    model.insert(db).await
}

pub async fn update_status(
    db: &DatabaseConnection,
    stack_id: Uuid,
    status: &str,
) -> Result<(), sea_orm::DbErr> {
    let stack = workload_stacks::Entity::find_by_id(stack_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(stack_id.to_string()))?;

    let mut active: workload_stacks::ActiveModel = stack.into();
    active.status = Set(status.to_string());
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await?;
    Ok(())
}
