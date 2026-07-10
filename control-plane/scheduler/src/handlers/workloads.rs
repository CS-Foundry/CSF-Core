use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;

use crate::{
    db,
    models::workload::{CreateWorkloadRequest, DesiredState},
    server::AppState,
};

pub async fn create_workload(
    State(state): State<AppState>,
    Json(req): Json<CreateWorkloadRequest>,
) -> impl IntoResponse {
    match state.scheduler.schedule(req).await {
        Ok(resp) => (StatusCode::CREATED, Json(serde_json::json!(resp))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

pub async fn list_workloads(State(state): State<AppState>) -> impl IntoResponse {
    match state.scheduler.list_workloads().await {
        Ok(workloads) => (StatusCode::OK, Json(serde_json::json!(workloads))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

pub async fn delete_workload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match state.scheduler.delete_workload(id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

pub async fn stop_workload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::workloads::set_desired_state(&state.db, id, DesiredState::Stopped).await {
        Ok(model) => (StatusCode::OK, Json(serde_json::json!(model))).into_response(),
        Err(sea_orm::DbErr::RecordNotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn restart_workload(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match db::workloads::request_restart(&state.db, id).await {
        Ok(model) => (StatusCode::OK, Json(serde_json::json!(model))).into_response(),
        Err(sea_orm::DbErr::RecordNotFound(_)) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}
