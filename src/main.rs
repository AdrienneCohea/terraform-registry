mod providers;
mod routes;
mod types;

use crate::providers::FakeBackend;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Initialize storage backends
    let providers = Arc::new(FakeBackend);

    // Build the application
    let app = routes::app(providers);

    // Configure the socket address
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Starting Terraform registry server on {}", addr);

    // Create the TCP listener
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on http://{}", addr);

    // Start serving
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::providers::FakeBackend;
    use crate::routes;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Arc;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_service_discovery_returns_ok() {
        let providers = Arc::new(FakeBackend);
        let app = routes::app(providers);

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
        let providers = Arc::new(FakeBackend);
        let app = routes::app(providers);

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
        let providers = Arc::new(FakeBackend);
        let app = routes::app(providers);

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
        let providers = Arc::new(FakeBackend);
        let app = routes::app(providers);

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
