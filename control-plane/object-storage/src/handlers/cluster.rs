use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;

use crate::{
    db::garage_nodes, garage::layout::replication_factor_for, models::ClusterStatusResponse,
    server::AppState,
};

pub async fn get_cluster_status(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let nodes = garage_nodes::list(&state.db).await.map_err(|e| {
        tracing::error!(error = %e, "failed to list garage nodes");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
    })?;

    let storage_node_count = nodes
        .iter()
        .filter(|n| n.role == "storage" && n.status == "up")
        .count();

    let replication_factor = replication_factor_for(storage_node_count);
    let degraded = storage_node_count == 0 || (storage_node_count as u32) < replication_factor;

    Ok(Json(json!(ClusterStatusResponse {
        storage_node_count: storage_node_count as u32,
        replication_factor,
        degraded,
        nodes,
    })))
}
