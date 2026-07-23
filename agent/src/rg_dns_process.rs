use anyhow::{Context, Result};
use std::collections::HashMap;
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::spec::first_host_ip;

pub struct RgDnsProcessSupervisor {
    processes: Mutex<HashMap<String, Child>>,
}

impl Default for RgDnsProcessSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl RgDnsProcessSupervisor {
    pub fn new() -> Self {
        Self {
            processes: Mutex::new(HashMap::new()),
        }
    }

    pub async fn ensure_running(&self, resource_group_id: &str, cidr: &str) -> Result<()> {
        let Some(dns_ip) = first_host_ip(cidr) else {
            warn!(resource_group_id = %resource_group_id, cidr = %cidr, "Invalid resource group cidr, skipping dns process");
            return Ok(());
        };

        let mut processes = self.processes.lock().await;
        if let Some(child) = processes.get_mut(resource_group_id) {
            if is_alive(child).await {
                return Ok(());
            }
            info!(resource_group_id = %resource_group_id, "Resource group dns process exited, respawning");
        }

        crate::rg_dns::write_corefile(resource_group_id, &dns_ip)
            .await
            .context("Failed to write dns corefile")?;

        let corefile = crate::rg_dns::corefile_path(resource_group_id);
        let child = Command::new("coredns")
            .arg("-conf")
            .arg(&corefile)
            .kill_on_drop(true)
            .spawn()
            .context("Failed to spawn coredns process")?;

        info!(resource_group_id = %resource_group_id, dns_ip = %dns_ip, "Resource group dns process started");

        processes.insert(resource_group_id.to_string(), child);

        Ok(())
    }

    pub async fn check_liveness(&self) {
        let mut processes = self.processes.lock().await;
        let mut dead = Vec::new();

        for (resource_group_id, child) in processes.iter_mut() {
            if !is_alive(child).await {
                dead.push(resource_group_id.clone());
            }
        }

        for resource_group_id in dead {
            warn!(resource_group_id = %resource_group_id, "Resource group dns process died, will respawn on next reconcile");
            processes.remove(&resource_group_id);
        }
    }

    pub async fn stop(&self, resource_group_id: &str) {
        let mut processes = self.processes.lock().await;
        if let Some(mut child) = processes.remove(resource_group_id) {
            let _ = child.kill().await;
        }

        if let Err(e) = crate::rg_dns::remove_zone_files(resource_group_id).await {
            warn!(resource_group_id = %resource_group_id, error = %e, "Failed to remove dns zone files");
        }
    }
}

async fn is_alive(child: &mut Child) -> bool {
    matches!(child.try_wait(), Ok(None))
}
