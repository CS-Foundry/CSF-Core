use anyhow::Result;
use entity::entities::{agents, garage_nodes};
use sea_orm::{DatabaseConnection, EntityTrait};
use tokio::time::{sleep, Duration};

use crate::{
    db::garage_nodes as garage_nodes_db,
    garage::{client::LayoutRole, leader::LayoutLeader, GarageClient},
    log_error, log_info, log_warn,
};

const RECONCILE_INTERVAL_SECONDS: u64 = 30;
const MIN_STORAGE_NODES_FOR_FULL_REPLICATION: usize = 3;

pub fn replication_factor_for(storage_node_count: usize) -> u32 {
    if storage_node_count >= MIN_STORAGE_NODES_FOR_FULL_REPLICATION {
        3
    } else {
        1
    }
}

async fn peer_addrs(db: &DatabaseConnection, known_nodes: &[garage_nodes::Model]) -> Result<Vec<String>> {
    let mut addrs = Vec::new();

    for node in known_nodes {
        let Some(garage_node_id) = &node.garage_node_id else {
            continue;
        };
        let Some(agent) = agents::Entity::find_by_id(node.agent_id).one(db).await? else {
            continue;
        };
        let Some(wg_ip) = agent.wg_tunnel_ip else {
            continue;
        };
        addrs.push(format!("{}@{}:3901", garage_node_id, wg_ip));
    }

    Ok(addrs)
}

async fn reconcile_once(db: &DatabaseConnection, garage: &GarageClient) -> Result<()> {
    let known_nodes = garage_nodes_db::list(db).await?;

    let addrs = peer_addrs(db, &known_nodes).await?;
    if let Err(e) = garage.connect_cluster_nodes(&addrs).await {
        log_warn!("garage::layout", &format!("failed to connect garage cluster nodes err={}", e));
    }

    let status = match garage.get_cluster_status().await {
        Ok(status) => status,
        Err(e) => {
            log_warn!("garage::layout", &format!("failed to read garage cluster status err={}", e));
            return Ok(());
        }
    };

    for node in &known_nodes {
        let is_up = status
            .nodes
            .iter()
            .any(|n| Some(n.id.clone()) == node.garage_node_id && n.is_up);

        if !is_up && node.status == "up" {
            log_warn!("garage::layout", &format!("garage node reported down agent_id={}", node.agent_id));
            garage_nodes_db::mark_down(db, node.agent_id).await?;
        }
    }

    let storage_nodes: Vec<&garage_nodes::Model> = known_nodes
        .iter()
        .filter(|n| n.role == "storage" && n.capacity_bytes.is_some())
        .collect();

    if storage_nodes.is_empty() {
        return Ok(());
    }

    let factor = replication_factor_for(storage_nodes.len());

    let roles: Vec<LayoutRole> = known_nodes
        .iter()
        .filter_map(|n| {
            n.garage_node_id.as_ref().map(|garage_id| LayoutRole {
                id: garage_id.clone(),
                zone: n.zone.clone(),
                capacity: n.capacity_bytes,
                tags: vec![],
            })
        })
        .collect();

    if roles.is_empty() {
        return Ok(());
    }

    if let Err(e) = garage.update_cluster_layout(roles, factor).await {
        log_error!("garage::layout", &format!("failed to stage cluster layout err={}", e));
        return Ok(());
    }

    if let Err(e) = garage.apply_cluster_layout(status.layout_version + 1).await {
        log_error!("garage::layout", &format!("failed to apply cluster layout err={}", e));
        return Ok(());
    }

    log_info!("garage::layout", &format!("applied cluster layout storage_nodes={} replication_factor={}", storage_nodes.len(), factor));
    Ok(())
}

pub async fn run_reconcile_loop(db: DatabaseConnection, garage: GarageClient, leader: LayoutLeader) {
    loop {
        sleep(Duration::from_secs(RECONCILE_INTERVAL_SECONDS)).await;

        if !leader.is_leader() {
            continue;
        }

        if let Err(e) = reconcile_once(&db, &garage).await {
            log_error!("garage::layout", &format!("reconcile loop iteration failed err={}", e));
        }
    }
}
