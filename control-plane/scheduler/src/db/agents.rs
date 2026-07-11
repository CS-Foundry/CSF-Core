use entity::entities::{agent_metrics, agents, volumes};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use uuid::Uuid;

use crate::models::workload::AgentResources;

const MILLICORES_PER_CORE: i32 = 1000;

pub async fn get_online_agents_with_resources(
    db: &DatabaseConnection,
) -> Result<Vec<AgentResources>, sea_orm::DbErr> {
    let online_agents = agents::Entity::find()
        .filter(agents::Column::Status.eq("Online"))
        .filter(agents::Column::Cordoned.eq(false))
        .all(db)
        .await?;

    let mut result = Vec::with_capacity(online_agents.len());

    for agent in online_agents {
        let latest_metrics = agent_metrics::Entity::find()
            .filter(agent_metrics::Column::AgentId.eq(agent.id))
            .order_by_desc(agent_metrics::Column::Timestamp)
            .one(db)
            .await?;

        let Some(m) = latest_metrics else {
            result.push(AgentResources {
                agent_id: agent.id,
                free_cpu_millicores: 0,
                free_memory_bytes: 0,
                free_disk_bytes: 0,
                kvm_capable: agent.kvm_capable,
            });
            continue;
        };

        let total_cpu_millicores = m.cpu_cores.unwrap_or(0) * MILLICORES_PER_CORE;
        let total_memory_bytes = m.memory_total_bytes.unwrap_or(0);
        let total_disk_bytes = m.disk_total_bytes.unwrap_or(0);

        let cpu_usage = m.cpu_usage_percent.unwrap_or(0.0);
        let used_cpu = ((cpu_usage / 100.0) * total_cpu_millicores as f32) as i32;
        let used_mem = m.memory_used_bytes.unwrap_or(0);
        let used_disk = m.disk_used_bytes.unwrap_or(0);

        result.push(AgentResources {
            agent_id: agent.id,
            free_cpu_millicores: total_cpu_millicores - used_cpu,
            free_memory_bytes: total_memory_bytes - used_mem,
            free_disk_bytes: total_disk_bytes - used_disk,
            kvm_capable: agent.kvm_capable,
        });
    }

    Ok(result)
}

pub async fn get_assigned_workload_resources(
    db: &DatabaseConnection,
    agent_id: Uuid,
) -> Result<(i32, i64, i64), sea_orm::DbErr> {
    use entity::entities::workloads;

    let workloads = workloads::Entity::find()
        .filter(workloads::Column::AssignedAgentId.eq(agent_id))
        .filter(
            workloads::Column::Status
                .eq("scheduled")
                .or(workloads::Column::Status.eq("running")),
        )
        .all(db)
        .await?;

    let cpu: i32 = workloads.iter().map(|w| w.cpu_millicores).sum();
    let mem: i64 = workloads.iter().map(|w| w.memory_bytes).sum();
    let disk: i64 = workloads.iter().map(|w| w.disk_bytes).sum();

    Ok((cpu, mem, disk))
}

pub async fn get_agents_hosting_resource_group(
    db: &DatabaseConnection,
    resource_group_id: Uuid,
) -> Result<std::collections::HashSet<Uuid>, sea_orm::DbErr> {
    use entity::entities::workloads;

    let rows = workloads::Entity::find()
        .filter(workloads::Column::ResourceGroupId.eq(resource_group_id))
        .filter(workloads::Column::AssignedAgentId.is_not_null())
        .filter(
            workloads::Column::Status
                .eq("scheduled")
                .or(workloads::Column::Status.eq("running")),
        )
        .all(db)
        .await?;

    Ok(rows
        .into_iter()
        .filter_map(|w| w.assigned_agent_id)
        .collect())
}

pub async fn get_volume_agent(
    db: &DatabaseConnection,
    volume_id: Uuid,
) -> Result<Option<Uuid>, sea_orm::DbErr> {
    let vol = volumes::Entity::find_by_id(volume_id).one(db).await?;
    Ok(vol.and_then(|v| v.attached_to_agent))
}
