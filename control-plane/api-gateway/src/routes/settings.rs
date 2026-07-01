use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use entity::{entities::system_settings, SystemSettings};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{auth::rbac::CanManageLogs, AppState};

const RETENTION_KEY: &str = "logs.retention_days";
const MIN_RETENTION_DAYS: i64 = 1;
const MAX_RETENTION_DAYS: i64 = 365;

#[derive(Debug, Serialize)]
pub struct LogsRetentionResponse {
    pub retention_days: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLogsRetentionRequest {
    pub retention_days: i64,
}

pub async fn get_logs_retention(
    CanManageLogs(_claims): CanManageLogs,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let retention_days = load_retention_days(&state).await?;
    Ok((
        StatusCode::OK,
        Json(json!(LogsRetentionResponse { retention_days })),
    ))
}

pub async fn update_logs_retention(
    CanManageLogs(_claims): CanManageLogs,
    State(state): State<AppState>,
    Json(req): Json<UpdateLogsRetentionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if req.retention_days < MIN_RETENTION_DAYS || req.retention_days > MAX_RETENTION_DAYS {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!(
                "retention_days must be between {} and {}",
                MIN_RETENTION_DAYS, MAX_RETENTION_DAYS
            ) })),
        ));
    }

    let existing = SystemSettings::find_by_id(RETENTION_KEY)
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load logs retention setting");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    let now = chrono::Utc::now().into();
    let model = match existing {
        Some(setting) => {
            let mut active: system_settings::ActiveModel = setting.into();
            active.value = Set(json!(req.retention_days));
            active.updated_at = Set(now);
            active
        }
        None => system_settings::ActiveModel {
            key: Set(RETENTION_KEY.to_string()),
            value: Set(json!(req.retention_days)),
            updated_at: Set(now),
        },
    };

    model.save(&state.db_conn).await.map_err(|e| {
        tracing::error!(error = %e, "failed to save logs retention setting");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "database error" })),
        )
    })?;

    Ok((
        StatusCode::OK,
        Json(json!(LogsRetentionResponse {
            retention_days: req.retention_days
        })),
    ))
}

pub async fn load_retention_days(
    state: &AppState,
) -> Result<i64, (StatusCode, Json<serde_json::Value>)> {
    let setting = SystemSettings::find_by_id(RETENTION_KEY)
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to load logs retention setting");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    Ok(setting
        .and_then(|s| s.value.as_i64())
        .unwrap_or(MAX_RETENTION_DAYS.min(30)))
}

pub fn settings_routes() -> Router<AppState> {
    Router::new().route(
        "/admin/settings/logs-retention",
        get(get_logs_retention).put(update_logs_retention),
    )
}
