use axum::{
    body::Body,
    extract::{OriginalUri, Path, State},
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
    require_external: bool,
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

    if require_external && bucket.exposure != "external" {
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

    let Some(agent_id) = node.agent_id else {
        return Ok(std::env::var("GARAGE_INTERNAL_HOST").unwrap_or_else(|_| "garage".to_string()));
    };

    let agent = agents::Entity::find_by_id(agent_id)
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

async fn proxy(
    state: &AppState,
    bucket: &str,
    raw_path_and_query: &str,
    require_external: bool,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let tunnel_ip = resolve_bucket_target(state, bucket, require_external).await?;

    let url = format!("http://{}:{}{}", tunnel_ip, S3_PORT, raw_path_and_query);

    let reqwest_method =
        reqwest::Method::from_bytes(method.as_str().as_bytes()).unwrap_or(reqwest::Method::GET);

    let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
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

pub async fn proxy_s3_request(
    State(state): State<AppState>,
    Path((bucket, _path)): Path<(String, String)>,
    OriginalUri(uri): OriginalUri,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let raw = uri
        .path()
        .strip_prefix("/api/s3")
        .unwrap_or_else(|| uri.path());
    let raw_path_and_query = match uri.query() {
        Some(query) => format!("{}?{}", raw, query),
        None => raw.to_string(),
    };

    proxy(
        &state,
        &bucket,
        &raw_path_and_query,
        true,
        method,
        headers,
        body,
    )
    .await
}

pub async fn proxy_object_data(
    State(state): State<AppState>,
    Path((bucket, _key)): Path<(String, String)>,
    OriginalUri(uri): OriginalUri,
    method: Method,
    headers: HeaderMap,
    body: Body,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let raw = uri.path().strip_prefix("/s3data").unwrap_or_else(|| uri.path());
    let raw_path_and_query = match uri.query() {
        Some(query) => format!("{}?{}", raw, query),
        None => raw.to_string(),
    };

    proxy(
        &state,
        &bucket,
        &raw_path_and_query,
        false,
        method,
        headers,
        body,
    )
    .await
}

pub fn s3_proxy_routes() -> Router<AppState> {
    Router::new().route("/s3/{bucket}/{*path}", any(proxy_s3_request))
}

pub fn object_data_router() -> Router<AppState> {
    Router::new().route("/s3data/{bucket}/{*key}", any(proxy_object_data))
}
