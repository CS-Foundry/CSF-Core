use std::collections::HashMap;
use uuid::Uuid;

use crate::models::workload::PortMapping;

#[derive(Debug, Clone)]
pub struct ComposeServiceSpec {
    pub service_name: String,
    pub image: String,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub cpu_millicores: i32,
    pub memory_bytes: i64,
    pub disk_bytes: i64,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CreateStackRequest {
    pub name: String,
    pub resource_group_id: Uuid,
    pub compose_yaml: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CreateStackResponse {
    pub stack_id: Uuid,
    pub workloads: Vec<crate::models::workload::CreateWorkloadResponse>,
}
