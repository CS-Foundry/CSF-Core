use anyhow::{Context, Result};
use bollard::exec::{CreateExecOptions, StartExecOptions, StartExecResults};
use bollard::models::{
    ContainerCreateBody, ContainerStateStatusEnum, EndpointSettings, HostConfig, Ipam, IpamConfig,
    NetworkCreateRequest, NetworkingConfig,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, InspectContainerOptionsBuilder,
    ListNetworksOptionsBuilder, LogsOptionsBuilder, StartContainerOptionsBuilder,
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
    pub image: String,
    pub cpu_millicores: i32,
    pub memory_bytes: i64,
    pub env_vars: Option<HashMap<String, String>>,
    pub ports: Option<Vec<PortMapping>>,
    pub volume_mounts: Option<Vec<VolumeMount>>,
    pub service_name: Option<String>,
    pub resource_group_id: Option<String>,
    pub resource_group_cidr: Option<String>,
}

pub fn rg_network_name(resource_group_id: &str) -> String {
    format!("csfx-rg-{}", resource_group_id)
}

pub fn rg_wireguard_port(resource_group_id: &str) -> u16 {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in resource_group_id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    49152u16 + (hash % (65535 - 49152)) as u16
}

pub fn rg_bridge_iface_name(resource_group_id: &str) -> String {
    const FNV_OFFSET_BASIS: u32 = 0x811c9dc5;
    const FNV_PRIME: u32 = 0x01000193;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in resource_group_id.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("csfxrg{:08x}", hash)
}

pub struct DockerRuntime {
    docker: Docker,
    wg_private_key_b64: String,
}

impl DockerRuntime {
    pub fn into_docker_handle(self) -> Docker {
        self.docker
    }

    pub async fn ensure_running(wg_private_key_b64: String) -> Result<Self> {
        if !Path::new(DOCKER_SOCKET_PATH).exists() {
            start_docker_unit().await?;
            wait_for_socket().await?;
        }

        let docker =
            Docker::connect_with_unix_defaults().context("Failed to connect to Docker socket")?;
        Ok(Self {
            docker,
            wg_private_key_b64,
        })
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

    pub async fn ensure_rg_network(
        &self,
        resource_group_id: &str,
        resource_group_cidr: Option<&str>,
    ) -> Result<String> {
        let network_name = rg_network_name(resource_group_id);

        let filters = HashMap::from([("name".to_string(), vec![network_name.clone()])]);
        let list_options = ListNetworksOptionsBuilder::default()
            .filters(&filters)
            .build();

        let existing = self
            .docker
            .list_networks(Some(list_options))
            .await
            .context("Failed to list networks")?;

        if let Some(network) = existing
            .into_iter()
            .find(|n| n.name.as_deref() == Some(&network_name))
        {
            return Ok(network.id.unwrap_or(network_name));
        }

        let ipam = resource_group_cidr.map(|cidr| Ipam {
            driver: Some("default".to_string()),
            config: Some(vec![IpamConfig {
                subnet: Some(cidr.to_string()),
                ..Default::default()
            }]),
            ..Default::default()
        });

        let iface_name = rg_bridge_iface_name(resource_group_id);

        let config = NetworkCreateRequest {
            name: network_name.clone(),
            driver: Some("bridge".to_string()),
            ipam,
            options: Some(HashMap::from([(
                "com.docker.network.bridge.name".to_string(),
                iface_name.clone(),
            )])),
            labels: Some(HashMap::from([(
                "csfx.resource_group_id".to_string(),
                resource_group_id.to_string(),
            )])),
            ..Default::default()
        };

        let response = self
            .docker
            .create_network(config)
            .await
            .context("Failed to create resource group network")?;

        info!(resource_group_id = %resource_group_id, network = %network_name, iface = %iface_name, "Resource group network ready");

        let other_bridges = self.list_rg_bridge_ifaces(Some(&network_name)).await?;
        crate::nftables::isolate_bridge(&iface_name, &other_bridges)
            .await
            .context("Failed to apply nftables isolation for resource group network")?;

        if let Some(cidr) = resource_group_cidr {
            self.ensure_rg_wireguard(resource_group_id, cidr).await?;
        }

        Ok(response.id)
    }

    async fn ensure_rg_wireguard(
        &self,
        resource_group_id: &str,
        resource_group_cidr: &str,
    ) -> Result<()> {
        let listen_port = rg_wireguard_port(resource_group_id);

        let wg_iface = crate::wireguard::ensure_interface(
            resource_group_id,
            &self.wg_private_key_b64,
            listen_port,
        )
        .await
        .context("Failed to bring up resource group WireGuard interface")?;

        crate::wireguard::set_route(&wg_iface, resource_group_cidr)
            .await
            .context("Failed to route resource group CIDR over WireGuard interface")?;

        Ok(())
    }

    pub async fn list_rg_bridge_ifaces(
        &self,
        exclude_network_name: Option<&str>,
    ) -> Result<Vec<String>> {
        let filters = HashMap::from([("name".to_string(), vec!["csfx-rg-".to_string()])]);
        let list_options = ListNetworksOptionsBuilder::default()
            .filters(&filters)
            .build();

        let networks = self
            .docker
            .list_networks(Some(list_options))
            .await
            .context("Failed to list resource group networks")?;

        Ok(networks
            .into_iter()
            .filter(|n| exclude_network_name != n.name.as_deref())
            .filter_map(|n| n.name)
            .filter_map(|name| name.strip_prefix("csfx-rg-").map(rg_bridge_iface_name))
            .collect())
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
            nano_cpus: if spec.cpu_millicores > 0 {
                Some(spec.cpu_millicores as i64 * 1_000_000)
            } else {
                None
            },
            memory: if spec.memory_bytes > 0 {
                Some(spec.memory_bytes)
            } else {
                None
            },
            ..Default::default()
        };

        let networking_config = match &spec.resource_group_id {
            Some(resource_group_id) => {
                let network_name = self
                    .ensure_rg_network(resource_group_id, spec.resource_group_cidr.as_deref())
                    .await?;
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

    fn logs_stream(&self, container_id: &str) -> crate::runtime::LogStream {
        let options = LogsOptionsBuilder::default()
            .stdout(true)
            .stderr(true)
            .follow(true)
            .tail("200")
            .build();

        Box::pin(
            self.docker
                .logs(container_id, Some(options))
                .map(|item| match item {
                    Ok(log_output) => Ok(log_output.into_bytes()),
                    Err(e) => Err(std::io::Error::other(e.to_string())),
                }),
        )
    }

    async fn exec_session(&self, container_id: &str) -> Result<crate::runtime::ExecSession> {
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

        let result = self
            .docker
            .start_exec(&created.id, Some(start_options))
            .await
            .context("Failed to start exec session")?;

        let StartExecResults::Attached { output, input } = result else {
            anyhow::bail!("exec session detached unexpectedly");
        };

        let output = output.map(|item| match item {
            Ok(log_output) => Ok(log_output.into_bytes()),
            Err(e) => Err(std::io::Error::other(e.to_string())),
        });

        Ok(crate::runtime::ExecSession {
            input: Box::pin(input),
            output: Box::pin(crate::runtime::StreamAsyncRead::new(output)),
        })
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

#[async_trait::async_trait]
impl crate::runtime::Runtime for DockerRuntime {
    async fn pull_image(&self, image: &str) -> Result<()> {
        self.pull_image(image).await
    }

    async fn start_workload(&self, spec: &WorkloadSpec) -> Result<String> {
        self.start_container(spec).await
    }

    async fn inspect_status(&self, workload_handle: &str) -> Result<String> {
        self.inspect_status(workload_handle).await
    }

    fn logs(&self, workload_handle: &str) -> crate::runtime::LogStream {
        self.logs_stream(workload_handle)
    }

    async fn exec(&self, workload_handle: &str) -> Result<crate::runtime::ExecSession> {
        self.exec_session(workload_handle).await
    }

    async fn stop_workload(&self, workload_handle: &str) -> Result<()> {
        self.stop_container(workload_handle).await
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
