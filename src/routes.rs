use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::providers::Backend;
use crate::types::{ServiceDiscovery, VersionsResponse};

/// Service discovery endpoint - returns registry metadata
async fn service_discovery() -> impl IntoResponse {
    info!("Service discovery requested");

    let response = ServiceDiscovery {
        providers_v1: "/v1/providers/".to_string(),
    };

    Json(response)
}

/// List available versions for a provider
async fn list_versions(
    State(backend): State<Arc<dyn Backend>>,
    Path((namespace, provider_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Versions requested for {}/{}", namespace, provider_type);

    match backend.list_provider_versions(namespace, provider_type) {
        Ok(versions) => Json(VersionsResponse { versions }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Find a provider package for download
async fn find_provider_package(
    State(backend): State<Arc<dyn Backend>>,
    Path((namespace, provider_type, version, os, arch)): Path<(
        String,
        String,
        String,
        String,
        String,
    )>,
) -> impl IntoResponse {
    info!(
        "Download requested for {}/{} version {} on {}/{}",
        namespace, provider_type, version, os, arch
    );

    match backend.find_provider_package(namespace, provider_type, version, os, arch) {
        Ok(package) => Json(package).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Build the application router with all routes
#[allow(clippy::needless_pass_by_value)]
pub fn app(providers: Arc<dyn Backend>) -> Router {
    Router::new()
        .route("/.well-known/terraform.json", get(service_discovery))
        .route(
            "/v1/providers/{namespace}/{type}/versions",
            get(list_versions),
        )
        .with_state(providers.clone())
        .route(
            "/v1/providers/{namespace}/{type}/{version}/download/{os}/{arch}",
            get(find_provider_package),
        )
        .route("/health", get(health_check))
        .with_state(providers.clone())
        .layer(TraceLayer::new_for_http())
}
