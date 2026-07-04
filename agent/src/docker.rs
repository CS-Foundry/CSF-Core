use anyhow::{Context, Result};
use axum::body::Bytes;
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{
    ContainerCreateBody, ContainerStateStatusEnum, EndpointSettings, HostConfig,
    NetworkCreateRequest, NetworkingConfig,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, InspectContainerOptionsBuilder,
    ListNetworksOptionsBuilder, LogsOptionsBuilder, StartContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::{Stream, StreamExt};
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
    pub stack_id: Option<String>,
    pub service_name: Option<String>,
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

    pub async fn ensure_stack_network(&self, stack_id: &str) -> Result<String> {
        let network_name = format!("csfx-stack-{}", stack_id);

        let filters = HashMap::from([("name".to_string(), vec![network_name.clone()])]);
        let list_options = ListNetworksOptionsBuilder::default()
            .filters(&filters)
            .build();

        let existing = self
            .docker
            .list_networks(Some(list_options))
            .await
            .context("Failed to list networks")?;

        if let Some(network) = existing.into_iter().find(|n| n.name.as_deref() == Some(&network_name)) {
            return Ok(network.id.unwrap_or(network_name));
        }

        let config = NetworkCreateRequest {
            name: network_name.clone(),
            driver: Some("bridge".to_string()),
            ..Default::default()
        };

        let response = self
            .docker
            .create_network(config)
            .await
            .context("Failed to create stack network")?;

        info!(stack_id = %stack_id, network = %network_name, "Stack network ready");

        Ok(response.id)
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

        let networking_config = match &spec.stack_id {
            Some(stack_id) => {
                let network_name = self.ensure_stack_network(stack_id).await?;
                let aliases = spec.service_name.clone().map(|name| vec![name]);
                Some(NetworkingConfig {
                    endpoints_config: Some(HashMap::from([(
                        network_name,
                        EndpointSettings {
                            aliases,
                            ..Default::default()
                        },
                    )])),
                })
            }
            None => None,
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
            networking_config,
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

    pub async fn inspect_status(&self, container_id: &str) -> Result<String> {
        let options = InspectContainerOptionsBuilder::default().build();

        let inspect = self
            .docker
            .inspect_container(container_id, Some(options))
            .await
            .context("Failed to inspect container")?;

        let state = inspect.state.unwrap_or_default();
        let status = state.status.unwrap_or(ContainerStateStatusEnum::EMPTY);
        let exit_code = state.exit_code.unwrap_or(0);

        Ok(match status {
            ContainerStateStatusEnum::CREATED => "creating".to_string(),
            ContainerStateStatusEnum::RUNNING => "running".to_string(),
            ContainerStateStatusEnum::RESTARTING | ContainerStateStatusEnum::PAUSED => {
                "running".to_string()
            }
            ContainerStateStatusEnum::EXITED if exit_code == 0 => "stopped".to_string(),
            ContainerStateStatusEnum::EXITED | ContainerStateStatusEnum::DEAD => {
                "failed".to_string()
            }
            ContainerStateStatusEnum::REMOVING | ContainerStateStatusEnum::STOPPING => {
                "stopped".to_string()
            }
            ContainerStateStatusEnum::EMPTY => "failed".to_string(),
        })
    }

    pub fn logs(
        &self,
        container_id: &str,
    ) -> impl Stream<Item = Result<Bytes, std::io::Error>> + Send + 'static {
        let options = LogsOptionsBuilder::default()
            .stdout(true)
            .stderr(true)
            .follow(true)
            .tail("200")
            .build();

        self.docker
            .logs(container_id, Some(options))
            .map(|item| match item {
                Ok(log_output) => Ok(Bytes::from(log_output.into_bytes())),
                Err(e) => Err(std::io::Error::other(e.to_string())),
            })
    }

    pub async fn exec(&self, container_id: &str) -> Result<StartExecResults> {
        let create_options = CreateExecOptions {
            attach_stdin: Some(true),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(true),
            cmd: Some(vec!["/bin/sh".to_string()]),
            ..Default::default()
        };

        let created = self
            .docker
            .create_exec(container_id, create_options)
            .await
            .context("Failed to create exec session")?;

        let start_options = StartExecOptions {
            tty: true,
            ..Default::default()
        };

        self.docker
            .start_exec(&created.id, Some(start_options))
            .await
            .context("Failed to start exec session")
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
