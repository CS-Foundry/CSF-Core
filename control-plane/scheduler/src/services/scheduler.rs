use chrono::Utc;
use etcd_client::Client as EtcdClient;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::models::compose::{ComposeServiceSpec, CreateStackRequest, CreateStackResponse};
use crate::models::workload::{
    AgentResources, CreateWorkloadRequest, CreateWorkloadResponse, WorkloadStatus,
};
use crate::services::compose_parser::{self, ComposeParseError};
use crate::services::etcd::{delete_placement, put_placement, PlacementRecord};

pub struct SchedulerService {
    db: DatabaseConnection,
    etcd: Arc<Mutex<EtcdClient>>,
}

impl SchedulerService {
    pub fn new(db: DatabaseConnection, etcd: Arc<Mutex<EtcdClient>>) -> Self {
        Self { db, etcd }
    }

    pub async fn schedule(
        &self,
        req: CreateWorkloadRequest,
    ) -> Result<CreateWorkloadResponse, String> {
        let workload = crate::db::workloads::create(&self.db, &req)
            .await
            .map_err(|e| format!("Failed to persist workload: {}", e))?;

        let mut agents = crate::db::agents::get_online_agents_with_resources(&self.db)
            .await
            .map_err(|e| format!("Failed to fetch agent resources: {}", e))?;

        for agent in agents.iter_mut() {
            let (reserved_cpu, reserved_mem, reserved_disk) =
                crate::db::agents::get_assigned_workload_resources(&self.db, agent.agent_id)
                    .await
                    .map_err(|e| format!("Failed to fetch reserved resources: {}", e))?;

            agent.free_cpu_millicores -= reserved_cpu;
            agent.free_memory_bytes -= reserved_mem;
            agent.free_disk_bytes -= reserved_disk;
        }

        let volume_pinned_agent = self.resolve_volume_affinity(&req).await?;

        if let Some(required_agent) = volume_pinned_agent {
            agents.retain(|a| a.agent_id == required_agent);
        }

        match self.first_fit(&req, &agents) {
            Some(agent_id) => {
                crate::db::workloads::assign(&self.db, workload.id, agent_id)
                    .await
                    .map_err(|e| format!("Failed to assign workload: {}", e))?;

                let record = PlacementRecord {
                    workload_id: workload.id,
                    agent_id,
                    image: req.image.clone(),
                    cpu_millicores: req.cpu_millicores,
                    memory_bytes: req.memory_bytes,
                    disk_bytes: req.disk_bytes,
                    scheduled_at: Utc::now().to_rfc3339(),
                    stack_id: req.stack_id,
                    service_name: req.service_name.clone(),
                };

                put_placement(&self.etcd, &record).await?;

                crate::log_info!(
                    "scheduler",
                    &format!(
                        "Workload scheduled workload_id={} agent_id={}",
                        workload.id, agent_id
                    )
                );

                Ok(CreateWorkloadResponse {
                    workload_id: workload.id,
                    status: WorkloadStatus::Scheduled,
                    assigned_agent_id: Some(agent_id),
                    message: format!("Workload assigned to agent {}", agent_id),
                })
            }
            None => {
                crate::log_warn!(
                    "scheduler",
                    &format!("No suitable agent found workload_id={}", workload.id)
                );

                Ok(CreateWorkloadResponse {
                    workload_id: workload.id,
                    status: WorkloadStatus::Pending,
                    assigned_agent_id: None,
                    message: "No agent with sufficient resources available".to_string(),
                })
            }
        }
    }

    pub async fn schedule_stack(
        &self,
        req: CreateStackRequest,
    ) -> Result<CreateStackResponse, String> {
        let services = compose_parser::parse_compose(&req.compose_yaml)
            .map_err(|e| format_compose_error(&e))?;

        let stack = crate::db::workload_stacks::create(
            &self.db,
            req.resource_group_id,
            &req.name,
            &req.compose_yaml,
        )
        .await
        .map_err(|e| format!("Failed to persist stack: {}", e))?;

        let workloads = self
            .place_stack_workloads(stack.id, req.resource_group_id, &services)
            .await?;

        crate::db::workload_stacks::update_status(
            &self.db,
            stack.id,
            if workloads.iter().all(|w| w.assigned_agent_id.is_some()) {
                "active"
            } else {
                "pending"
            },
        )
        .await
        .map_err(|e| format!("Failed to update stack status: {}", e))?;

        Ok(CreateStackResponse {
            stack_id: stack.id,
            workloads,
        })
    }

    async fn place_stack_workloads(
        &self,
        stack_id: Uuid,
        resource_group_id: Uuid,
        services: &[ComposeServiceSpec],
    ) -> Result<Vec<CreateWorkloadResponse>, String> {
        let mut agents = crate::db::agents::get_online_agents_with_resources(&self.db)
            .await
            .map_err(|e| format!("Failed to fetch agent resources: {}", e))?;

        for agent in agents.iter_mut() {
            let (reserved_cpu, reserved_mem, reserved_disk) =
                crate::db::agents::get_assigned_workload_resources(&self.db, agent.agent_id)
                    .await
                    .map_err(|e| format!("Failed to fetch reserved resources: {}", e))?;

            agent.free_cpu_millicores -= reserved_cpu;
            agent.free_memory_bytes -= reserved_mem;
            agent.free_disk_bytes -= reserved_disk;
        }

        let total_cpu: i32 = services.iter().map(|s| s.cpu_millicores).sum();
        let total_memory: i64 = services.iter().map(|s| s.memory_bytes).sum();
        let total_disk: i64 = services.iter().map(|s| s.disk_bytes).sum();

        let agent_id = self.first_fit_resources(total_cpu, total_memory, total_disk, &agents);

        let mut responses = Vec::with_capacity(services.len());
        for service in services {
            let req = CreateWorkloadRequest {
                name: format!("{}-{}", stack_id, service.service_name),
                image: service.image.clone(),
                cpu_millicores: service.cpu_millicores,
                memory_bytes: service.memory_bytes,
                disk_bytes: service.disk_bytes,
                env_vars: service.env_vars.clone(),
                ports: service.ports.clone(),
                volume_mounts: None,
                resource_group_id: Some(resource_group_id),
                stack_id: Some(stack_id),
                service_name: Some(service.service_name.clone()),
            };

            let workload = crate::db::workloads::create(&self.db, &req)
                .await
                .map_err(|e| format!("Failed to persist workload: {}", e))?;

            responses.push(match agent_id {
                Some(agent_id) => {
                    crate::db::workloads::assign(&self.db, workload.id, agent_id)
                        .await
                        .map_err(|e| format!("Failed to assign workload: {}", e))?;

                    let record = PlacementRecord {
                        workload_id: workload.id,
                        agent_id,
                        image: req.image.clone(),
                        cpu_millicores: req.cpu_millicores,
                        memory_bytes: req.memory_bytes,
                        disk_bytes: req.disk_bytes,
                        scheduled_at: Utc::now().to_rfc3339(),
                        stack_id: Some(stack_id),
                        service_name: Some(service.service_name.clone()),
                    };
                    put_placement(&self.etcd, &record).await?;

                    CreateWorkloadResponse {
                        workload_id: workload.id,
                        status: WorkloadStatus::Scheduled,
                        assigned_agent_id: Some(agent_id),
                        message: format!("Workload assigned to agent {}", agent_id),
                    }
                }
                None => CreateWorkloadResponse {
                    workload_id: workload.id,
                    status: WorkloadStatus::Pending,
                    assigned_agent_id: None,
                    message: "No agent with sufficient resources available".to_string(),
                },
            });
        }

        crate::log_info!(
            "scheduler",
            &format!(
                "Stack scheduled stack_id={} agent_id={:?} services={}",
                stack_id,
                agent_id,
                services.len()
            )
        );

        Ok(responses)
    }

    async fn resolve_volume_affinity(
        &self,
        req: &CreateWorkloadRequest,
    ) -> Result<Option<Uuid>, String> {
        let mounts = match &req.volume_mounts {
            Some(m) if !m.is_empty() => m,
            _ => return Ok(None),
        };

        let mut pinned: Option<Uuid> = None;

        for mount in mounts {
            let agent_id =
                crate::db::agents::get_volume_agent(&self.db, mount.volume_id)
                    .await
                    .map_err(|e| format!("Failed to check volume affinity: {}", e))?;

            if let Some(aid) = agent_id {
                match pinned {
                    None => pinned = Some(aid),
                    Some(existing) if existing != aid => {
                        return Err(format!(
                            "Volume mounts require conflicting agents: {} vs {}",
                            existing, aid
                        ));
                    }
                    _ => {}
                }
            }
        }

        Ok(pinned)
    }

    fn first_fit(&self, req: &CreateWorkloadRequest, agents: &[AgentResources]) -> Option<Uuid> {
        self.first_fit_resources(
            req.cpu_millicores,
            req.memory_bytes,
            req.disk_bytes,
            agents,
        )
    }

    fn first_fit_resources(
        &self,
        cpu_millicores: i32,
        memory_bytes: i64,
        disk_bytes: i64,
        agents: &[AgentResources],
    ) -> Option<Uuid> {
        let mut sorted: Vec<&AgentResources> = agents.iter().collect();
        sorted.sort_by(|a, b| {
            let score_a = a.free_cpu_millicores as i64 + a.free_memory_bytes / (1024 * 1024);
            let score_b = b.free_cpu_millicores as i64 + b.free_memory_bytes / (1024 * 1024);
            score_b.cmp(&score_a)
        });

        sorted
            .into_iter()
            .find(|a| {
                a.free_cpu_millicores >= cpu_millicores
                    && a.free_memory_bytes >= memory_bytes
                    && a.free_disk_bytes >= disk_bytes
            })
            .map(|a| a.agent_id)
    }

    pub async fn list_workloads(
        &self,
    ) -> Result<Vec<crate::models::workload::WorkloadResponse>, String> {
        crate::db::workloads::get_all(&self.db)
            .await
            .map_err(|e| format!("Failed to list workloads: {}", e))
    }

    pub async fn delete_workload(&self, workload_id: Uuid) -> Result<(), String> {
        crate::db::workloads::delete(&self.db, workload_id)
            .await
            .map_err(|e| format!("Failed to delete workload: {}", e))?;

        delete_placement(&self.etcd, workload_id).await?;

        crate::log_info!(
            "scheduler",
            &format!("Workload deleted workload_id={}", workload_id)
        );

        Ok(())
    }
}

fn format_compose_error(error: &ComposeParseError) -> String {
    match error {
        ComposeParseError::InvalidYaml(detail) => format!("invalid compose yaml: {}", detail),
        ComposeParseError::NoServicesDefined => "compose file defines no services".to_string(),
        ComposeParseError::UnsupportedBuildDirective(service) => format!(
            "service '{}' uses build, only image is supported",
            service
        ),
        ComposeParseError::MissingImage(service) => {
            format!("service '{}' has no image", service)
        }
    }
}
