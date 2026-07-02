use axum::{
    body::Body,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use entity::entities::{agents, workloads};
use futures_util::{SinkExt, StreamExt};
use sea_orm::EntityTrait;
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use uuid::Uuid;

use crate::{
    auth::jwt::{create_exec_ticket, verify_exec_ticket},
    auth::rbac::{CanManageWorkloads, CanViewWorkloads},
    AppState,
};

const CSFX_AGENT_PORT_ENV: &str = "CSFX_AGENT_PORT";

async fn resolve_agent_target(
    state: &AppState,
    workload_id: Uuid,
) -> Result<(String, Uuid), (StatusCode, Json<serde_json::Value>)> {
    let workload = workloads::Entity::find_by_id(workload_id)
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
                Json(json!({ "error": "workload not found" })),
            )
        })?;

    let agent_id = workload.assigned_agent_id.ok_or_else(|| {
        (
            StatusCode::CONFLICT,
            Json(json!({ "error": "workload has no assigned agent" })),
        )
    })?;

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
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "agent not found" })),
            )
        })?;

    let tunnel_ip = agent.wg_tunnel_ip.ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "agent has no known tunnel address" })),
        )
    })?;

    Ok((tunnel_ip, agent_id))
}

async fn fetch_proxy_ticket(
    state: &AppState,
    agent_id: Uuid,
    workload_id: Uuid,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let (status, body) = state
        .service_client
        .forward_to_registry(
            reqwest::Method::POST,
            "/internal/agent-proxy-ticket",
            Some(json!({ "agent_id": agent_id, "workload_id": workload_id })),
            None,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("failed to reach registry: {}", e) })),
            )
        })?;

    if !status.is_success() {
        return Err((
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "registry refused to issue proxy ticket" })),
        ));
    }

    let body = body.ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "registry returned empty proxy ticket response" })),
        )
    })?;

    let payload = body["payload"].as_str().ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "malformed proxy ticket payload" })),
        )
    })?;
    let signature = body["signature"].as_str().ok_or_else(|| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": "malformed proxy ticket signature" })),
        )
    })?;

    Ok(format!("{}.{}", payload, signature))
}

pub async fn stream_workload_logs(
    CanViewWorkloads(_claims): CanViewWorkloads,
    State(state): State<AppState>,
    Path(workload_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let (tunnel_ip, agent_id) = resolve_agent_target(&state, workload_id).await?;
    let ticket = fetch_proxy_ticket(&state, agent_id, workload_id).await?;

    let agent_port: u16 = std::env::var(CSFX_AGENT_PORT_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7443);

    let url = format!("http://{}:{}/logs/{}", tunnel_ip, agent_port, workload_id);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .header("X-Csfx-Proxy-Ticket", ticket)
        .send()
        .await
        .map_err(|e| {
            (
                StatusCode::BAD_GATEWAY,
                Json(json!({ "error": format!("failed to reach agent: {}", e) })),
            )
        })?;

    if !resp.status().is_success() {
        let status =
            StatusCode::from_u16(resp.status().as_u16()).unwrap_or(StatusCode::BAD_GATEWAY);
        return Err((
            status,
            Json(json!({ "error": "agent refused log stream request" })),
        ));
    }

    let stream = resp.bytes_stream();
    Ok(Body::from_stream(stream))
}

pub async fn issue_exec_ticket(
    CanManageWorkloads(claims): CanManageWorkloads,
    Path(workload_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let ticket = create_exec_ticket(workload_id, claims.user_id).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("failed to issue exec ticket: {}", e) })),
        )
    })?;

    Ok(Json(json!({ "ticket": ticket })))
}

#[derive(Deserialize)]
pub struct ExecQuery {
    ticket: String,
}

pub async fn exec_workload(
    State(state): State<AppState>,
    Path(workload_id): Path<Uuid>,
    Query(query): Query<ExecQuery>,
    ws: WebSocketUpgrade,
) -> Result<axum::response::Response, (StatusCode, Json<serde_json::Value>)> {
    verify_exec_ticket(&query.ticket, workload_id).map_err(|_| {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "invalid or expired exec ticket" })),
        )
    })?;

    let (tunnel_ip, agent_id) = resolve_agent_target(&state, workload_id).await?;
    let proxy_ticket = fetch_proxy_ticket(&state, agent_id, workload_id).await?;

    let agent_port: u16 = std::env::var(CSFX_AGENT_PORT_ENV)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7443);

    let agent_url = format!("ws://{}:{}/exec/{}", tunnel_ip, agent_port, workload_id);

    let mut request = agent_url.into_client_request().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("invalid agent exec url: {}", e) })),
        )
    })?;
    request.headers_mut().insert(
        "X-Csfx-Proxy-Ticket",
        proxy_ticket.parse().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "invalid proxy ticket header" })),
            )
        })?,
    );

    let (agent_socket, _) = tokio_tungstenite::connect_async(request).await.map_err(|e| {
        (
            StatusCode::BAD_GATEWAY,
            Json(json!({ "error": format!("failed to connect to agent exec socket: {}", e) })),
        )
    })?;

    Ok(ws.on_upgrade(move |browser_socket| bridge_exec_sockets(browser_socket, agent_socket)))
}

async fn bridge_exec_sockets(
    browser_socket: WebSocket,
    agent_socket: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    let (mut browser_sink, mut browser_stream) = browser_socket.split();
    let (mut agent_sink, mut agent_stream) = agent_socket.split();

    let browser_to_agent = tokio::spawn(async move {
        while let Some(Ok(msg)) = browser_stream.next().await {
            let forwarded = match msg {
                Message::Binary(data) => TungsteniteMessage::Binary(data),
                Message::Text(text) => TungsteniteMessage::Text(text.as_str().into()),
                Message::Close(_) => break,
                _ => continue,
            };
            if agent_sink.send(forwarded).await.is_err() {
                break;
            }
        }
    });

    let agent_to_browser = tokio::spawn(async move {
        while let Some(Ok(msg)) = agent_stream.next().await {
            let forwarded = match msg {
                TungsteniteMessage::Binary(data) => Message::Binary(data),
                TungsteniteMessage::Text(text) => Message::Text(text.as_str().into()),
                TungsteniteMessage::Close(_) => break,
                _ => continue,
            };
            if browser_sink.send(forwarded).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = browser_to_agent => {}
        _ = agent_to_browser => {}
    }
}

pub fn agent_proxy_routes() -> Router<AppState> {
    Router::new()
        .route("/workloads/{id}/logs", get(stream_workload_logs))
        .route("/workloads/{id}/exec/ticket", axum::routing::post(issue_exec_ticket))
        .route("/workloads/{id}/exec", get(exec_workload))
}
