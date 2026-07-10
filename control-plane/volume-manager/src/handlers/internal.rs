use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use uuid::Uuid;

use crate::server::AppState;

pub async fn detach_all_for_agent(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.volume_service.force_detach_all(agent_id).await {
        Ok(volume_ids) => (
            StatusCode::OK,
            Json(serde_json::json!({ "detached_volume_ids": volume_ids })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}
