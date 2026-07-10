use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WorkloadStatus {
    Pending,
    Scheduled,
    Pulling,
    Creating,
    Starting,
    Running,
    Failed,
    Stopped,
}

impl WorkloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkloadStatus::Pending => "pending",
            WorkloadStatus::Scheduled => "scheduled",
            WorkloadStatus::Pulling => "pulling",
            WorkloadStatus::Creating => "creating",
            WorkloadStatus::Starting => "starting",
            WorkloadStatus::Running => "running",
            WorkloadStatus::Failed => "failed",
            WorkloadStatus::Stopped => "stopped",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "scheduled" => WorkloadStatus::Scheduled,
            "pulling" => WorkloadStatus::Pulling,
            "creating" => WorkloadStatus::Creating,
            "starting" => WorkloadStatus::Starting,
            "running" => WorkloadStatus::Running,
            "failed" => WorkloadStatus::Failed,
            "stopped" => WorkloadStatus::Stopped,
            _ => WorkloadStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub volume_id: Uuid,
    pub mount_path: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Never,
}

impl RestartPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RestartPolicy::Always => "always",
            RestartPolicy::OnFailure => "on-failure",
            RestartPolicy::Never => "never",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "on-failure" => RestartPolicy::OnFailure,
            "never" => RestartPolicy::Never,
            _ => RestartPolicy::Always,
        }
    }
}

fn default_restart_policy() -> RestartPolicy {
    RestartPolicy::Always
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeClass {
    Docker,
    Firecracker,
    Vm,
}

impl RuntimeClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            RuntimeClass::Docker => "docker",
            RuntimeClass::Firecracker => "firecracker",
            RuntimeClass::Vm => "vm",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "firecracker" => RuntimeClass::Firecracker,
            "vm" => RuntimeClass::Vm,
            _ => RuntimeClass::Docker,
        }
    }

    pub fn requires_kvm(&self) -> bool {
        matches!(self, RuntimeClass::Firecracker | RuntimeClass::Vm)
    }
}

fn default_runtime_class() -> RuntimeClass {
    RuntimeClass::Docker
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkloadRequest {
    pub name: String,
    pub image: String,
    pub cpu_millicores: i32,
    pub memory_bytes: i64,
    pub disk_bytes: i64,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
    pub resource_group_id: Option<Uuid>,
    #[serde(default)]
    pub stack_id: Option<Uuid>,
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default = "default_restart_policy")]
    pub restart_policy: RestartPolicy,
    #[serde(default)]
    pub max_restarts: Option<i32>,
    #[serde(default = "default_runtime_class")]
    pub runtime_class: RuntimeClass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub protocol: Option<String>,
    pub node_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkloadResponse {
    pub workload_id: Uuid,
    pub status: WorkloadStatus,
    pub assigned_agent_id: Option<Uuid>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadResponse {
    pub id: Uuid,
    pub name: String,
    pub image: String,
    pub cpu_millicores: i32,
    pub memory_bytes: i64,
    pub disk_bytes: i64,
    pub status: WorkloadStatus,
    pub assigned_agent_id: Option<Uuid>,
    pub container_id: Option<String>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
    pub resource_group_id: Option<Uuid>,
    pub stack_id: Option<Uuid>,
    pub service_name: Option<String>,
    pub restart_policy: RestartPolicy,
    pub max_restarts: Option<i32>,
    pub restart_count: i32,
    pub runtime_class: RuntimeClass,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct AgentResources {
    pub agent_id: Uuid,
    pub free_cpu_millicores: i32,
    pub free_memory_bytes: i64,
    pub free_disk_bytes: i64,
    pub kvm_capable: bool,
}
