use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::server::AppState;

#[derive(Debug, Deserialize)]
pub struct RescheduleRequest {
    pub workload_ids: Vec<Uuid>,
}

pub async fn reschedule_agent_workloads(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<RescheduleRequest>,
) -> impl IntoResponse {
    match state
        .scheduler
        .reschedule_from_agent(agent_id, &req.workload_ids)
        .await
    {
        Ok(results) => (StatusCode::OK, Json(serde_json::json!(results))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ContainerStatusUpdate {
    pub workload_id: Uuid,
    pub container_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct BatchStatusRequest {
    pub statuses: Vec<ContainerStatusUpdate>,
}

pub async fn update_container_statuses(
    State(state): State<AppState>,
    Json(req): Json<BatchStatusRequest>,
) -> impl IntoResponse {
    for update in &req.statuses {
        if let Err(e) = crate::db::workloads::update_container_status(
            &state.db,
            update.workload_id,
            &update.container_id,
            &update.status,
        )
        .await
        {
            crate::log_warn!(
                "internal",
                &format!(
                    "Failed to update container status workload_id={} err={}",
                    update.workload_id, e
                )
            );
        } else {
            crate::log_info!(
                "internal",
                &format!(
                    "Container status updated workload_id={} status={}",
                    update.workload_id, update.status
                )
            );
        }
    }

    StatusCode::NO_CONTENT
}
