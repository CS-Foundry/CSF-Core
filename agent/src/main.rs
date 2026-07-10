mod client;
mod config;
mod docker;
mod firecracker;
mod nftables;
mod pki;
mod rbd;
mod runtime;
mod server;
mod ssh_keys;
mod system;
mod update_watch;
mod wg_identity;
mod wireguard;

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("failed to install ring crypto provider");

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let gateway_url = std::env::var("CSFX_GATEWAY_URL")
        .context("CSFX_GATEWAY_URL environment variable is required")?;

    if let Some(username) = std::env::args()
        .nth(1)
        .filter(|a| a == "--authorized-keys")
        .and_then(|_| std::env::args().nth(2))
    {
        let agent_id = config::load_config()
            .context("Failed to load daemon config")?
            .agent_id;
        ssh_keys::run_authorized_keys_command(&gateway_url, agent_id).await;
        let _ = username;
        return Ok(());
    }

    info!(version = env!("CARGO_PKG_VERSION"), "csfx-agent starting");

    let heartbeat_interval_secs: u64 = std::env::var("CSFX_HEARTBEAT_INTERVAL")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60);

    let wg_endpoint = std::env::var("CSFX_WG_ENDPOINT").ok();
    let wg_tunnel_ip = std::env::var("CSFX_WG_TUNNEL_IP").ok();
    let wg_identity =
        wg_identity::load_or_generate().context("Failed to initialize WireGuard identity")?;

    let api_client = client::ApiClient::new(
        gateway_url.clone(),
        wg_identity.public_key_b64.clone(),
        wg_endpoint,
        wg_tunnel_ip,
    )
    .context("Failed to initialize API client")?;

    let agent_pki = pki::AgentPki::load_or_generate().context("Failed to initialize PKI")?;

    let (agent_id, api_key) = if config::is_registered() {
        info!("Existing registration found, loading credentials");
        let cfg = config::load_config().context("Failed to load daemon config")?;
        let creds = config::load_credentials().context("Failed to load credentials")?;
        (cfg.agent_id, creds.api_key)
    } else {
        info!("No registration found, starting registration");
        perform_registration(
            &api_client,
            &gateway_url,
            heartbeat_interval_secs,
            &agent_pki,
        )
        .await?
    };

    let api_client = if pki::AgentPki::has_certificate() {
        match pki::AgentPki::load_cert_pem() {
            Ok(cert_pem) => {
                info!("mTLS: client certificate loaded");
                api_client.with_certificate(cert_pem)
            }
            Err(e) => {
                warn!(error = %e, "mTLS: failed to load certificate, continuing without");
                api_client
            }
        }
    } else {
        api_client
    };

    info!(agent_id = %agent_id, "Agent registered, starting heartbeat loop");

    if let Err(e) = nftables::ensure_table_and_chain().await {
        warn!(error = %e, "Failed to initialize nftables resource group isolation");
    }

    let docker_manager: Arc<Mutex<Option<Box<dyn runtime::Runtime>>>> = Arc::new(Mutex::new(None));

    let running_containers: Arc<Mutex<HashMap<String, String>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let workload_phases: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let mounted_volumes: Arc<Mutex<HashMap<String, String>>> = Arc::new(Mutex::new(HashMap::new()));

    let restart_counts: Arc<Mutex<HashMap<String, u32>>> = Arc::new(Mutex::new(HashMap::new()));

    if let Some(port) = std::env::var("CSFX_AGENT_PORT")
        .ok()
        .and_then(|v| v.parse::<u16>().ok())
    {
        let server_state = server::ServerState {
            docker: docker_manager.clone(),
            running_containers: running_containers.clone(),
            agent_id,
        };
        tokio::spawn(async move {
            if let Err(e) = server::run(server_state, port).await {
                error!(error = %e, "agent inbound server stopped");
            }
        });
    } else {
        info!("CSFX_AGENT_PORT not set, agent inbound server disabled");
    }

    run_heartbeat_loop(
        &api_client,
        agent_id,
        &api_key,
        heartbeat_interval_secs,
        docker_manager,
        running_containers,
        workload_phases,
        mounted_volumes,
        restart_counts,
        &wg_identity.private_key_b64,
    )
    .await;

    Ok(())
}

async fn perform_registration(
    client: &client::ApiClient,
    gateway_url: &str,
    heartbeat_interval_secs: u64,
    agent_pki: &pki::AgentPki,
) -> Result<(uuid::Uuid, String)> {
    let token = match std::env::var("CSFX_REGISTRATION_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
    {
        Some(t) => t,
        None => {
            info!("CSFX_REGISTRATION_TOKEN not set, fetching bootstrap token from gateway");
            client
                .fetch_bootstrap_token()
                .await
                .context("Failed to fetch bootstrap token from gateway")?
        }
    };

    let info = system::collect_info();

    info!(
        hostname = %info.hostname,
        os_type = %info.os_type,
        architecture = %info.architecture,
        "Registering with registry"
    );

    let resp = client
        .register(
            &token,
            &info.hostname,
            &info.hostname,
            &info.os_type,
            &info.os_version,
            &info.architecture,
            agent_pki.csr_pem(),
        )
        .await
        .context("Registration request failed")?;

    if let (Some(cert_pem), Some(ca_pem)) = (&resp.certificate_pem, &resp.ca_cert_pem) {
        pki::AgentPki::save_certificate(cert_pem, ca_pem).context("Failed to save certificate")?;
        info!("PKI: certificate received and stored");
    } else {
        warn!("Registry did not issue a certificate during registration");
    }

    let cfg = config::DaemonConfig {
        gateway_url: gateway_url.to_string(),
        agent_id: resp.agent_id,
        heartbeat_interval_secs,
    };

    config::save_config(&cfg).context("Failed to save daemon config")?;
    config::save_credentials(&resp.api_key).context("Failed to save credentials")?;

    info!(agent_id = %resp.agent_id, "Registration successful");

    Ok((resp.agent_id, resp.api_key))
}

async fn run_heartbeat_loop(
    client: &client::ApiClient,
    agent_id: uuid::Uuid,
    api_key: &str,
    interval_secs: u64,
    docker: Arc<Mutex<Option<Box<dyn runtime::Runtime>>>>,
    running_containers: Arc<Mutex<HashMap<String, String>>>,
    workload_phases: Arc<Mutex<HashMap<String, String>>>,
    mounted_volumes: Arc<Mutex<HashMap<String, String>>>,
    restart_counts: Arc<Mutex<HashMap<String, u32>>>,
    wg_private_key_b64: &str,
) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    let mut failure_count: u32 = 0;
    let mut current_flake_rev = String::new();

    loop {
        tokio::select! {
            _ = interval.tick() => {
                process_volumes(client, agent_id, api_key, &mounted_volumes).await;
                let resource_group_ids = process_workloads(client, api_key, &docker, &running_containers, &workload_phases, &mounted_volumes, &restart_counts, wg_private_key_b64).await;
                sync_wireguard_peers(client, api_key, agent_id, &resource_group_ids).await;
                cleanup_stale_resource_groups(client, api_key, wg_private_key_b64).await;

                let statuses = build_container_statuses(&docker, &running_containers, &workload_phases).await;
                let metrics = system::collect_metrics();

                match client.heartbeat(agent_id, api_key, Some(statuses), Some(metrics)).await {
                    Ok(resp) => {
                        if failure_count > 0 {
                            info!(agent_id = %agent_id, "Heartbeat recovered after {} failures", failure_count);
                            failure_count = 0;
                        }

                        info!(
                            agent_id = %agent_id,
                            desired_flake_rev = ?resp.desired_flake_rev,
                            "heartbeat ok"
                        );

                        if let Some(count) = resp.post_update_heartbeats {
                            update_watch::write_heartbeat_counter(count).await;
                        }

                        if let Some(rev) = resp.desired_flake_rev {
                            if rev != current_flake_rev {
                                info!(
                                    agent_id = %agent_id,
                                    current_flake_rev = %current_flake_rev,
                                    desired_flake_rev = %rev,
                                    "update signal received from gateway, scheduling update"
                                );
                            }
                            let rev_clone = rev.clone();
                            let current = current_flake_rev.clone();
                            tokio::spawn(async move {
                                update_watch::handle(agent_id, &rev_clone, &current).await;
                            });
                            current_flake_rev = rev;
                        }
                    }
                    Err(e) => {
                        failure_count += 1;
                        warn!(
                            agent_id = %agent_id,
                            failures = failure_count,
                            error = %e,
                            "Heartbeat failed"
                        );
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    if failure_count > 0 {
        error!(
            failures = failure_count,
            "Agent shutting down with unresolved heartbeat failures"
        );
    }
}

async fn process_volumes(
    client: &client::ApiClient,
    agent_id: uuid::Uuid,
    api_key: &str,
    mounted_volumes: &Arc<Mutex<HashMap<String, String>>>,
) {
    let volumes = match client.fetch_assigned_volumes(agent_id, api_key).await {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "Failed to fetch assigned volumes");
            return;
        }
    };

    for volume in volumes {
        let already_mounted = mounted_volumes.lock().await.contains_key(&volume.id);
        if already_mounted {
            continue;
        }

        info!(volume_id = %volume.id, image = %volume.image_name, "Mapping volume");

        let device = match rbd::map_device(&volume.pool, &volume.image_name).await {
            Ok(d) => d,
            Err(e) => {
                warn!(volume_id = %volume.id, error = %e, "Failed to map RBD device");
                continue;
            }
        };

        let mount_point = rbd::mount_point_for(&volume.id);

        if let Err(e) = rbd::mount(&device, &mount_point).await {
            warn!(volume_id = %volume.id, error = %e, "Failed to mount device");
            let _ = rbd::unmap_device(&device).await;
            continue;
        }

        mounted_volumes
            .lock()
            .await
            .insert(volume.id.clone(), device.clone());

        info!(
            volume_id = %volume.id,
            device = %device,
            mount_point = %mount_point,
            "Volume mounted"
        );
    }
}

async fn reap_stale_containers(
    docker: &dyn runtime::Runtime,
    running_containers: &Arc<Mutex<HashMap<String, String>>>,
    workload_phases: &Arc<Mutex<HashMap<String, String>>>,
    restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
    desired: &[client::AssignedWorkload],
) {
    let desired_ids: std::collections::HashSet<&str> =
        desired.iter().map(|w| w.id.as_str()).collect();

    let stale: Vec<(String, String)> = running_containers
        .lock()
        .await
        .iter()
        .filter(|(workload_id, _)| !desired_ids.contains(workload_id.as_str()))
        .map(|(workload_id, container_id)| (workload_id.clone(), container_id.clone()))
        .collect();

    for (workload_id, container_id) in stale {
        info!(workload_id = %workload_id, container_id = %container_id, "Tearing down removed workload");

        if let Err(e) = docker.stop_workload(&container_id).await {
            warn!(workload_id = %workload_id, container_id = %container_id, error = %e, "Failed to tear down container");
            continue;
        }

        running_containers.lock().await.remove(&workload_id);
        workload_phases.lock().await.remove(&workload_id);
        restart_counts.lock().await.remove(&workload_id);
    }
}

async fn should_restart_after_crash(
    docker: &dyn runtime::Runtime,
    workload: &client::AssignedWorkload,
    container_id: &str,
    restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
) -> bool {
    let status = match docker.inspect_status(container_id).await {
        Ok(s) => s,
        Err(e) => {
            warn!(workload_id = %workload.id, container_id = %container_id, error = %e, "Failed to inspect container");
            return false;
        }
    };

    if status != "failed" {
        return false;
    }

    if workload.restart_policy == "never" {
        return false;
    }

    let mut counts = restart_counts.lock().await;
    let count = counts.entry(workload.id.clone()).or_insert(0);

    if let Some(max) = workload.max_restarts {
        if *count as i32 >= max {
            warn!(workload_id = %workload.id, restart_count = *count, max_restarts = max, "Crash-loop limit reached, not restarting");
            return false;
        }
    }

    *count += 1;
    info!(workload_id = %workload.id, restart_count = *count, "Container crashed, scheduling restart");
    true
}

async fn process_workloads(
    client: &client::ApiClient,
    api_key: &str,
    docker: &Arc<Mutex<Option<Box<dyn runtime::Runtime>>>>,
    running_containers: &Arc<Mutex<HashMap<String, String>>>,
    workload_phases: &Arc<Mutex<HashMap<String, String>>>,
    mounted_volumes: &Arc<Mutex<HashMap<String, String>>>,
    restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
    wg_private_key_b64: &str,
) -> Vec<String> {
    let workloads = match client.fetch_assigned_workloads(api_key).await {
        Ok(w) => w,
        Err(e) => {
            warn!(error = %e, "Failed to fetch assigned workloads");
            return Vec::new();
        }
    };

    let resource_group_ids: Vec<String> = {
        let mut ids: Vec<String> = workloads
            .iter()
            .filter_map(|w| w.resource_group_id.clone())
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    };

    let mut docker_guard = docker.lock().await;
    if docker_guard.is_none() {
        match docker::DockerRuntime::ensure_running(wg_private_key_b64.to_string()).await {
            Ok(dm) => {
                info!("Docker ready");
                *docker_guard = Some(Box::new(dm));
            }
            Err(e) => {
                warn!(error = %e, "Docker unavailable, deferring workloads");
                return resource_group_ids;
            }
        }
    }
    let docker_runtime: &dyn runtime::Runtime = docker_guard
        .as_deref()
        .expect("docker manager initialized above");

    reap_stale_containers(
        docker_runtime,
        running_containers,
        workload_phases,
        restart_counts,
        &workloads,
    )
    .await;

    let mut firecracker_runtime: Option<firecracker::runtime::FirecrackerRuntime> = None;

    for workload in workloads {
        let selected_runtime: &dyn runtime::Runtime = match workload.runtime_class.as_str() {
            "docker" => docker_runtime,
            "firecracker" => {
                if firecracker_runtime.is_none() {
                    match docker::DockerRuntime::ensure_running(wg_private_key_b64.to_string())
                        .await
                    {
                        Ok(dm) => {
                            firecracker_runtime =
                                Some(firecracker::runtime::FirecrackerRuntime::new(dm));
                        }
                        Err(e) => {
                            warn!(workload_id = %workload.id, error = %e, "Docker unavailable for rootfs build, deferring firecracker workload");
                            continue;
                        }
                    }
                }
                firecracker_runtime.as_ref().expect("initialized above")
            }
            other => {
                warn!(workload_id = %workload.id, runtime_class = %other, "Unsupported runtime class, skipping");
                continue;
            }
        };

        start_or_restart_workload(
            selected_runtime,
            workload,
            running_containers,
            workload_phases,
            mounted_volumes,
            restart_counts,
        )
        .await;
    }

    resource_group_ids
}

async fn start_or_restart_workload(
    runtime: &dyn runtime::Runtime,
    workload: client::AssignedWorkload,
    running_containers: &Arc<Mutex<HashMap<String, String>>>,
    workload_phases: &Arc<Mutex<HashMap<String, String>>>,
    mounted_volumes: &Arc<Mutex<HashMap<String, String>>>,
    restart_counts: &Arc<Mutex<HashMap<String, u32>>>,
) {
    let existing_container_id = running_containers.lock().await.get(&workload.id).cloned();

    if let Some(container_id) = existing_container_id {
        if !should_restart_after_crash(runtime, &workload, &container_id, restart_counts).await {
            return;
        }
        running_containers.lock().await.remove(&workload.id);
    }

    info!(workload_id = %workload.id, image = %workload.image, "Starting workload");

    workload_phases
        .lock()
        .await
        .insert(workload.id.clone(), "pulling".to_string());

    if let Err(e) = runtime.pull_image(&workload.image).await {
        warn!(workload_id = %workload.id, error = %e, "Failed to pull image");
        workload_phases.lock().await.remove(&workload.id);
        return;
    }

    if let Some(ref mounts) = workload.volume_mounts {
        let locked = mounted_volumes.lock().await;
        let all_ready = mounts.iter().all(|m| locked.contains_key(&m.volume_id));
        drop(locked);
        if !all_ready {
            info!(workload_id = %workload.id, "Waiting for volumes to be mounted, deferring workload");
            return;
        }
    }

    workload_phases
        .lock()
        .await
        .insert(workload.id.clone(), "creating".to_string());

    let spec = docker::WorkloadSpec {
        workload_id: workload.id.clone(),
        image: workload.image.clone(),
        cpu_millicores: workload.cpu_millicores,
        memory_bytes: workload.memory_bytes,
        env_vars: workload.env_vars,
        ports: workload.ports,
        volume_mounts: workload.volume_mounts.map(|mounts| {
            mounts
                .into_iter()
                .map(|m| docker::VolumeMount {
                    volume_id: m.volume_id,
                    mount_path: m.mount_path,
                })
                .collect()
        }),
        service_name: workload.service_name,
        resource_group_id: workload.resource_group_id,
        resource_group_cidr: workload.resource_group_cidr,
    };

    match runtime.start_workload(&spec).await {
        Ok(container_id) => {
            workload_phases
                .lock()
                .await
                .insert(workload.id.clone(), "starting".to_string());
            running_containers
                .lock()
                .await
                .insert(workload.id.clone(), container_id.clone());
            info!(
                workload_id = %workload.id,
                container_id = %container_id,
                "Workload started"
            );
        }
        Err(e) => {
            workload_phases.lock().await.remove(&workload.id);
            warn!(workload_id = %workload.id, error = %e, "Failed to start workload");
        }
    }
}

async fn sync_wireguard_peers(
    client: &client::ApiClient,
    api_key: &str,
    agent_id: uuid::Uuid,
    resource_group_ids: &[String],
) {
    for resource_group_id in resource_group_ids {
        let peers = match client
            .fetch_resource_group_peers(api_key, resource_group_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                warn!(resource_group_id = %resource_group_id, error = %e, "Failed to fetch resource group peers");
                continue;
            }
        };

        let agent_id_str = agent_id.to_string();
        let wg_peers: Vec<wireguard::Peer> = peers
            .into_iter()
            .filter(|p| p.agent_id != agent_id_str)
            .map(|p| wireguard::Peer {
                public_key: p.wg_public_key,
                endpoint: Some(p.wg_endpoint),
                allowed_ips: format!("{}/32", p.wg_tunnel_ip),
            })
            .collect();

        if wg_peers.is_empty() {
            continue;
        }

        let iface = wireguard::rg_interface_name(resource_group_id);
        if let Err(e) = wireguard::set_peers(&iface, &wg_peers).await {
            warn!(resource_group_id = %resource_group_id, iface = %iface, error = %e, "Failed to sync WireGuard peers");
        }
    }
}

async fn cleanup_stale_resource_groups(
    client: &client::ApiClient,
    api_key: &str,
    wg_private_key_b64: &str,
) {
    let active_ids = match client.fetch_active_resource_group_ids(api_key).await {
        Ok(ids) => ids,
        Err(e) => {
            warn!(error = %e, "Failed to fetch active resource group ids");
            return;
        }
    };

    let docker = match docker::DockerRuntime::ensure_running(wg_private_key_b64.to_string()).await {
        Ok(d) => d,
        Err(e) => {
            warn!(error = %e, "Docker unavailable, deferring resource group cleanup");
            return;
        }
    };

    let local_ids = match docker.list_rg_ids().await {
        Ok(ids) => ids,
        Err(e) => {
            warn!(error = %e, "Failed to list local resource group networks");
            return;
        }
    };

    for local_id in local_ids {
        if active_ids.iter().any(|id| id == &local_id) {
            continue;
        }

        info!(resource_group_id = %local_id, "Tearing down stale resource group network");

        if let Err(e) = docker.teardown_rg_network(&local_id).await {
            warn!(resource_group_id = %local_id, error = %e, "Failed to tear down stale resource group network");
        }
    }
}

async fn build_container_statuses(
    docker: &Arc<Mutex<Option<Box<dyn runtime::Runtime>>>>,
    running_containers: &Arc<Mutex<HashMap<String, String>>>,
    workload_phases: &Arc<Mutex<HashMap<String, String>>>,
) -> Vec<client::ContainerStatus> {
    let containers = running_containers.lock().await.clone();
    let mut statuses = Vec::with_capacity(containers.len());

    let docker_guard = docker.lock().await;
    for (workload_id, container_id) in containers.iter() {
        let status = match docker_guard.as_ref() {
            Some(dm) => match dm.inspect_status(container_id).await {
                Ok(s) => s,
                Err(e) => {
                    warn!(workload_id = %workload_id, container_id = %container_id, error = %e, "Failed to inspect container");
                    "failed".to_string()
                }
            },
            None => "failed".to_string(),
        };

        if status != "creating" {
            workload_phases.lock().await.remove(workload_id);
        }

        statuses.push(client::ContainerStatus {
            workload_id: workload_id.clone(),
            container_id: container_id.clone(),
            status,
        });
    }
    drop(docker_guard);

    let phases = workload_phases.lock().await;
    for (workload_id, phase) in phases.iter() {
        if !containers.contains_key(workload_id) {
            statuses.push(client::ContainerStatus {
                workload_id: workload_id.clone(),
                container_id: String::new(),
                status: phase.clone(),
            });
        }
    }

    statuses
}
