use anyhow::{Context, Result};
use nix::mount::{mount, MsFlags};
use nix::pty::openpty;
use nix::sys::reboot::{reboot, RebootMode};
use std::os::fd::{AsRawFd, OwnedFd};
use std::process::Stdio;
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio_vsock::{VsockAddr, VsockListener, VMADDR_CID_ANY};
use tracing::{error, info, warn};

const READY_PORT: u32 = 10000;
const LOG_PORT: u32 = 10001;
const EXEC_PORT: u32 = 10002;
const ENTRYPOINT_PATH: &str = "/csfx-entrypoint";

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

    let log_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, LOG_PORT))
        .context("Failed to bind vsock log port")?;
    let exec_listener = VsockListener::bind(VsockAddr::new(VMADDR_CID_ANY, EXEC_PORT))
        .context("Failed to bind vsock exec port")?;

    let mut child = spawn_entrypoint().await?;
    let stdout = child.stdout.take().context("child has no stdout")?;
    let stderr = child.stderr.take().context("child has no stderr")?;

    tokio::spawn(stream_logs(log_listener, stdout, stderr));
    tokio::spawn(serve_exec(exec_listener));

    signal_ready().await;

    let status = child.wait().await.context("Failed to wait on entrypoint")?;
    info!(code = ?status.code(), "entrypoint exited");

    power_off();
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

async fn spawn_entrypoint() -> Result<tokio::process::Child> {
    Command::new(ENTRYPOINT_PATH)
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
