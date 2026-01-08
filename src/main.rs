use axum::{Json, Router, extract::Path, http::StatusCode, response::IntoResponse, routing::get};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::info;

/// Service discovery response for Terraform registry protocol
#[derive(Debug, Serialize, Deserialize)]
struct ServiceDiscovery {
    #[serde(rename = "providers.v1")]
    providers_v1: String,
}

/// Provider versions response
#[derive(Debug, Serialize, Deserialize)]
struct VersionsResponse {
    versions: Vec<VersionInfo>,
}

/// Information about a specific provider version
#[derive(Debug, Serialize, Deserialize)]
struct VersionInfo {
    version: String,
    protocols: Vec<String>,
    platforms: Vec<Platform>,
}

/// Platform information
#[derive(Debug, Serialize, Deserialize)]
struct Platform {
    os: String,
    arch: String,
}

/// Provider download response
#[derive(Debug, Serialize, Deserialize)]
struct DownloadResponse {
    protocols: Vec<String>,
    os: String,
    arch: String,
    filename: String,
    download_url: String,
    shasums_url: String,
    shasums_signature_url: String,
    shasum: String,
    signing_keys: SigningKeys,
}

/// GPG signing keys
#[derive(Debug, Serialize, Deserialize)]
struct SigningKeys {
    gpg_public_keys: Vec<GpgPublicKey>,
}

/// GPG public key information
#[derive(Debug, Serialize, Deserialize)]
struct GpgPublicKey {
    key_id: String,
    ascii_armor: String,
}

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
fn app() -> Router {
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

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Build the application
    let app = app();

    // Configure the socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting Terraform registry server on {}", addr);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address");

    info!("Server listening on http://{}", addr);

    // Start serving
    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_service_discovery_returns_ok() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/terraform.json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_health_check_returns_ok() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_versions_returns_ok() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/providers/hashicorp/aws/versions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_download_endpoint_returns_ok() {
        let app = app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/v1/providers/hashicorp/aws/1.0.0/download/linux/amd64")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
