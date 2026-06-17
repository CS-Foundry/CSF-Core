use anyhow::{Context, Result};
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, StartContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkloadSpec {
    pub workload_id: String,
    pub name: String,
    pub image: String,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
}

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub fn new() -> Result<Self> {
        let docker =
            Docker::connect_with_unix_defaults().context("Failed to connect to Docker socket")?;
        Ok(Self { docker })
    }

    pub async fn pull_image(&self, image: &str) -> Result<()> {
        info!(image = %image, "Pulling image");

        let options = CreateImageOptionsBuilder::default()
            .from_image(image)
            .build();

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(info) => {
                    if let Some(status) = info.status {
                        if status != "Pulling fs layer"
                            && status != "Waiting"
                            && status != "Downloading"
                            && status != "Verifying Checksum"
                            && status != "Extracting"
                        {
                            info!(image = %image, status = %status, "Pull progress");
                        }
                    }
                }
                Err(e) => {
                    warn!(image = %image, error = %e, "Pull stream error");
                }
            }
        }

        info!(image = %image, "Image pull complete");
        Ok(())
    }

    pub async fn start_container(&self, spec: &WorkloadSpec) -> Result<String> {
        let container_name = format!("csfx-{}", spec.workload_id);

        let env: Option<Vec<String>> = spec
            .env_vars
            .as_ref()
            .map(|vars| vars.iter().map(|(k, v)| format!("{}={}", k, v)).collect());

        let (port_bindings, exposed_ports) = build_port_config(spec.ports.as_deref());

        let host_config = HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            ..Default::default()
        };

        let config = ContainerCreateBody {
            image: Some(spec.image.clone()),
            env,
            exposed_ports: if exposed_ports.is_empty() {
                None
            } else {
                Some(exposed_ports)
            },
            host_config: Some(host_config),
            labels: Some(HashMap::from([
                ("csfx.workload_id".to_string(), spec.workload_id.clone()),
                ("csfx.managed".to_string(), "true".to_string()),
            ])),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name(&container_name)
            .build();

        let container = self
            .docker
            .create_container(Some(options), config)
            .await
            .context("Failed to create container")?;

        let start_options = StartContainerOptionsBuilder::default().build();

        self.docker
            .start_container(&container.id, Some(start_options))
            .await
            .context("Failed to start container")?;

        info!(
            workload_id = %spec.workload_id,
            container_id = %container.id,
            "Container started"
        );

        Ok(container.id)
    }

    pub async fn stop_container(&self, container_id: &str) -> Result<()> {
        self.docker
            .stop_container(container_id, None)
            .await
            .context("Failed to stop container")?;

        self.docker
            .remove_container(container_id, None)
            .await
            .context("Failed to remove container")?;

        info!(container_id = %container_id, "Container stopped and removed");
        Ok(())
    }
}

fn build_port_config(
    ports: Option<&[PortMapping]>,
) -> (
    HashMap<String, Option<Vec<bollard::models::PortBinding>>>,
    Vec<String>,
) {
    let mut port_bindings: HashMap<String, Option<Vec<bollard::models::PortBinding>>> =
        HashMap::new();
    let mut exposed_ports: Vec<String> = Vec::new();

    if let Some(ports) = ports {
        for p in ports {
            let proto = p.protocol.as_deref().unwrap_or("tcp");
            let container_key = format!("{}/{}", p.container_port, proto);

            port_bindings.insert(
                container_key.clone(),
                Some(vec![bollard::models::PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(p.host_port.to_string()),
                }]),
            );

            exposed_ports.push(container_key);
        }
    }

    (port_bindings, exposed_ports)
}
