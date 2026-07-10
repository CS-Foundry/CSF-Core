use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::sync::Mutex;
use tracing::info;

use crate::docker::WorkloadSpec;
use crate::firecracker::api::FirecrackerApiClient;
use crate::firecracker::rootfs::RootfsBuilder;
use crate::runtime::{ExecSession, LogStream};

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
    docker: bollard::Docker,
    handles: Mutex<HashMap<String, VmHandle>>,
    next_cid: Mutex<u32>,
}

impl FirecrackerRuntime {
    pub fn new(docker: bollard::Docker) -> Self {
        Self {
            docker,
            handles: Mutex::new(HashMap::new()),
            next_cid: Mutex::new(GUEST_CID_BASE),
        }
    }

    async fn allocate_cid(&self) -> u32 {
        let mut next = self.next_cid.lock().await;
        let cid = *next;
        *next += 1;
        cid
    }
}

#[async_trait::async_trait]
impl crate::runtime::Runtime for FirecrackerRuntime {
    async fn pull_image(&self, image: &str) -> Result<()> {
        let options = bollard::query_parameters::CreateImageOptionsBuilder::default()
            .from_image(image)
            .build();
        let mut stream = self.docker.create_image(Some(options), None, None);
        use futures_util::StreamExt;
        while let Some(item) = stream.next().await {
            item.context("Failed to pull image for rootfs build")?;
        }
        Ok(())
    }

    async fn start_workload(&self, spec: &WorkloadSpec) -> Result<String> {
        let rootfs_builder = RootfsBuilder::new(&self.docker);
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

        let jailer_pid = spawn_jailer(&spec.workload_id, &chroot_dir, &api_socket).await?;

        configure_and_boot_vm(
            &api_socket,
            &rootfs_path,
            &tap_device,
            vsock_cid,
            spec.cpu_millicores,
            spec.memory_bytes,
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

        let stream = tokio_vsock::VsockStream::connect(tokio_vsock::VsockAddr::new(
            handle.vsock_cid,
            10002,
        ))
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
}

fn vsock_log_stream(vsock_cid: u32) -> impl futures_util::Stream<Item = Result<axum::body::Bytes, std::io::Error>> {
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

async fn spawn_jailer(workload_id: &str, chroot_dir: &PathBuf, api_socket: &str) -> Result<u32> {
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
    vsock_cid: u32,
    cpu_millicores: i32,
    memory_bytes: i64,
) -> Result<()> {
    create_tap_device(tap_device).await?;

    let client = FirecrackerApiClient::new(api_socket.to_string());

    let vcpu_count = ((cpu_millicores.max(100)) as f64 / 1000.0).ceil() as i64;
    let mem_size_mib = (memory_bytes / 1024 / 1024).max(128);

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
                "boot_args": "console=ttyS0 reboot=k panic=1 pci=off",
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

    client
        .put("/actions", &json!({ "action_type": "InstanceStart" }))
        .await?;

    Ok(())
}

async fn create_tap_device(tap_device: &str) -> Result<()> {
    let status = Command::new("ip")
        .args(["tuntap", "add", "dev", tap_device, "mode", "tap"])
        .status()
        .await
        .context("Failed to execute ip tuntap add")?;

    if !status.success() {
        anyhow::bail!("failed to create tap device {}", tap_device);
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
