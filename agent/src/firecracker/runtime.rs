use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::firecracker::api::FirecrackerApiClient;
use crate::firecracker::jailer_ids::JailerIdAllocator;
use crate::firecracker::rootfs::RootfsBuilder;
use crate::rg_dns_process::RgDnsProcessSupervisor;
use crate::runtime::{ExecSession, LogStream};
use crate::spec::{rg_wireguard_port, WorkloadSpec};

const JAILER_BASE_DIR: &str = "/var/lib/csfx-agent/firecracker";
const GUEST_KERNEL_PATH: &str = "/var/lib/csfx-agent/vmlinux";
const GUEST_CID_BASE: u32 = 1000;
const CGROUP_ROOT: &str = "/sys/fs/cgroup";
const CGROUP_PARENT: &str = "csfx-firecracker";

struct VmHandle {
    workload_id: String,
    jailer_pid: u32,
    chroot_dir: PathBuf,
    tap_device: String,
    vsock_cid: u32,
    resource_group_id: Option<String>,
    jailer_uid: u32,
    cgroup_path: PathBuf,
    metrics_path: PathBuf,
    previous_cpu_sample: Mutex<Option<(std::time::Instant, u64)>>,
}

struct GuestNetwork {
    ip: String,
    prefix: String,
    gateway: Option<String>,
}

pub struct FirecrackerRuntime {
    wg_private_key_b64: String,
    dns_supervisor: RgDnsProcessSupervisor,
    jailer_ids: JailerIdAllocator,
    handles: Mutex<HashMap<String, VmHandle>>,
    next_cid: Mutex<u32>,
    reconciled: AtomicBool,
}

impl FirecrackerRuntime {
    pub fn new(wg_private_key_b64: String) -> Self {
        Self {
            wg_private_key_b64,
            dns_supervisor: RgDnsProcessSupervisor::new(),
            jailer_ids: JailerIdAllocator::new(),
            handles: Mutex::new(HashMap::new()),
            next_cid: Mutex::new(GUEST_CID_BASE),
            reconciled: AtomicBool::new(false),
        }
    }

    pub async fn reconcile_once(&self) -> Vec<(String, String)> {
        if self.reconciled.swap(true, Ordering::SeqCst) {
            return Vec::new();
        }

        match crate::runtime::Runtime::list_managed_workloads(self).await {
            Ok(recovered) => {
                info!(
                    count = recovered.len(),
                    "Reconciled running microvms from disk state"
                );
                recovered
            }
            Err(e) => {
                warn!(error = %e, "Failed to reconcile firecracker workloads from disk state");
                Vec::new()
            }
        }
    }

    async fn ensure_rg_network(
        &self,
        resource_group_id: &str,
        resource_group_cidr: Option<&str>,
    ) -> Result<String> {
        let iface = crate::rg_network::ensure_bridge(resource_group_id, resource_group_cidr)
            .await
            .context("Failed to ensure resource group bridge")?;

        if let Some(cidr) = resource_group_cidr {
            self.dns_supervisor
                .ensure_running(resource_group_id, cidr)
                .await
                .context("Failed to ensure resource group dns process")?;

            let listen_port = rg_wireguard_port(resource_group_id);
            let wg_iface = crate::wireguard::ensure_interface(
                resource_group_id,
                &self.wg_private_key_b64,
                listen_port,
            )
            .await
            .context("Failed to bring up resource group WireGuard interface")?;

            crate::wireguard::set_route(&wg_iface, cidr)
                .await
                .context("Failed to route resource group CIDR over WireGuard interface")?;
        }

        Ok(iface)
    }

    async fn allocate_cid(&self) -> u32 {
        let mut next = self.next_cid.lock().await;
        let cid = *next;
        *next += 1;
        cid
    }

    pub async fn check_dns_liveness(&self) {
        self.dns_supervisor.check_liveness().await;
    }

    pub async fn teardown_rg_network(&self, resource_group_id: &str) -> Result<()> {
        self.dns_supervisor.stop(resource_group_id).await;

        crate::rg_network::teardown_bridge(resource_group_id)
            .await
            .context("Failed to tear down resource group bridge")?;

        crate::nftables::remove_bridge_rules(&crate::spec::rg_bridge_iface_name(
            resource_group_id,
        ))
        .await
        .context("Failed to remove nftables rules for resource group")?;

        let wg_iface = crate::wireguard::rg_interface_name(resource_group_id);
        crate::wireguard::remove_interface(&wg_iface)
            .await
            .context("Failed to remove resource group WireGuard interface")?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl crate::runtime::Runtime for FirecrackerRuntime {
    async fn pull_image(&self, _image: &str) -> Result<()> {
        Ok(())
    }

    async fn start_workload(&self, spec: &WorkloadSpec) -> Result<String> {
        let rootfs_builder = RootfsBuilder::new();
        let rootfs_path = rootfs_builder.ensure_rootfs(&spec.image).await?;

        let chroot_dir = PathBuf::from(JAILER_BASE_DIR).join(&spec.workload_id);
        tokio::fs::create_dir_all(&chroot_dir)
            .await
            .context("Failed to create jailer chroot directory")?;

        let vsock_cid = self.allocate_cid().await;
        let tap_device = format!("fctap{}", vsock_cid);
        let api_socket = chroot_dir
            .join("firecracker.socket")
            .to_string_lossy()
            .to_string();

        let bridge_iface = match &spec.resource_group_id {
            Some(resource_group_id) => Some(
                self.ensure_rg_network(resource_group_id, spec.resource_group_cidr.as_deref())
                    .await?,
            ),
            None => None,
        };

        let guest_network = match (&spec.resource_group_id, &spec.resource_group_cidr) {
            (Some(resource_group_id), Some(cidr)) => {
                let ip = crate::rg_ipam::allocate(resource_group_id, cidr, &spec.workload_id)
                    .await
                    .context("Failed to allocate resource group ip address")?;
                let prefix = cidr.split('/').nth(1).unwrap_or("24").to_string();
                let gateway = crate::spec::second_host_ip(cidr);
                Some(GuestNetwork { ip, prefix, gateway })
            }
            _ => None,
        };

        let jailer_uid = self
            .jailer_ids
            .allocate()
            .await
            .context("Failed to allocate jailer uid")?;

        let jailer_pid =
            match spawn_jailer(&spec.workload_id, &chroot_dir, &api_socket, jailer_uid).await {
                Ok(pid) => pid,
                Err(e) => {
                    self.jailer_ids.release(jailer_uid).await;
                    return Err(e);
                }
            };

        let metrics_path = chroot_dir.join("metrics.fifo");

        let boot_config = VmBootConfig {
            api_socket: api_socket.clone(),
            rootfs_path: rootfs_path.clone(),
            tap_device: tap_device.clone(),
            metrics_path: metrics_path.clone(),
            vsock_cid,
        };

        configure_and_boot_vm(&boot_config, bridge_iface.as_deref(), spec, guest_network.as_ref())
            .await?;

        let sidecar = VmSidecarMetadata {
            workload_id: spec.workload_id.clone(),
            vsock_cid,
            tap_device: tap_device.clone(),
            jailer_uid,
            resource_group_id: spec.resource_group_id.clone(),
        };
        write_sidecar_metadata(&chroot_dir, &sidecar)
            .await
            .context("Failed to write vm sidecar metadata")?;

        self.handles.lock().await.insert(
            spec.workload_id.clone(),
            VmHandle {
                workload_id: spec.workload_id.clone(),
                jailer_pid,
                cgroup_path: cgroup_path(&spec.workload_id),
                metrics_path,
                chroot_dir,
                tap_device,
                vsock_cid,
                resource_group_id: spec.resource_group_id.clone(),
                jailer_uid,
                previous_cpu_sample: Mutex::new(None),
            },
        );

        info!(workload_id = %spec.workload_id, vsock_cid = vsock_cid, "Firecracker microVM started");

        Ok(spec.workload_id.clone())
    }

    async fn inspect_status(&self, workload_handle: &str) -> Result<String> {
        let handles = self.handles.lock().await;
        let Some(handle) = handles.get(workload_handle) else {
            return Ok("failed".to_string());
        };

        let alive = tokio::fs::metadata(format!("/proc/{}", handle.jailer_pid))
            .await
            .is_ok();

        Ok(if alive {
            "running".to_string()
        } else {
            "failed".to_string()
        })
    }

    fn logs(&self, workload_handle: &str) -> LogStream {
        let vsock_cid = {
            let handles = match self.handles.try_lock() {
                Ok(h) => h,
                Err(_) => return Box::pin(futures_util::stream::empty()),
            };
            match handles.get(workload_handle) {
                Some(handle) => handle.vsock_cid,
                None => return Box::pin(futures_util::stream::empty()),
            }
        };

        Box::pin(vsock_log_stream(vsock_cid))
    }

    async fn exec(&self, workload_handle: &str) -> Result<ExecSession> {
        let handles = self.handles.lock().await;
        let handle = handles
            .get(workload_handle)
            .context("workload not running here")?;

        let stream =
            tokio_vsock::VsockStream::connect(tokio_vsock::VsockAddr::new(handle.vsock_cid, 10002))
                .await
                .context("Failed to connect to guest exec vsock port")?;

        let (read_half, write_half) = stream.into_split();

        Ok(ExecSession {
            input: Box::pin(write_half),
            output: Box::pin(read_half),
        })
    }

    async fn stop_workload(&self, workload_handle: &str) -> Result<()> {
        let mut handles = self.handles.lock().await;
        let Some(handle) = handles.remove(workload_handle) else {
            return Ok(());
        };

        let _ = Command::new("kill")
            .arg(handle.jailer_pid.to_string())
            .status()
            .await;

        let _ = Command::new("ip")
            .args(["link", "delete", &handle.tap_device])
            .status()
            .await;

        let _ = tokio::fs::remove_dir_all(&handle.chroot_dir).await;

        if let Some(resource_group_id) = &handle.resource_group_id {
            crate::rg_ipam::release(resource_group_id, &handle.workload_id).await;
        }

        self.jailer_ids.release(handle.jailer_uid).await;

        info!(workload_id = %handle.workload_id, "Firecracker microVM stopped");

        Ok(())
    }

    async fn stats(&self, workload_handle: &str) -> Result<crate::runtime::ContainerStats> {
        let handles = self.handles.lock().await;
        let Some(handle) = handles.get(workload_handle) else {
            return Ok(crate::runtime::ContainerStats::default());
        };

        let cpu_usage_percent = read_cpu_usage_percent(handle).await;
        let memory_usage_bytes = read_memory_usage_bytes(&handle.cgroup_path).await;
        let (network_rx_bytes, network_tx_bytes) =
            read_network_bytes(&handle.metrics_path).await.unzip();

        Ok(crate::runtime::ContainerStats {
            cpu_usage_percent,
            memory_usage_bytes,
            network_rx_bytes,
            network_tx_bytes,
        })
    }

    async fn list_managed_workloads(&self) -> Result<Vec<(String, String)>> {
        let mut entries = match tokio::fs::read_dir(JAILER_BASE_DIR).await {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e).context("Failed to read jailer base directory"),
        };

        let mut recovered = Vec::new();

        while let Some(entry) = entries
            .next_entry()
            .await
            .context("Failed to read jailer base directory entry")?
        {
            let chroot_dir = entry.path();
            let Some(workload_id) = entry.file_name().to_str().map(str::to_string) else {
                continue;
            };

            let Some(sidecar) = read_sidecar_metadata(&chroot_dir).await else {
                let _ = tokio::fs::remove_dir_all(&chroot_dir).await;
                continue;
            };

            let Some(jailer_pid) = find_live_jailer_pid(&workload_id).await else {
                let _ = tokio::fs::remove_dir_all(&chroot_dir).await;
                continue;
            };

            self.jailer_ids.mark_allocated(sidecar.jailer_uid).await;

            let metrics_path = chroot_dir.join("metrics.fifo");
            let handle = VmHandle {
                workload_id: workload_id.clone(),
                jailer_pid,
                cgroup_path: cgroup_path(&workload_id),
                metrics_path,
                chroot_dir,
                tap_device: sidecar.tap_device,
                vsock_cid: sidecar.vsock_cid,
                resource_group_id: sidecar.resource_group_id,
                jailer_uid: sidecar.jailer_uid,
                previous_cpu_sample: Mutex::new(None),
            };

            self.handles
                .lock()
                .await
                .insert(workload_id.clone(), handle);

            recovered.push((workload_id.clone(), workload_id));
        }

        Ok(recovered)
    }
}

async fn read_cpu_usage_percent(handle: &VmHandle) -> Option<f64> {
    let usage_usec = read_cgroup_u64(&handle.cgroup_path, "cpu.stat", "usage_usec").await?;
    let now = std::time::Instant::now();

    let mut previous = handle.previous_cpu_sample.lock().await;
    let percent = match *previous {
        Some((previous_time, previous_usage)) => {
            let elapsed_usec = now.duration_since(previous_time).as_micros() as u64;
            if elapsed_usec == 0 || usage_usec < previous_usage {
                None
            } else {
                let delta_usec = usage_usec - previous_usage;
                Some((delta_usec as f64 / elapsed_usec as f64) * 100.0)
            }
        }
        None => None,
    };

    *previous = Some((now, usage_usec));
    percent
}

async fn read_memory_usage_bytes(cgroup_path: &Path) -> Option<i64> {
    let content = tokio::fs::read_to_string(cgroup_path.join("memory.current"))
        .await
        .ok()?;
    content.trim().parse::<i64>().ok()
}

async fn read_cgroup_u64(cgroup_path: &Path, file: &str, key: &str) -> Option<u64> {
    let content = tokio::fs::read_to_string(cgroup_path.join(file))
        .await
        .ok()?;

    content.lines().find_map(|line| {
        let (line_key, value) = line.split_once(' ')?;
        if line_key == key {
            value.trim().parse::<u64>().ok()
        } else {
            None
        }
    })
}

async fn read_network_bytes(metrics_path: &Path) -> Option<(i64, i64)> {
    let content = tokio::fs::read_to_string(metrics_path).await.ok()?;
    let last_line = content.lines().last()?;
    let sample: serde_json::Value = serde_json::from_str(last_line).ok()?;

    let rx = sample.get("net")?.get("rx_bytes_count")?.as_i64()?;
    let tx = sample.get("net")?.get("tx_bytes_count")?.as_i64()?;

    Some((rx, tx))
}

fn vsock_log_stream(
    vsock_cid: u32,
) -> impl futures_util::Stream<Item = Result<axum::body::Bytes, std::io::Error>> {
    async_stream::stream! {
        let connect_result = tokio_vsock::VsockStream::connect(tokio_vsock::VsockAddr::new(vsock_cid, 10001)).await;

        let mut stream = match connect_result {
            Ok(s) => s,
            Err(e) => {
                yield Err(std::io::Error::other(e));
                return;
            }
        };

        let mut buf = [0u8; 4096];
        loop {
            match tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await {
                Ok(0) => break,
                Ok(n) => yield Ok(axum::body::Bytes::copy_from_slice(&buf[..n])),
                Err(e) => {
                    yield Err(e);
                    break;
                }
            }
        }
    }
}

async fn spawn_jailer(
    workload_id: &str,
    chroot_dir: &Path,
    api_socket: &str,
    jailer_uid: u32,
) -> Result<u32> {
    let uid_arg = jailer_uid.to_string();
    let child = Command::new("jailer")
        .args([
            "--id",
            workload_id,
            "--exec-file",
            "/usr/bin/firecracker",
            "--uid",
            &uid_arg,
            "--gid",
            &uid_arg,
            "--chroot-base-dir",
            chroot_dir.to_string_lossy().as_ref(),
            "--cgroup-version",
            "2",
            "--parent-cgroup",
            CGROUP_PARENT,
            "--",
            "--api-sock",
            api_socket,
        ])
        .spawn()
        .context("Failed to spawn jailer")?;

    child.id().context("jailer process has no pid")
}

fn cgroup_path(workload_id: &str) -> PathBuf {
    Path::new(CGROUP_ROOT)
        .join(CGROUP_PARENT)
        .join(workload_id)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VmSidecarMetadata {
    workload_id: String,
    vsock_cid: u32,
    tap_device: String,
    jailer_uid: u32,
    resource_group_id: Option<String>,
}

fn sidecar_metadata_path(chroot_dir: &Path) -> PathBuf {
    chroot_dir.join("csfx-meta.json")
}

async fn write_sidecar_metadata(chroot_dir: &Path, metadata: &VmSidecarMetadata) -> Result<()> {
    let payload = serde_json::to_vec(metadata).context("Failed to serialize sidecar metadata")?;
    tokio::fs::write(sidecar_metadata_path(chroot_dir), payload)
        .await
        .context("Failed to write sidecar metadata file")?;
    Ok(())
}

async fn read_sidecar_metadata(chroot_dir: &Path) -> Option<VmSidecarMetadata> {
    let content = tokio::fs::read(sidecar_metadata_path(chroot_dir))
        .await
        .ok()?;
    serde_json::from_slice(&content).ok()
}

async fn find_live_jailer_pid(workload_id: &str) -> Option<u32> {
    let mut proc_entries = tokio::fs::read_dir("/proc").await.ok()?;

    while let Ok(Some(entry)) = proc_entries.next_entry().await {
        let Some(pid) = entry.file_name().to_str().and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };

        let cmdline_path = entry.path().join("cmdline");
        let Ok(cmdline_raw) = tokio::fs::read(&cmdline_path).await else {
            continue;
        };

        let args: Vec<&str> = cmdline_raw
            .split(|b| *b == 0)
            .filter_map(|part| std::str::from_utf8(part).ok())
            .filter(|part| !part.is_empty())
            .collect();

        let is_jailer = args.first().map(|a| a.ends_with("jailer")).unwrap_or(false);
        if !is_jailer {
            continue;
        }

        let matches_id = args
            .iter()
            .zip(args.iter().skip(1))
            .any(|(flag, value)| *flag == "--id" && *value == workload_id);

        if matches_id {
            return Some(pid);
        }
    }

    None
}

struct VmBootConfig {
    api_socket: String,
    rootfs_path: PathBuf,
    tap_device: String,
    metrics_path: PathBuf,
    vsock_cid: u32,
}

async fn configure_and_boot_vm(
    boot_config: &VmBootConfig,
    bridge_iface: Option<&str>,
    spec: &WorkloadSpec,
    guest_network: Option<&GuestNetwork>,
) -> Result<()> {
    create_tap_device(&boot_config.tap_device, bridge_iface).await?;

    let client = FirecrackerApiClient::new(boot_config.api_socket.clone());

    let vcpu_count = ((spec.cpu_millicores.max(100)) as f64 / 1000.0).ceil() as i64;
    let mem_size_mib = (spec.memory_bytes / 1024 / 1024).max(128);

    client
        .put(
            "/machine-config",
            &json!({
                "vcpu_count": vcpu_count,
                "mem_size_mib": mem_size_mib,
            }),
        )
        .await?;

    client
        .put(
            "/metrics",
            &json!({
                "metrics_path": boot_config.metrics_path.to_string_lossy(),
            }),
        )
        .await
        .context("Failed to configure metrics")?;

    client
        .put(
            "/boot-source",
            &json!({
                "kernel_image_path": GUEST_KERNEL_PATH,
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off init=/sbin/csfx-guest-init",
            }),
        )
        .await?;

    client
        .put(
            "/drives/rootfs",
            &json!({
                "drive_id": "rootfs",
                "path_on_host": boot_config.rootfs_path.to_string_lossy(),
                "is_root_device": true,
                "is_read_only": false,
            }),
        )
        .await?;

    let mounted_volumes = attach_volume_drives(
        &client,
        spec.volume_mounts.as_deref().unwrap_or_default(),
    )
    .await?;

    client
        .put(
            "/network-interfaces/eth0",
            &json!({
                "iface_id": "eth0",
                "host_dev_name": boot_config.tap_device,
            }),
        )
        .await?;

    client
        .put(
            "/vsock",
            &json!({
                "guest_cid": boot_config.vsock_cid,
                "uds_path": format!("{}.vsock", boot_config.api_socket),
            }),
        )
        .await?;

    configure_mmds(&client, &mounted_volumes, guest_network).await?;

    client
        .put("/actions", &json!({ "action_type": "InstanceStart" }))
        .await?;

    Ok(())
}

struct MountedVolume {
    guest_device: String,
    mount_path: String,
}

async fn attach_volume_drives(
    client: &FirecrackerApiClient,
    volume_mounts: &[crate::spec::VolumeMount],
) -> Result<Vec<MountedVolume>> {
    let mut mounted = Vec::with_capacity(volume_mounts.len());

    for (index, volume_mount) in volume_mounts.iter().enumerate() {
        let guest_device = guest_device_letter(index);

        client
            .put(
                &format!("/drives/{}", volume_mount.volume_id),
                &json!({
                    "drive_id": volume_mount.volume_id,
                    "path_on_host": volume_mount.device_path,
                    "is_root_device": false,
                    "is_read_only": false,
                }),
            )
            .await
            .context("Failed to attach volume drive")?;

        mounted.push(MountedVolume {
            guest_device,
            mount_path: volume_mount.mount_path.clone(),
        });
    }

    Ok(mounted)
}

fn guest_device_letter(index: usize) -> String {
    let letter = (b'b' + index as u8) as char;
    format!("/dev/vd{}", letter)
}

async fn configure_mmds(
    client: &FirecrackerApiClient,
    volumes: &[MountedVolume],
    guest_network: Option<&GuestNetwork>,
) -> Result<()> {
    client
        .put(
            "/mmds/config",
            &json!({
                "version": "V1",
                "network_interfaces": ["eth0"],
            }),
        )
        .await
        .context("Failed to configure mmds")?;

    let volumes_payload: Vec<serde_json::Value> = volumes
        .iter()
        .map(|v| {
            json!({
                "device": v.guest_device,
                "mount_path": v.mount_path,
            })
        })
        .collect();

    let network_payload = guest_network.map(|net| {
        json!({
            "ip": net.ip,
            "prefix": net.prefix,
            "gateway": net.gateway,
        })
    });

    client
        .put(
            "/mmds",
            &json!({
                "volumes": volumes_payload,
                "network": network_payload,
            }),
        )
        .await
        .context("Failed to write mmds data")?;

    Ok(())
}

async fn create_tap_device(tap_device: &str, bridge_iface: Option<&str>) -> Result<()> {
    let status = Command::new("ip")
        .args(["tuntap", "add", "dev", tap_device, "mode", "tap"])
        .status()
        .await
        .context("Failed to execute ip tuntap add")?;

    if !status.success() {
        anyhow::bail!("failed to create tap device {}", tap_device);
    }

    if let Some(bridge) = bridge_iface {
        let status = Command::new("ip")
            .args(["link", "set", "dev", tap_device, "master", bridge])
            .status()
            .await
            .context("Failed to execute ip link set master")?;

        if !status.success() {
            anyhow::bail!(
                "failed to attach tap device {} to bridge {}",
                tap_device,
                bridge
            );
        }
    }

    let status = Command::new("ip")
        .args(["link", "set", "dev", tap_device, "up"])
        .status()
        .await
        .context("Failed to execute ip link set up")?;

    if !status.success() {
        anyhow::bail!("failed to bring up tap device {}", tap_device);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn sidecar_metadata_round_trips() {
        let dir = std::env::temp_dir().join(format!("csfx-sidecar-test-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();

        let metadata = VmSidecarMetadata {
            workload_id: "workload-1".to_string(),
            vsock_cid: 1234,
            tap_device: "fctap1234".to_string(),
            jailer_uid: 60001,
            resource_group_id: Some("rg-1".to_string()),
        };

        write_sidecar_metadata(&dir, &metadata).await.unwrap();
        let read_back = read_sidecar_metadata(&dir).await.unwrap();

        assert_eq!(read_back.workload_id, metadata.workload_id);
        assert_eq!(read_back.vsock_cid, metadata.vsock_cid);
        assert_eq!(read_back.tap_device, metadata.tap_device);
        assert_eq!(read_back.jailer_uid, metadata.jailer_uid);
        assert_eq!(read_back.resource_group_id, metadata.resource_group_id);

        tokio::fs::remove_dir_all(&dir).await.unwrap();
    }

    #[tokio::test]
    async fn missing_sidecar_metadata_returns_none() {
        let dir = std::env::temp_dir().join(format!("csfx-sidecar-test-{}", uuid::Uuid::new_v4()));
        assert!(read_sidecar_metadata(&dir).await.is_none());
    }
}
