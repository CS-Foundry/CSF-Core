use chrono::Utc;
use entity::entities::workloads;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
    QueryFilter,
};
use uuid::Uuid;

use crate::models::workload::{
    CreateWorkloadRequest, DesiredState, RestartPolicy, RuntimeClass, WorkloadResponse,
    WorkloadStatus,
};

pub async fn create(
    db: &DatabaseConnection,
    req: &CreateWorkloadRequest,
) -> Result<workloads::Model, sea_orm::DbErr> {
    let env_vars = req
        .env_vars
        .as_ref()
        .and_then(|e| serde_json::to_value(e).ok());
    let ports = req
        .ports
        .as_ref()
        .and_then(|p| serde_json::to_value(p).ok());
    let volume_mounts = req
        .volume_mounts
        .as_ref()
        .and_then(|v| serde_json::to_value(v).ok());

    let model = workloads::ActiveModel {
        id: Set(Uuid::new_v4()),
        name: Set(req.name.clone()),
        image: Set(req.image.clone()),
        cpu_millicores: Set(req.cpu_millicores),
        memory_bytes: Set(req.memory_bytes),
        disk_bytes: Set(req.disk_bytes),
        env_vars: Set(env_vars),
        ports: Set(ports),
        volume_mounts: Set(volume_mounts),
        status: Set(WorkloadStatus::Pending.as_str().to_string()),
        assigned_agent_id: Set(None),
        container_id: Set(None),
        created_by: Set(None),
        organization_id: Set(None),
        resource_group_id: Set(req.resource_group_id),
        stack_id: Set(req.stack_id),
        service_name: Set(req.service_name.clone()),
        restart_policy: Set(req.restart_policy.as_str().to_string()),
        max_restarts: Set(req.max_restarts),
        restart_count: Set(0),
        runtime_class: Set(req.runtime_class.as_str().to_string()),
        desired_state: Set(DesiredState::Running.as_str().to_string()),
        restart_requested: Set(false),
        cpu_usage_percent: Set(None),
        memory_usage_bytes: Set(None),
        network_rx_bytes: Set(None),
        network_tx_bytes: Set(None),
        stats_updated_at: Set(None),
        created_at: Set(Utc::now().naive_utc()),
        updated_at: Set(None),
    };

    model.insert(db).await
}

pub async fn assign(
    db: &DatabaseConnection,
    workload_id: Uuid,
    agent_id: Uuid,
) -> Result<workloads::Model, sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    active.assigned_agent_id = Set(Some(agent_id));
    active.status = Set(WorkloadStatus::Scheduled.as_str().to_string());
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await
}

pub async fn get_all(db: &DatabaseConnection) -> Result<Vec<WorkloadResponse>, sea_orm::DbErr> {
    let rows = workloads::Entity::find().all(db).await?;
    Ok(rows.into_iter().map(into_response).collect())
}

pub async fn get_pending(db: &DatabaseConnection) -> Result<Vec<workloads::Model>, sea_orm::DbErr> {
    workloads::Entity::find()
        .filter(workloads::Column::Status.eq(WorkloadStatus::Pending.as_str()))
        .filter(workloads::Column::AssignedAgentId.is_null())
        .all(db)
        .await
}

pub async fn get_by_id(
    db: &DatabaseConnection,
    workload_id: Uuid,
) -> Result<Option<workloads::Model>, sea_orm::DbErr> {
    workloads::Entity::find_by_id(workload_id).one(db).await
}

pub async fn update_container_status(
    db: &DatabaseConnection,
    workload_id: Uuid,
    container_id: &str,
    status: &str,
) -> Result<(), sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    if !container_id.is_empty() {
        active.container_id = Set(Some(container_id.to_string()));
    }
    active.status = Set(status.to_string());
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await?;
    Ok(())
}

pub async fn set_desired_state(
    db: &DatabaseConnection,
    workload_id: Uuid,
    desired_state: DesiredState,
) -> Result<workloads::Model, sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    active.desired_state = Set(desired_state.as_str().to_string());
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await
}

pub async fn request_restart(
    db: &DatabaseConnection,
    workload_id: Uuid,
) -> Result<workloads::Model, sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    active.desired_state = Set(DesiredState::Running.as_str().to_string());
    active.restart_requested = Set(true);
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await
}

pub async fn clear_restart_request(
    db: &DatabaseConnection,
    workload_id: Uuid,
) -> Result<(), sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    active.restart_requested = Set(false);
    active.updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await?;
    Ok(())
}

pub async fn update_stats(
    db: &DatabaseConnection,
    workload_id: Uuid,
    cpu_usage_percent: Option<f64>,
    memory_usage_bytes: Option<i64>,
    network_rx_bytes: Option<i64>,
    network_tx_bytes: Option<i64>,
) -> Result<(), sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    let mut active: workloads::ActiveModel = workload.into();
    active.cpu_usage_percent = Set(cpu_usage_percent);
    active.memory_usage_bytes = Set(memory_usage_bytes);
    active.network_rx_bytes = Set(network_rx_bytes);
    active.network_tx_bytes = Set(network_tx_bytes);
    active.stats_updated_at = Set(Some(Utc::now().naive_utc()));

    active.update(db).await?;
    Ok(())
}

pub async fn delete(db: &DatabaseConnection, workload_id: Uuid) -> Result<(), sea_orm::DbErr> {
    let workload = workloads::Entity::find_by_id(workload_id)
        .one(db)
        .await?
        .ok_or(sea_orm::DbErr::RecordNotFound(workload_id.to_string()))?;

    workload.delete(db).await?;
    Ok(())
}

fn into_response(m: workloads::Model) -> WorkloadResponse {
    let volume_mounts = m
        .volume_mounts
        .as_ref()
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    WorkloadResponse {
        id: m.id,
        name: m.name,
        image: m.image,
        cpu_millicores: m.cpu_millicores,
        memory_bytes: m.memory_bytes,
        disk_bytes: m.disk_bytes,
        status: WorkloadStatus::from_str(&m.status),
        assigned_agent_id: m.assigned_agent_id,
        container_id: m.container_id,
        volume_mounts,
        resource_group_id: m.resource_group_id,
        stack_id: m.stack_id,
        service_name: m.service_name,
        restart_policy: RestartPolicy::from_str(&m.restart_policy),
        max_restarts: m.max_restarts,
        restart_count: m.restart_count,
        runtime_class: RuntimeClass::from_str(&m.runtime_class),
        desired_state: DesiredState::from_str(&m.desired_state),
        cpu_usage_percent: m.cpu_usage_percent,
        memory_usage_bytes: m.memory_usage_bytes,
        network_rx_bytes: m.network_rx_bytes,
        network_tx_bytes: m.network_tx_bytes,
        stats_updated_at: m.stats_updated_at.map(|dt| dt.and_utc()),
        created_at: m.created_at.and_utc(),
        updated_at: m.updated_at.map(|dt| dt.and_utc()),
    }
}
