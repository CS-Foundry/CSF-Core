use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, post, put},
    Router,
};
use entity::entities::{docker_resources, resource_groups};
use entity::{DockerResources, Organization, ResourceGroups};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::middleware::AuthenticatedUser;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateResourceRequest {
    pub name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub resource_group_id: Uuid,
    pub configuration: Option<serde_json::Value>,
    pub tags: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateResourceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub configuration: Option<serde_json::Value>,
    pub status: Option<String>,
    pub tags: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourceResponse {
    pub id: Uuid,
    pub name: String,
    pub resource_type: String,
    pub description: Option<String>,
    pub resource_group_id: Uuid,
    pub resource_group_name: String,
    pub configuration: Option<serde_json::Value>,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: String,
    pub updated_at: String,
    pub tags: Option<serde_json::Value>,
    pub container_id: Option<String>,
    pub stack_name: Option<String>,
}

/// List all resources across all resource groups
async fn list_resources(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
) -> impl IntoResponse {
    let db = &state.db_conn;

    // Get the organization
    let org = match Organization::find().one(db).await {
        Ok(Some(o)) => o,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Organization not found"
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get organization: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve organization"
                })),
            )
                .into_response();
        }
    };

    // Get all resource groups for this org
    let resource_group_ids: Vec<Uuid> = match ResourceGroups::find()
        .filter(resource_groups::Column::OrganizationId.eq(org.id))
        .all(db)
        .await
    {
        Ok(groups) => groups.into_iter().map(|g| g.id).collect(),
        Err(e) => {
            tracing::error!("Failed to get resource groups: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource groups"
                })),
            )
                .into_response();
        }
    };

    // Get all resources for these resource groups
    match DockerResources::find()
        .filter(docker_resources::Column::ResourceGroupId.is_in(resource_group_ids))
        .order_by_desc(docker_resources::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(resources) => {
            let mut response_resources = Vec::new();

            // Fetch resource group names
            for resource in resources {
                if let Ok(Some(rg)) = ResourceGroups::find_by_id(resource.resource_group_id)
                    .one(db)
                    .await
                {
                    response_resources.push(ResourceResponse {
                        id: resource.id,
                        name: resource.name,
                        resource_type: resource.resource_type,
                        description: resource.description,
                        resource_group_id: resource.resource_group_id,
                        resource_group_name: rg.name,
                        configuration: resource.configuration,
                        status: resource.status,
                        created_by: resource.created_by,
                        created_at: resource.created_at.to_string(),
                        updated_at: resource.updated_at.to_string(),
                        tags: resource.tags,
                        container_id: resource.container_id,
                        stack_name: resource.stack_name,
                    });
                }
            }

            (StatusCode::OK, Json(response_resources)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get resources: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resources"
                })),
            )
                .into_response()
        }
    }
}

/// List resources for a specific resource group
async fn list_resources_by_group(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(resource_group_id): Path<Uuid>,
) -> impl IntoResponse {
    let db = &state.db_conn;

    // Verify resource group exists and belongs to user's org
    let _rg = match ResourceGroups::find_by_id(resource_group_id).one(db).await {
        Ok(Some(rg)) => rg,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Resource group not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get resource group: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource group"
                })),
            )
                .into_response();
        }
    };

    match DockerResources::find()
        .filter(docker_resources::Column::ResourceGroupId.eq(resource_group_id))
        .order_by_desc(docker_resources::Column::CreatedAt)
        .all(db)
        .await
    {
        Ok(resources) => {
            let response_resources: Vec<ResourceResponse> = resources
                .into_iter()
                .map(|r| ResourceResponse {
                    id: r.id,
                    name: r.name.clone(),
                    resource_type: r.resource_type.clone(),
                    description: r.description.clone(),
                    resource_group_id: r.resource_group_id,
                    resource_group_name: _rg.name.clone(),
                    configuration: r.configuration.clone(),
                    status: r.status.clone(),
                    created_by: r.created_by,
                    created_at: r.created_at.to_string(),
                    updated_at: r.updated_at.to_string(),
                    tags: r.tags.clone(),
                    container_id: r.container_id.clone(),
                    stack_name: r.stack_name.clone(),
                })
                .collect();

            (StatusCode::OK, Json(response_resources)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get resources: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resources"
                })),
            )
                .into_response()
        }
    }
}

/// Get a specific resource by ID
async fn get_resource(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let db = &state.db_conn;

    match DockerResources::find_by_id(id).one(db).await {
        Ok(Some(resource)) => {
            // Get resource group name
            let rg_name = if let Ok(Some(rg)) =
                ResourceGroups::find_by_id(resource.resource_group_id)
                    .one(db)
                    .await
            {
                rg.name
            } else {
                "Unknown".to_string()
            };

            let response = ResourceResponse {
                id: resource.id,
                name: resource.name,
                resource_type: resource.resource_type,
                description: resource.description,
                resource_group_id: resource.resource_group_id,
                resource_group_name: rg_name,
                configuration: resource.configuration,
                status: resource.status,
                created_by: resource.created_by,
                created_at: resource.created_at.to_string(),
                updated_at: resource.updated_at.to_string(),
                tags: resource.tags,
                container_id: resource.container_id,
                stack_name: resource.stack_name,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Resource not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get resource: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource"
                })),
            )
                .into_response()
        }
    }
}

/// Create a new resource
async fn create_resource(
    State(state): State<AppState>,
    AuthenticatedUser(user): AuthenticatedUser,
    Json(payload): Json<CreateResourceRequest>,
) -> impl IntoResponse {
    let db = &state.db_conn;

    // Verify resource group exists
    let rg = match ResourceGroups::find_by_id(payload.resource_group_id)
        .one(db)
        .await
    {
        Ok(Some(rg)) => rg,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Resource group not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get resource group: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource group"
                })),
            )
                .into_response();
        }
    };

    let now = chrono::Utc::now().naive_utc();
    let new_resource = docker_resources::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        name: ActiveValue::Set(payload.name.clone()),
        resource_type: ActiveValue::Set(payload.resource_type.clone()),
        description: ActiveValue::Set(payload.description.clone()),
        resource_group_id: ActiveValue::Set(payload.resource_group_id),
        configuration: ActiveValue::Set(payload.configuration.clone()),
        status: ActiveValue::Set("pending".to_string()),
        created_by: ActiveValue::Set(Some(user.user_id)),
        created_at: ActiveValue::Set(now),
        updated_at: ActiveValue::Set(now),
        tags: ActiveValue::Set(payload.tags.clone()),
        container_id: ActiveValue::NotSet,
        stack_name: ActiveValue::NotSet,
    };

    match new_resource.insert(db).await {
        Ok(resource) => {
            let response = ResourceResponse {
                id: resource.id,
                name: resource.name,
                resource_type: resource.resource_type,
                description: resource.description,
                resource_group_id: resource.resource_group_id,
                resource_group_name: rg.name,
                configuration: resource.configuration,
                status: resource.status,
                created_by: resource.created_by,
                created_at: resource.created_at.to_string(),
                updated_at: resource.updated_at.to_string(),
                tags: resource.tags,
                container_id: resource.container_id,
                stack_name: resource.stack_name,
            };

            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create resource: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to create resource"
                })),
            )
                .into_response()
        }
    }
}

/// Update an existing resource
async fn update_resource(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateResourceRequest>,
) -> impl IntoResponse {
    let db = &state.db_conn;

    let resource = match DockerResources::find_by_id(id).one(db).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "Resource not found"
                })),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!("Failed to get resource: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource"
                })),
            )
                .into_response();
        }
    };

    let mut resource_model: docker_resources::ActiveModel = resource.into();

    if let Some(name) = payload.name {
        resource_model.name = ActiveValue::Set(name);
    }
    if let Some(description) = payload.description {
        resource_model.description = ActiveValue::Set(Some(description));
    }
    if let Some(configuration) = payload.configuration {
        resource_model.configuration = ActiveValue::Set(Some(configuration));
    }
    if let Some(status) = payload.status {
        resource_model.status = ActiveValue::Set(status);
    }
    if let Some(tags) = payload.tags {
        resource_model.tags = ActiveValue::Set(Some(tags));
    }

    resource_model.updated_at = ActiveValue::Set(chrono::Utc::now().naive_utc());

    match resource_model.update(db).await {
        Ok(updated) => {
            let rg_name = if let Ok(Some(rg)) =
                ResourceGroups::find_by_id(updated.resource_group_id)
                    .one(db)
                    .await
            {
                rg.name
            } else {
                "Unknown".to_string()
            };

            let response = ResourceResponse {
                id: updated.id,
                name: updated.name,
                resource_type: updated.resource_type,
                description: updated.description,
                resource_group_id: updated.resource_group_id,
                resource_group_name: rg_name,
                configuration: updated.configuration,
                status: updated.status,
                created_by: updated.created_by,
                created_at: updated.created_at.to_string(),
                updated_at: updated.updated_at.to_string(),
                tags: updated.tags,
                container_id: updated.container_id,
                stack_name: updated.stack_name,
            };

            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to update resource: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to update resource"
                })),
            )
                .into_response()
        }
    }
}

/// Delete a resource
async fn delete_resource(
    State(state): State<AppState>,
    AuthenticatedUser(_user): AuthenticatedUser,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let db = &state.db_conn;

    match DockerResources::find_by_id(id).one(db).await {
        Ok(Some(resource)) => {
            let resource_model: docker_resources::ActiveModel = resource.into();
            match resource_model.delete(db).await {
                Ok(_) => (StatusCode::NO_CONTENT).into_response(),
                Err(e) => {
                    tracing::error!("Failed to delete resource: {}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({
                            "error": "Failed to delete resource"
                        })),
                    )
                        .into_response()
                }
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Resource not found"
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!("Failed to get resource: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Failed to retrieve resource"
                })),
            )
                .into_response()
        }
    }
}

pub fn resources_routes() -> Router<AppState> {
    Router::new()
        .route("/resources", get(list_resources))
        .route("/resources/:id", get(get_resource))
        .route("/resources", post(create_resource))
        .route("/resources/:id", put(update_resource))
        .route("/resources/:id", delete(delete_resource))
        .route(
            "/resource-groups/:resource_group_id/resources",
            get(list_resources_by_group),
        )
}
