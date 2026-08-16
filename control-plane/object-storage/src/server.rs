use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use sea_orm::DatabaseConnection;

use crate::{
    garage::GarageClient,
    handlers::{buckets, cluster, keys},
    metrics,
};

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub garage: GarageClient,
}

impl AppState {
    pub fn new(db: DatabaseConnection, garage: GarageClient) -> Self {
        Self { db, garage }
    }
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "Object Storage OK")
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/metrics", get(metrics::metrics_handler))
        .route(
            "/buckets",
            get(buckets::list_buckets).post(buckets::create_bucket),
        )
        .route(
            "/buckets/{id}",
            get(buckets::get_bucket)
                .patch(buckets::update_bucket)
                .delete(buckets::delete_bucket),
        )
        .route(
            "/buckets/{id}/keys",
            get(keys::list_keys).post(keys::create_key),
        )
        .route(
            "/buckets/{id}/keys/{key_id}/rotate",
            axum::routing::post(keys::rotate_key),
        )
        .route(
            "/buckets/{id}/keys/{key_id}",
            axum::routing::delete(keys::delete_key),
        )
        .route("/cluster", get(cluster::get_cluster_status))
        .with_state(state)
}
