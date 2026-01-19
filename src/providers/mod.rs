mod fake;
mod gitlabrelease;

pub use fake::FakeBackend;

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

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
pub enum Error {
    NotFound,
    StorageError,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound => axum::http::StatusCode::NOT_FOUND.into_response(),
            Self::StorageError => axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
