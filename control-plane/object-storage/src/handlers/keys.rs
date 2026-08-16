use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde_json::json;
use uuid::Uuid;

use crate::{models::CreateAccessKeyRequest, server::AppState, services::access_key as service};

pub async fn create_key(
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
    Json(req): Json<CreateAccessKeyRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service::create_key(&state.db, &state.garage, bucket_id, req).await {
        Ok(Some(key)) => Ok((StatusCode::CREATED, Json(json!(key)))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Bucket not found"})),
        )),
        Err(e) => {
            tracing::error!(error = %e, "failed to create access key");
            Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn list_keys(
    State(state): State<AppState>,
    Path(bucket_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service::list_keys(&state.db, bucket_id).await {
        Ok(keys) => Ok(Json(json!(keys))),
        Err(e) => {
            tracing::error!(error = %e, "failed to list access keys");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn rotate_key(
    State(state): State<AppState>,
    Path((bucket_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service::rotate_key(&state.db, &state.garage, bucket_id, key_id).await {
        Ok(Some(key)) => Ok(Json(json!(key))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Access key not found"})),
        )),
        Err(e) => {
            tracing::error!(error = %e, "failed to rotate access key");
            Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": e.to_string()})),
            ))
        }
    }
}

pub async fn delete_key(
    State(state): State<AppState>,
    Path((bucket_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match service::delete_key(&state.db, &state.garage, bucket_id, key_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Access key not found"})),
        )),
        Err(e) => {
            tracing::error!(error = %e, "failed to delete access key");
            Err((
                StatusCode::BAD_GATEWAY,
                Json(json!({"error": e.to_string()})),
            ))
        }
    }
}
