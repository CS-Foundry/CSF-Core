use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};

use crate::{models::compose::CreateStackRequest, server::AppState};

pub async fn create_stack(
    State(state): State<AppState>,
    Json(req): Json<CreateStackRequest>,
) -> impl IntoResponse {
    match state.scheduler.schedule_stack(req).await {
        Ok(resp) => (StatusCode::CREATED, Json(serde_json::json!(resp))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}
