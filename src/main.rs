mod routes;
mod types;

use std::net::SocketAddr;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Build the application
    let app = routes::app();

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
    use crate::routes;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_service_discovery_returns_ok() {
        let app = routes::app();

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
        let app = routes::app();

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
        let app = routes::app();

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
        let app = routes::app();

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
