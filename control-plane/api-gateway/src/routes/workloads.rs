use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post},
    Router,
};
use serde_json::json;

use crate::{
    auth::rbac::{CanManageWorkloads, CanViewWorkloads},
    AppState,
};

async fn proxy_to_scheduler(
    state: &AppState,
    method: reqwest::Method,
    path: &str,
    body: Option<serde_json::Value>,
    headers: Option<Vec<(String, String)>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match state
        .service_client
        .forward_to_scheduler(method, path, body, headers)
        .await
    {
        Ok((status, Some(body))) => {
            let axum_status =
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Ok((axum_status, Json(body)).into_response())
        }
        Ok((status, None)) => {
            let axum_status =
                StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            Ok((axum_status, Body::empty()).into_response())
        }
        Err(e) => {
            tracing::error!("Failed to forward request to scheduler: {}", e);
            Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": "Scheduler service unavailable", "details": e.to_string() })),
            ))
        }
    }
}

pub async fn create_workload(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let body_json: Option<serde_json::Value> = serde_json::from_str(&body).ok();
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::POST,
        "/workloads",
        body_json,
        Some(header_map),
    )
    .await
}

pub async fn list_workloads(
    CanViewWorkloads(_claims): CanViewWorkloads,
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::GET,
        "/workloads",
        None,
        Some(header_map),
    )
    .await
}

pub async fn delete_workload(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::DELETE,
        &format!("/workloads/{}", id),
        None,
        Some(header_map),
    )
    .await
}

pub async fn update_workload(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let body_json: Option<serde_json::Value> = serde_json::from_str(&body).ok();
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::PATCH,
        &format!("/workloads/{}", id),
        body_json,
        Some(header_map),
    )
    .await
}

pub async fn stop_workload(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::POST,
        &format!("/workloads/{}/stop", id),
        None,
        Some(header_map),
    )
    .await
}

pub async fn restart_workload(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::POST,
        &format!("/workloads/{}/restart", id),
        None,
        Some(header_map),
    )
    .await
}

pub async fn create_stack(
    CanManageWorkloads(_claims): CanManageWorkloads,
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let body_json: Option<serde_json::Value> = serde_json::from_str(&body).ok();
    let header_map = header_vec(&headers);
    proxy_to_scheduler(
        &state,
        reqwest::Method::POST,
        "/workload-stacks",
        body_json,
        Some(header_map),
    )
    .await
}

fn header_vec(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|val| (k.to_string(), val.to_string())))
        .collect()
}

pub fn workloads_routes() -> Router<AppState> {
    Router::new()
        .route("/workloads", post(create_workload))
        .route("/workloads", get(list_workloads))
        .route("/workloads/{id}", delete(delete_workload))
        .route("/workloads/{id}", patch(update_workload))
        .route("/workloads/{id}/stop", post(stop_workload))
        .route("/workloads/{id}/restart", post(restart_workload))
        .route("/workload-stacks", post(create_stack))
}
