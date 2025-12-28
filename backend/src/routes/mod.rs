use super::AppState;
use crate::utils::router_ext::RouterExt;
use axum::body::Body;
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::Method;
use axum::http::{HeaderValue, Request, Response};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{info_span, Span};

pub mod agents;
pub mod expenses;
pub mod frontend;
pub mod marketplace;
pub mod organizations;
pub mod resource_groups;
pub mod resources;
pub mod subscriptions;
pub mod system;
pub mod users;

/// Creates the main application router and logs all registered routes.
pub fn create_router() -> Router<AppState> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    tracing::info!("CORS configured for frontend URL: {}", frontend_url);

    // a permissive CORS layer is used for development
    let cors = CorsLayer::new()
        .allow_origin(frontend_url.parse::<HeaderValue>().unwrap())
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true);

    let api_router = Router::new()
        .merge(agents::agents_routes())
        .merge(expenses::expenses_routes())
        .merge(marketplace::marketplace_routes())
        .merge(organizations::routes())
        .merge(resource_groups::resource_groups_routes())
        .merge(resources::resources_routes())
        .merge(subscriptions::subscriptions_routes())
        .merge(system::routes())
        .merge(users::users_routes());

    Router::new()
        // API routes have priority
        .logged_nest("/api", api_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    info_span!(
                        "http_request",
                        method = %request.method(),
                        uri = %request.uri(),
                        version = ?request.version(),
                    )
                })
                .on_request(|_request: &Request<Body>, _span: &Span| {
                    tracing::info!("started processing request")
                })
                .on_response(
                    |_response: &Response<Body>, latency: std::time::Duration, _span: &Span| {
                        tracing::info!(
                            latency_ms = latency.as_millis(),
                            "finished processing request"
                        )
                    },
                ),
        )
        .layer(cors)
        // Frontend proxy as fallback - catches all routes not handled above
        .fallback_service(frontend::frontend_router())
}
