mod fake;
mod gitlabrelease;

pub use fake::FakeBackend;
pub use gitlabrelease::GitLabBackend;

use crate::types::{Package, VersionInfo};
use axum::response::{IntoResponse, Response};

pub trait Backend: Send + Sync {
    fn list_provider_versions(
        &self,
        namespace: String,
        provider_type: String,
    ) -> Result<Vec<VersionInfo>>;

    fn find_provider_package(
        &self,
        namespace: String,
        provider_type: String,
        version: String,
        os: String,
        arch: String,
    ) -> Result<Package>;
}

pub type Result<T> = std::result::Result<T, ProviderBackendError>;

use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ProviderBackendError {
    #[error("not found")]
    NotFound,
    #[error("storage error")]
    StorageError,
}

impl IntoResponse for ProviderBackendError {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => axum::http::StatusCode::NOT_FOUND.into_response(),
            Self::StorageError => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::providers::ProviderBackendError;
    use axum::response::IntoResponse;

    #[test]
    fn into_response_for_error() {
        assert_eq!(
            axum::http::StatusCode::NOT_FOUND,
            ProviderBackendError::NotFound.into_response().status()
        );
        assert_eq!(
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            ProviderBackendError::StorageError.into_response().status()
        );
    }
}
