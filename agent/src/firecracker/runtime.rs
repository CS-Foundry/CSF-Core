use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::info;

use crate::firecracker::api::FirecrackerApiClient;
use crate::firecracker::rootfs::RootfsBuilder;
use crate::rg_dns_process::RgDnsProcessSupervisor;
use crate::runtime::{ExecSession, LogStream};
use crate::spec::{rg_wireguard_port, WorkloadSpec};

const JAILER_BASE_DIR: &str = "/var/lib/csfx-agent/firecracker";
const GUEST_KERNEL_PATH: &str = "/var/lib/csfx-agent/vmlinux";
const GUEST_CID_BASE: u32 = 1000;

struct VmHandle {
    workload_id: String,
    jailer_pid: u32,
    chroot_dir: PathBuf,
    tap_device: String,
    vsock_cid: u32,
}

pub struct FirecrackerRuntime {
    wg_private_key_b64: String,
    dns_supervisor: RgDnsProcessSupervisor,
    handles: Mutex<HashMap<String, VmHandle>>,
    next_cid: Mutex<u32>,
}

impl FirecrackerRuntime {
    pub fn new(wg_private_key_b64: String) -> Self {
        Self {
            wg_private_key_b64,
            dns_supervisor: RgDnsProcessSupervisor::new(),
            handles: Mutex::new(HashMap::new()),
            next_cid: Mutex::new(GUEST_CID_BASE),
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

        let jailer_pid = spawn_jailer(&spec.workload_id, &chroot_dir, &api_socket).await?;

        configure_and_boot_vm(
            &api_socket,
            &rootfs_path,
            &tap_device,
            bridge_iface.as_deref(),
            vsock_cid,
            spec,
        )
        .await?;

        self.handles.lock().await.insert(
            spec.workload_id.clone(),
            VmHandle {
                workload_id: spec.workload_id.clone(),
                jailer_pid,
                chroot_dir,
                tap_device,
                vsock_cid,
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

        info!(workload_id = %handle.workload_id, "Firecracker microVM stopped");

        Ok(())
    }

    async fn stats(&self, _workload_handle: &str) -> Result<crate::runtime::ContainerStats> {
        Ok(crate::runtime::ContainerStats::default())
    }
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

async fn spawn_jailer(workload_id: &str, chroot_dir: &Path, api_socket: &str) -> Result<u32> {
    let child = Command::new("jailer")
        .args([
            "--id",
            workload_id,
            "--exec-file",
            "/usr/bin/firecracker",
            "--uid",
            "0",
            "--gid",
            "0",
            "--chroot-base-dir",
            chroot_dir.to_string_lossy().as_ref(),
            "--",
            "--api-sock",
            api_socket,
        ])
        .spawn()
        .context("Failed to spawn jailer")?;

    child.id().context("jailer process has no pid")
}

async fn configure_and_boot_vm(
    api_socket: &str,
    rootfs_path: &std::path::Path,
    tap_device: &str,
    bridge_iface: Option<&str>,
    vsock_cid: u32,
    spec: &WorkloadSpec,
) -> Result<()> {
    create_tap_device(tap_device, bridge_iface).await?;

    let client = FirecrackerApiClient::new(api_socket.to_string());

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
                "path_on_host": rootfs_path.to_string_lossy(),
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
                "host_dev_name": tap_device,
            }),
        )
        .await?;

    client
        .put(
            "/vsock",
            &json!({
                "guest_cid": vsock_cid,
                "uds_path": format!("{}.vsock", api_socket),
            }),
        )
        .await?;

    configure_mmds(&client, &mounted_volumes).await?;

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

async fn configure_mmds(client: &FirecrackerApiClient, volumes: &[MountedVolume]) -> Result<()> {
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

    client
        .put(
            "/mmds",
            &json!({
                "volumes": volumes_payload,
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
