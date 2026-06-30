use anyhow::{Context, Result};
use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, StartContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};
use zbus::Connection;

const DOCKER_SOCKET_PATH: &str = "/var/run/docker.sock";
const DOCKER_UNIT_NAME: &str = "docker.service";
const DOCKER_START_TIMEOUT: Duration = Duration::from_secs(30);
const DOCKER_SOCKET_POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub container_port: u16,
    pub protocol: Option<String>,
    pub node_port: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct VolumeMount {
    pub volume_id: String,
    pub mount_path: String,
}

#[derive(Debug, Clone)]
pub struct WorkloadSpec {
    pub workload_id: String,
    pub name: String,
    pub image: String,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
}

pub struct DockerManager {
    docker: Docker,
}

impl DockerManager {
    pub async fn ensure_running() -> Result<Self> {
        if !Path::new(DOCKER_SOCKET_PATH).exists() {
            start_docker_unit().await?;
            wait_for_socket().await?;
        }

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

        let binds = spec.volume_mounts.as_deref().map(|mounts| {
            mounts
                .iter()
                .map(|m| {
                    format!(
                        "{}:{}",
                        crate::rbd::mount_point_for(&m.volume_id),
                        m.mount_path
                    )
                })
                .collect::<Vec<_>>()
        });

        let host_config = HostConfig {
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            binds,
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

async fn start_docker_unit() -> Result<()> {
    info!(unit = DOCKER_UNIT_NAME, "Starting Docker via systemd");

    let connection = Connection::system()
        .await
        .context("Failed to connect to system D-Bus")?;

    let proxy = zbus::Proxy::new(
        &connection,
        "org.freedesktop.systemd1",
        "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager",
    )
    .await
    .context("Failed to build systemd D-Bus proxy")?;

    proxy
        .call_method("StartUnit", &(DOCKER_UNIT_NAME, "replace"))
        .await
        .context("Failed to call StartUnit for docker.service")?;

    Ok(())
}

async fn wait_for_socket() -> Result<()> {
    let deadline = tokio::time::Instant::now() + DOCKER_START_TIMEOUT;

    while tokio::time::Instant::now() < deadline {
        if Path::new(DOCKER_SOCKET_PATH).exists() {
            info!(socket = DOCKER_SOCKET_PATH, "Docker socket available");
            return Ok(());
        }
        tokio::time::sleep(DOCKER_SOCKET_POLL_INTERVAL).await;
    }

    anyhow::bail!(
        "Docker socket did not appear within {}s",
        DOCKER_START_TIMEOUT.as_secs()
    )
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
            exposed_ports.push(container_key.clone());

            if let Some(node_port) = p.node_port {
                port_bindings.insert(
                    container_key,
                    Some(vec![bollard::models::PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(node_port.to_string()),
                    }]),
                );
            }
        }
    }

    (port_bindings, exposed_ports)
}
