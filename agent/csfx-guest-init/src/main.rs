use anyhow::{Context, Result};
use nix::mount::{mount, MsFlags};
use nix::pty::openpty;
use nix::sys::reboot::{reboot, RebootMode};
use serde::Deserialize;
use std::os::fd::{AsRawFd, OwnedFd};
use std::process::Stdio;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tracing::{error, info, warn};

const READY_PORT: u32 = 10000;
const LOG_PORT: u32 = 10001;
const EXEC_PORT: u32 = 10002;
const ENTRYPOINT_PATH: &str = "/csfx-entrypoint";
const MMDS_ADDR: &str = "169.254.169.254:80";
const RESOLV_CONF_PATH: &str = "/etc/resolv.conf";

#[derive(Debug, Deserialize)]
struct MmdsVolume {
    device: String,
    mount_path: String,
}

#[derive(Debug, Deserialize)]
struct MmdsNetwork {
    ip: String,
    prefix: String,
    gateway: Option<String>,
    dns: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct MmdsData {
    #[serde(default)]
    volumes: Vec<MmdsVolume>,
    network: Option<MmdsNetwork>,
    #[serde(default)]
    env: std::collections::HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    info!("csfx-guest-init starting");

    mount_pseudo_filesystems();

    if let Err(e) = bring_up_interface("eth0") {
        warn!(error = %e, "Failed to bring up network interface");
    }

    if let Err(e) = add_host_route(std::net::Ipv4Addr::new(169, 254, 169, 254), "eth0") {
        warn!(error = %e, "Failed to add mmds route");
    }

    let mmds_data = fetch_mmds_data().await.unwrap_or_else(|e| {
        warn!(error = %e, "Failed to fetch mmds data, continuing without volumes");
        MmdsData::default()
    });

    if let Some(network) = &mmds_data.network {
        if let Err(e) = configure_network("eth0", network) {
            error!(error = %e, "Failed to configure guest network");
        }
    }

    for volume in &mmds_data.volumes {
        if let Err(e) = mount_volume(volume) {
            error!(device = %volume.device, mount_path = %volume.mount_path, error = %e, "Failed to mount volume");
        }
    }

    let log_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, LOG_PORT))
        .context("Failed to bind vsock log port")?;
    let exec_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, EXEC_PORT))
        .context("Failed to bind vsock exec port")?;

    let mut child = spawn_entrypoint(&mmds_data.env).await?;
    let stdout = child.stdout.take().context("child has no stdout")?;
    let stderr = child.stderr.take().context("child has no stderr")?;

    tokio::spawn(stream_logs(log_listener, stdout, stderr));
    tokio::spawn(serve_exec(exec_listener));

    signal_ready().await;

    let status = child.wait().await.context("Failed to wait on entrypoint")?;
    info!(code = ?status.code(), "entrypoint exited");

    power_off();
}

fn bring_up_interface(name: &str) -> Result<()> {
    let socket_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket_fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut request: libc::ifreq = unsafe { std::mem::zeroed() };
    for (dest, src) in request.ifr_name.iter_mut().zip(name.as_bytes()) {
        *dest = *src as libc::c_char;
    }

    let result = unsafe {
        if libc::ioctl(socket_fd, libc::SIOCGIFFLAGS as _, &mut request) < 0 {
            libc::close(socket_fd);
            return Err(std::io::Error::last_os_error().into());
        }

        request.ifr_ifru.ifru_flags |= (libc::IFF_UP | libc::IFF_RUNNING) as libc::c_short;

        let set_result = libc::ioctl(socket_fd, libc::SIOCSIFFLAGS as _, &request);
        libc::close(socket_fd);
        set_result
    };

    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

fn configure_network(iface: &str, network: &MmdsNetwork) -> Result<()> {
    let ip: std::net::Ipv4Addr = network.ip.parse().context("invalid guest ip")?;
    let prefix: u32 = network.prefix.parse().context("invalid guest prefix")?;
    let netmask = prefix_to_netmask(prefix)?;

    let socket_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket_fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let result = (|| -> Result<()> {
        set_ifreq_addr(socket_fd, iface, libc::SIOCSIFADDR, ip)?;
        set_ifreq_addr(socket_fd, iface, libc::SIOCSIFNETMASK, netmask)?;
        Ok(())
    })();

    unsafe {
        libc::close(socket_fd);
    }
    result?;

    if let Some(gateway) = &network.gateway {
        let gateway: std::net::Ipv4Addr = gateway.parse().context("invalid gateway ip")?;
        add_default_route(gateway)?;
    }

    if let Some(dns) = &network.dns {
        write_resolv_conf(dns)?;
    }

    info!(iface = %iface, ip = %ip, prefix = prefix, "Guest network configured");
    Ok(())
}

fn write_resolv_conf(dns: &str) -> Result<()> {
    std::fs::write(RESOLV_CONF_PATH, format!("nameserver {}\n", dns))
        .context("Failed to write resolv.conf")
}

fn prefix_to_netmask(prefix: u32) -> Result<std::net::Ipv4Addr> {
    if prefix > 32 {
        anyhow::bail!("invalid prefix length {}", prefix);
    }
    let bits: u32 = if prefix == 0 { 0 } else { !0u32 << (32 - prefix) };
    Ok(std::net::Ipv4Addr::from(bits))
}

fn set_ifreq_addr(
    socket_fd: libc::c_int,
    iface: &str,
    request_code: libc::c_ulong,
    addr: std::net::Ipv4Addr,
) -> Result<()> {
    let mut request: libc::ifreq = unsafe { std::mem::zeroed() };
    for (dest, src) in request.ifr_name.iter_mut().zip(iface.as_bytes()) {
        *dest = *src as libc::c_char;
    }

    let sockaddr = ipv4_to_sockaddr(addr);
    unsafe {
        std::ptr::copy_nonoverlapping(
            &sockaddr as *const libc::sockaddr_in as *const u8,
            &mut request.ifr_ifru.ifru_addr as *mut libc::sockaddr as *mut u8,
            std::mem::size_of::<libc::sockaddr_in>(),
        );

        if libc::ioctl(socket_fd, request_code as _, &request) < 0 {
            return Err(std::io::Error::last_os_error().into());
        }
    }

    Ok(())
}

fn add_host_route(dest: std::net::Ipv4Addr, iface: &str) -> Result<()> {
    let socket_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket_fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut iface_name = [0i8; libc::IFNAMSIZ];
    for (dst, src) in iface_name.iter_mut().zip(iface.as_bytes()) {
        *dst = *src as libc::c_char;
    }

    let mut route: libc::rtentry = unsafe { std::mem::zeroed() };
    route.rt_dst = ipv4_to_sockaddr_storage(dest);
    route.rt_genmask = ipv4_to_sockaddr_storage(std::net::Ipv4Addr::new(255, 255, 255, 255));
    route.rt_flags = (libc::RTF_UP | libc::RTF_HOST) as libc::c_ushort;
    route.rt_dev = iface_name.as_mut_ptr();

    let result = unsafe { libc::ioctl(socket_fd, libc::SIOCADDRT as _, &route) };
    unsafe {
        libc::close(socket_fd);
    }

    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

fn add_default_route(gateway: std::net::Ipv4Addr) -> Result<()> {
    let socket_fd = unsafe { libc::socket(libc::AF_INET, libc::SOCK_DGRAM, 0) };
    if socket_fd < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    let mut route: libc::rtentry = unsafe { std::mem::zeroed() };
    route.rt_dst = ipv4_to_sockaddr_storage(std::net::Ipv4Addr::UNSPECIFIED);
    route.rt_genmask = ipv4_to_sockaddr_storage(std::net::Ipv4Addr::UNSPECIFIED);
    route.rt_gateway = ipv4_to_sockaddr_storage(gateway);
    route.rt_flags = (libc::RTF_UP | libc::RTF_GATEWAY) as libc::c_ushort;

    let result = unsafe { libc::ioctl(socket_fd, libc::SIOCADDRT as _, &route) };
    unsafe {
        libc::close(socket_fd);
    }

    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    }

    Ok(())
}

fn ipv4_to_sockaddr(addr: std::net::Ipv4Addr) -> libc::sockaddr_in {
    libc::sockaddr_in {
        sin_family: libc::AF_INET as libc::sa_family_t,
        sin_port: 0,
        sin_addr: libc::in_addr {
            s_addr: u32::from_be_bytes(addr.octets()),
        },
        sin_zero: [0; 8],
    }
}

fn ipv4_to_sockaddr_storage(addr: std::net::Ipv4Addr) -> libc::sockaddr {
    let sockaddr_in = ipv4_to_sockaddr(addr);
    let mut sockaddr: libc::sockaddr = unsafe { std::mem::zeroed() };
    unsafe {
        std::ptr::copy_nonoverlapping(
            &sockaddr_in as *const libc::sockaddr_in as *const u8,
            &mut sockaddr as *mut libc::sockaddr as *mut u8,
            std::mem::size_of::<libc::sockaddr_in>(),
        );
    }
    sockaddr
}

async fn fetch_mmds_data() -> Result<MmdsData> {
    let mut stream = TcpStream::connect(MMDS_ADDR)
        .await
        .context("Failed to connect to mmds")?;

    let request = "GET /latest/meta-data/ HTTP/1.1\r\nHost: 169.254.169.254\r\nAccept: application/json\r\nConnection: close\r\n\r\n";
    stream
        .write_all(request.as_bytes())
        .await
        .context("Failed to write mmds request")?;

    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .await
        .context("Failed to read mmds response")?;

    let response_text = String::from_utf8_lossy(&response);
    let body = response_text
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .context("Malformed mmds response")?;

    serde_json::from_str(body).context("Failed to parse mmds json")
}

fn mount_volume(volume: &MmdsVolume) -> Result<()> {
    std::fs::create_dir_all(&volume.mount_path)
        .with_context(|| format!("Failed to create mount path {}", volume.mount_path))?;

    for fstype in ["ext4", "xfs", "btrfs"] {
        if mount(
            Some(volume.device.as_str()),
            volume.mount_path.as_str(),
            Some(fstype),
            MsFlags::empty(),
            None::<&str>,
        )
        .is_ok()
        {
            info!(device = %volume.device, mount_path = %volume.mount_path, fstype = %fstype, "Volume mounted");
            return Ok(());
        }
    }

    anyhow::bail!("failed to mount device {} with any known filesystem type", volume.device)
}

fn mount_pseudo_filesystems() {
    let mounts: &[(&str, &str, &str, MsFlags)] = &[
        ("proc", "/proc", "proc", MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC),
        ("sysfs", "/sys", "sysfs", MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC),
        ("devtmpfs", "/dev", "devtmpfs", MsFlags::MS_NOSUID),
    ];

    for (source, target, fstype, flags) in mounts {
        if let Err(e) = mount(Some(*source), *target, Some(*fstype), *flags, None::<&str>) {
            warn!(target = %target, error = %e, "Failed to mount pseudo filesystem");
        }
    }
}

fn power_off() -> ! {
    info!("powering off");
    match reboot(RebootMode::RB_POWER_OFF) {
        Ok(_) => unreachable!("reboot syscall does not return on success"),
        Err(e) => {
            error!(error = %e, "power off failed");
            std::process::exit(1);
        }
    }
}

async fn spawn_entrypoint(
    env: &std::collections::HashMap<String, String>,
) -> Result<tokio::process::Child> {
    Command::new(ENTRYPOINT_PATH)
        .envs(env)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn entrypoint")
}

async fn signal_ready() {
    match VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, READY_PORT)) {
        Ok(listener) => {
            if let Ok((mut stream, _)) = listener.accept().await {
                let _ = stream.write_all(b"ready\n").await;
            }
        }
        Err(e) => warn!(error = %e, "Failed to bind ready port"),
    }
}

async fn stream_logs(
    listener: VsockListener,
    mut stdout: tokio::process::ChildStdout,
    mut stderr: tokio::process::ChildStderr,
) {
    let (mut stream, _) = match listener.accept().await {
        Ok(conn) => conn,
        Err(e) => {
            error!(error = %e, "Failed to accept log connection");
            return;
        }
    };

    let mut stdout_buf = [0u8; 4096];
    let mut stderr_buf = [0u8; 4096];

    loop {
        tokio::select! {
            result = stdout.read(&mut stdout_buf) => {
                match result {
                    Ok(0) => break,
                    Ok(n) => {
                        if stream.write_all(&stdout_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            result = stderr.read(&mut stderr_buf) => {
                match result {
                    Ok(0) => {}
                    Ok(n) => {
                        if stream.write_all(&stderr_buf[..n]).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    }
}

async fn serve_exec(listener: VsockListener) {
    loop {
        let (stream, _) = match listener.accept().await {
            Ok(conn) => conn,
            Err(e) => {
                error!(error = %e, "Failed to accept exec connection");
                continue;
            }
        };

        tokio::spawn(handle_exec_session(stream));
    }
}

async fn handle_exec_session(stream: tokio_vsock::VsockStream) {
    let pty = match openpty(None, None) {
        Ok(pty) => pty,
        Err(e) => {
            error!(error = %e, "Failed to open pty");
            return;
        }
    };

    let mut child = match spawn_shell_on_pty(&pty) {
        Ok(child) => child,
        Err(e) => {
            error!(error = %e, "Failed to spawn exec shell");
            return;
        }
    };

    drop(pty.slave);

    if let Err(e) = set_nonblocking(&pty.master) {
        error!(error = %e, "Failed to set pty master nonblocking");
        let _ = child.kill().await;
        return;
    }

    match AsyncFd::new(pty.master) {
        Ok(master) => pipe_pty_to_vsock(master, stream, child).await,
        Err(e) => {
            error!(error = %e, "Failed to register pty master with async runtime");
            let _ = child.kill().await;
        }
    }
}

fn set_nonblocking(fd: &OwnedFd) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
    if flags < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let result = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) };
    if result < 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok(())
}

fn spawn_shell_on_pty(pty: &nix::pty::OpenptyResult) -> Result<tokio::process::Child> {
    let slave_fd = pty.slave.as_raw_fd();

    let mut command = Command::new("/bin/sh");
    unsafe {
        command.pre_exec(move || {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::ioctl(slave_fd, libc::TIOCSCTTY as _, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            for target_fd in 0..3 {
                if libc::dup2(slave_fd, target_fd) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }

    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn shell")
}

async fn pipe_pty_to_vsock(
    master: AsyncFd<OwnedFd>,
    stream: tokio_vsock::VsockStream,
    mut child: tokio::process::Child,
) {
    let (mut vsock_read, mut vsock_write) = stream.into_split();

    let output_task = async {
        let mut buf = [0u8; 4096];
        loop {
            match read_pty(&master, &mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if vsock_write.write_all(&buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    let input_task = async {
        let mut buf = [0u8; 4096];
        loop {
            match vsock_read.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if write_pty(&master, &buf[..n]).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    };

    tokio::select! {
        _ = output_task => {}
        _ = input_task => {}
    }

    let _ = child.kill().await;
}

async fn read_pty(master: &AsyncFd<OwnedFd>, buf: &mut [u8]) -> std::io::Result<usize> {
    loop {
        let mut guard = master.readable().await?;
        match guard.try_io(|fd| {
            let n = unsafe {
                libc::read(fd.as_raw_fd(), buf.as_mut_ptr() as *mut libc::c_void, buf.len())
            };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }) {
            Ok(result) => return result,
            Err(_would_block) => continue,
        }
    }
}

async fn write_pty(master: &AsyncFd<OwnedFd>, buf: &[u8]) -> std::io::Result<()> {
    let mut written = 0;
    while written < buf.len() {
        let mut guard = master.writable().await?;
        match guard.try_io(|fd| {
            let n = unsafe {
                libc::write(
                    fd.as_raw_fd(),
                    buf[written..].as_ptr() as *const libc::c_void,
                    buf.len() - written,
                )
            };
            if n < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(n as usize)
            }
        }) {
            Ok(Ok(n)) => written += n,
            Ok(Err(e)) => return Err(e),
            Err(_would_block) => continue,
        }
    }
    Ok(())
}
