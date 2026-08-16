use anyhow::{Context, Result};
use chrono::Utc;
use entity::entities::garage_nodes;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use uuid::Uuid;

pub async fn list(db: &DatabaseConnection) -> Result<Vec<garage_nodes::Model>> {
    garage_nodes::Entity::find()
        .all(db)
        .await
        .context("failed to list garage nodes")
}

pub async fn get_by_agent_id(
    db: &DatabaseConnection,
    agent_id: Uuid,
) -> Result<Option<garage_nodes::Model>> {
    garage_nodes::Entity::find()
        .filter(garage_nodes::Column::AgentId.eq(agent_id))
        .one(db)
        .await
        .context("failed to get garage node by agent id")
}

pub async fn mark_down(db: &DatabaseConnection, agent_id: Uuid) -> Result<()> {
    if let Some(existing) = get_by_agent_id(db, agent_id).await? {
        let mut model: garage_nodes::ActiveModel = existing.into();
        model.status = Set("down".to_string());
        model.updated_at = Set(Some(Utc::now().naive_utc()));
        model
            .update(db)
            .await
            .context("failed to mark garage node down")?;
    }
    Ok(())
}
