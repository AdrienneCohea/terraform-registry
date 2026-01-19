use crate::providers::Result as ProviderResult;
use crate::providers::{Backend, FakeBackend, GitLabBackend};
use config::Config;
use serde_derive::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct AppConfig {
    pub bind_address: SocketAddr,
    pub providers_backend: ProvidersBackend,
}

#[derive(Deserialize)]
pub enum ProvidersBackend {
    Fake,
    GitLabRelease(GitLabConfig),
}

#[derive(Deserialize, Clone)]
pub struct GitLabConfig {
    pub host: String,
    pub token: String,
    pub project: Option<String>,
}

impl AppConfig {
    pub fn load(file: &str) -> Result<Self, config::ConfigError> {
        Config::builder()
            .add_source(config::File::with_name(file))
            .build()?
            .try_deserialize()
    }

    pub fn providers_backend(&self) -> ProviderResult<Arc<dyn Backend>> {
        match &self.providers_backend {
            ProvidersBackend::Fake => Ok(Arc::new(FakeBackend)),
            ProvidersBackend::GitLabRelease(cfg) => Ok(Arc::new(GitLabBackend::new(cfg.clone())?)),
        }
    }
}
