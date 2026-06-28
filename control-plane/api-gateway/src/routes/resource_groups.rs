use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use chrono::Utc;
use entity::{
    entities::{agents, networks, resource_groups, volumes, workloads},
    Agents, Networks, ResourceGroups, Volumes, Workloads,
};
use ring::rand::{SecureRandom, SystemRandom};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::rbac::{CanManageResourceGroups, CanViewResourceGroups},
    AppState,
};

#[derive(Debug, Deserialize)]
pub struct CreateResourceGroupRequest {
    pub name: String,
    pub description: Option<String>,
    pub internal_cidr: String,
}

#[derive(Debug, Serialize)]
pub struct ResourceGroupResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub internal_cidr: String,
    pub status: String,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: Option<chrono::NaiveDateTime>,
}

impl From<resource_groups::Model> for ResourceGroupResponse {
    fn from(m: resource_groups::Model) -> Self {
        Self {
            id: m.id,
            organization_id: m.organization_id,
            name: m.name,
            description: m.description,
            internal_cidr: m.internal_cidr,
            status: m.status,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

fn get_org_id(state: &AppState) -> Uuid {
    state
        .default_org_id
        .expect("default organization not initialized")
}

pub async fn list_resource_groups(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    let groups = ResourceGroups::find()
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .all(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list resource groups");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    let resp: Vec<ResourceGroupResponse> = groups.into_iter().map(Into::into).collect();
    Ok((StatusCode::OK, Json(json!(resp))))
}

pub async fn create_resource_group(
    CanManageResourceGroups(_claims): CanManageResourceGroups,
    State(state): State<AppState>,
    Json(req): Json<CreateResourceGroupRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    let existing = ResourceGroups::find()
        .filter(resource_groups::Column::InternalCidr.eq(&req.internal_cidr))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to check cidr uniqueness");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": format!("CIDR {} is already in use by another resource group", req.internal_cidr) })),
        ));
    }

    let now = Utc::now().naive_utc();
    let model = resource_groups::ActiveModel {
        id: Set(Uuid::new_v4()),
        organization_id: Set(org_id),
        name: Set(req.name),
        description: Set(req.description),
        internal_cidr: Set(req.internal_cidr),
        status: Set("active".to_string()),
        created_at: Set(now),
        updated_at: Set(None),
    };

    let inserted = model.insert(&state.db_conn).await.map_err(|e| {
        tracing::error!(error = %e, "failed to create resource group");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "database error" })),
        )
    })?;

    Ok((StatusCode::CREATED, Json(json!(ResourceGroupResponse::from(inserted)))))
}

pub async fn get_resource_group(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    let group = ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to get resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    Ok((StatusCode::OK, Json(json!(ResourceGroupResponse::from(group)))))
}

pub async fn delete_resource_group(
    CanManageResourceGroups(_claims): CanManageResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    let group = ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to find resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    let active_workloads = Workloads::find()
        .filter(workloads::Column::ResourceGroupId.eq(id))
        .filter(workloads::Column::Status.is_in(["pending", "scheduled", "running"]))
        .count(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to count workloads");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    if active_workloads > 0 {
        return Err((
            StatusCode::CONFLICT,
            Json(json!({ "error": "resource group has active workloads" })),
        ));
    }

    let mut active: resource_groups::ActiveModel = group.into();
    active.status = Set("deleting".to_string());
    active.updated_at = Set(Some(Utc::now().naive_utc()));
    active.update(&state.db_conn).await.map_err(|e| {
        tracing::error!(error = %e, "failed to update resource group status");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "database error" })),
        )
    })?;

    ResourceGroups::delete_by_id(id)
        .exec(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to delete resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    Ok(StatusCode::NO_CONTENT.into_response())
}

pub async fn list_resource_group_workloads(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to find resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    let workloads = Workloads::find()
        .filter(workloads::Column::ResourceGroupId.eq(id))
        .all(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list workloads");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    Ok((StatusCode::OK, Json(json!(workloads))))
}

pub async fn list_resource_group_volumes(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to find resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    let vols = Volumes::find()
        .filter(volumes::Column::ResourceGroupId.eq(id))
        .all(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list volumes");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    Ok((StatusCode::OK, Json(json!(vols))))
}

pub async fn list_resource_group_networks(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to find resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    let nets = Networks::find()
        .filter(networks::Column::ResourceGroupId.eq(id))
        .all(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to list networks");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?;

    Ok((StatusCode::OK, Json(json!(nets))))
}

pub async fn get_vpn_config(
    CanViewResourceGroups(_claims): CanViewResourceGroups,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let org_id = get_org_id(&state);

    let group = ResourceGroups::find_by_id(id)
        .filter(resource_groups::Column::OrganizationId.eq(org_id))
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to get resource group");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "resource group not found" })),
            )
        })?;

    let gateway_agent = Agents::find()
        .filter(agents::Column::Status.eq("Online"))
        .filter(agents::Column::WgPublicKey.is_not_null())
        .filter(agents::Column::WgEndpoint.is_not_null())
        .order_by_asc(agents::Column::RegisteredAt)
        .one(&state.db_conn)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "failed to find gateway agent");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "database error" })),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({ "error": "no WireGuard-enabled agent online" })),
            )
        })?;

    let server_pubkey = gateway_agent.wg_public_key.as_deref().unwrap_or("");
    let endpoint = gateway_agent.wg_endpoint.as_deref().unwrap_or("");

    let client_private_key = generate_wg_key().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": "failed to generate keypair" })),
        )
    })?;

    let dns = first_host_ip(&group.internal_cidr).unwrap_or_else(|| "1.1.1.1".to_string());

    let config = format!(
        "[Interface]\nPrivateKey = {client_private_key}\nAddress = {dns}/32\nDNS = {dns}\n\n[Peer]\nPublicKey = {server_pubkey}\nEndpoint = {endpoint}\nAllowedIPs = {cidr}\nPersistentKeepalive = 25\n",
        client_private_key = client_private_key,
        dns = dns,
        server_pubkey = server_pubkey,
        endpoint = endpoint,
        cidr = group.internal_cidr,
    );

    let filename = format!("csfx-{}.conf", group.name.to_lowercase().replace(' ', "-"));

    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                &format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        config,
    )
        .into_response())
}

fn generate_wg_key() -> Result<String, ring::error::Unspecified> {
    let rng = SystemRandom::new();
    let mut key_bytes = [0u8; 32];
    rng.fill(&mut key_bytes)?;
    key_bytes[0] &= 248;
    key_bytes[31] &= 127;
    key_bytes[31] |= 64;
    Ok(B64.encode(key_bytes))
}

fn first_host_ip(cidr: &str) -> Option<String> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return None;
    }
    let octets: Vec<u8> = parts[0]
        .split('.')
        .filter_map(|o| o.parse().ok())
        .collect();
    if octets.len() != 4 {
        return None;
    }
    let n = u32::from_be_bytes([octets[0], octets[1], octets[2], octets[3]]);
    let host = n + 1;
    let [a, b, c, d] = host.to_be_bytes();
    Some(format!("{}.{}.{}.{}", a, b, c, d))
}

pub fn resource_groups_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/resource-groups",
            get(list_resource_groups).post(create_resource_group),
        )
        .route(
            "/resource-groups/{id}",
            get(get_resource_group).delete(delete_resource_group),
        )
        .route(
            "/resource-groups/{id}/workloads",
            get(list_resource_group_workloads),
        )
        .route(
            "/resource-groups/{id}/volumes",
            get(list_resource_group_volumes),
        )
        .route(
            "/resource-groups/{id}/networks",
            get(list_resource_group_networks),
        )
        .route("/resource-groups/{id}/vpn-config", get(get_vpn_config))
}
