use anyhow::{Context, Result};
use etcd_client::{Client, Compare, CompareOp, PutOptions, Txn, TxnOp};
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::{log_error, log_info, log_warn};

const LAYOUT_LOCK_KEY: &str = "/csfx/object-storage/layout-lock";
const LEASE_TTL_SECONDS: i64 = 10;

#[derive(Clone)]
pub struct LayoutLeader {
    etcd: Client,
    node_id: String,
    lease_id: Arc<AtomicI64>,
    is_leader: Arc<AtomicBool>,
}

impl LayoutLeader {
    pub fn new(etcd: Client, node_id: String) -> Self {
        Self {
            etcd,
            node_id,
            lease_id: Arc::new(AtomicI64::new(0)),
            is_leader: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn is_leader(&self) -> bool {
        self.is_leader.load(Ordering::SeqCst)
    }

    async fn campaign(&mut self) -> Result<()> {
        if self.is_leader() {
            return Ok(());
        }

        let lease = self
            .etcd
            .lease_grant(LEASE_TTL_SECONDS, None)
            .await
            .context("failed to grant etcd lease")?;
        let lease_id = lease.id();

        let txn = Txn::new()
            .when(vec![Compare::create_revision(LAYOUT_LOCK_KEY, CompareOp::Equal, 0)])
            .and_then(vec![TxnOp::put(
                LAYOUT_LOCK_KEY,
                self.node_id.as_bytes(),
                Some(PutOptions::new().with_lease(lease_id)),
            )]);

        let response = self.etcd.txn(txn).await.context("etcd txn failed")?;

        if response.succeeded() {
            self.lease_id.store(lease_id, Ordering::SeqCst);
            self.is_leader.store(true, Ordering::SeqCst);
            log_info!("garage::leader", &format!("became object-storage layout leader node_id={}", self.node_id));
            self.spawn_lease_renewal(lease_id);
        } else {
            let _ = self.etcd.lease_revoke(lease_id).await;
        }

        Ok(())
    }

    fn spawn_lease_renewal(&self, lease_id: i64) {
        let mut etcd = self.etcd.clone();
        let is_leader = Arc::clone(&self.is_leader);
        let node_id = self.node_id.clone();

        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(4)).await;
                if !is_leader.load(Ordering::SeqCst) {
                    break;
                }
                if let Err(e) = etcd.lease_keep_alive(lease_id).await {
                    log_error!("garage::leader", &format!("lease renewal failed node_id={} err={}", node_id, e));
                    is_leader.store(false, Ordering::SeqCst);
                    log_warn!("garage::leader", &format!("lost object-storage layout leadership node_id={}", node_id));
                    break;
                }
            }
        });
    }

    pub async fn run_campaign_loop(mut self) {
        loop {
            if let Err(e) = self.campaign().await {
                log_error!("garage::leader", &format!("layout leader campaign failed err={}", e));
            }
            sleep(Duration::from_secs(5)).await;
        }
    }
}
