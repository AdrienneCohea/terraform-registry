use axum::{Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::types::*;

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
    Path((namespace, provider_type)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Versions requested for {}/{}", namespace, provider_type);

    // Stub response with example version data
    let response = VersionsResponse {
        versions: vec![
            VersionInfo {
                version: "1.0.0".to_string(),
                protocols: vec!["5.0".to_string()],
                platforms: vec![
                    Platform {
                        os: "linux".to_string(),
                        arch: "amd64".to_string(),
                    },
                    Platform {
                        os: "linux".to_string(),
                        arch: "arm64".to_string(),
                    },
                    Platform {
                        os: "darwin".to_string(),
                        arch: "amd64".to_string(),
                    },
                    Platform {
                        os: "darwin".to_string(),
                        arch: "arm64".to_string(),
                    },
                    Platform {
                        os: "windows".to_string(),
                        arch: "amd64".to_string(),
                    },
                ],
            },
            VersionInfo {
                version: "0.9.0".to_string(),
                protocols: vec!["5.0".to_string()],
                platforms: vec![Platform {
                    os: "linux".to_string(),
                    arch: "amd64".to_string(),
                }],
            },
        ],
    };

    Json(response)
}

/// Find a provider package for download
async fn find_provider_package(
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

    // Stub response with example download data
    let filename = format!("terraform-provider-{provider_type}_{version}_{os}_{arch}.zip");

    let response = DownloadResponse {
        protocols: vec!["5.0".to_string()],
        os: os.clone(),
        arch: arch.clone(),
        filename: filename.clone(),
        download_url: format!(
            "https://releases.example.com/{namespace}/{provider_type}/{filename}"
        ),
        shasums_url: format!(
            "https://releases.example.com/{namespace}/{provider_type}/terraform-provider-{provider_type}_{version}_SHA256SUMS"
        ),
        shasums_signature_url: format!(
            "https://releases.example.com/{namespace}/{provider_type}/terraform-provider-{provider_type}_{version}_SHA256SUMS.sig"
        ),
        shasum: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
        signing_keys: SigningKeys {
            gpg_public_keys: vec![GpgPublicKey {
                key_id: "0123456789ABCDEF".to_string(),
                ascii_armor:
                    "-----BEGIN PGP PUBLIC KEY BLOCK-----\n...\n-----END PGP PUBLIC KEY BLOCK-----"
                        .to_string(),
            }],
        },
    };

    Json(response)
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

/// Build the application router with all routes
pub fn app() -> Router {
    Router::new()
        .route("/.well-known/terraform.json", get(service_discovery))
        .route(
            "/v1/providers/{namespace}/{type}/versions",
            get(list_versions),
        )
        .route(
            "/v1/providers/{namespace}/{type}/{version}/download/{os}/{arch}",
            get(find_provider_package),
        )
        .route("/health", get(health_check))
        .layer(TraceLayer::new_for_http())
}