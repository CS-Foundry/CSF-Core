use axum::{
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, Method, StatusCode},
    response::{IntoResponse, Json},
    routing::any,
    Router,
};
use entity::entities::{agents, buckets, garage_nodes};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::json;

use crate::AppState;

const S3_PORT: u16 = 3900;

async fn resolve_bucket_target(
    state: &AppState,
    global_alias: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let bucket = buckets::Entity::find()
        .filter(buckets::Column::GlobalAlias.eq(global_alias))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("database error: {}", e) })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "bucket not found" })),
            )
        })?;

    if bucket.exposure != "external" {
        return Err((
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "bucket not found" })),
        ));
    }

    let node = garage_nodes::Entity::find()
        .filter(garage_nodes::Column::Status.eq("up"))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("database error: {}", e) })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "no garage node available" })),
            )
        })?;

    let agent = agents::Entity::find_by_id(node.agent_id)
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("database error: {}", e) })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "garage node has no agent record" })),
            )
        })?;

    let tunnel_ip = agent.wg_tunnel_ip.ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "garage node has no known tunnel address" })),
        )
    })?;

    Ok(tunnel_ip)
}

pub async fn proxy_s3_request(
    State(state): State<AppState>,
    Path((bucket, path)): Path<(String, String)>,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let tunnel_ip = resolve_bucket_target(&state, &bucket).await?;

    let url = format!("http://{}:{}/{}/{}", tunnel_ip, S3_PORT, bucket, path);

    let reqwest_method = reqwest::Method::from_bytes(method.as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);

    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": format!("failed to read request body: {}", e) })),
            )
        })?;

    let client = reqwest::Client::new();
    let mut request = client.request(reqwest_method, &url).body(body_bytes);

    for (key, value) in headers.iter() {
        let key_lower = key.as_str().to_lowercase();
        if key_lower == "content-length" {
            continue;
        }
        if let Ok(value_str) = value.to_str() {
            request = request.header(key.as_str(), value_str);
        }
    }

    let response = request.send().await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("failed to reach garage node: {}", e) })),
        )
    })?;

    let status =
        StatusCode::from_u16(response.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
    let mut response_headers = HeaderMap::new();
    for (key, value) in response.headers().iter() {
        response_headers.insert(key.clone(), value.clone());
    }

    let stream = response.bytes_stream();

    Ok((status, response_headers, Body::from_stream(stream)))
}

pub fn s3_proxy_routes() -> Router<AppState> {
    Router::new().route("/s3/{bucket}/{*path}", any(proxy_s3_request))
}
