use crate::providers::Result as ProviderResult;
use crate::providers::{Backend, FakeBackend, GitLabBackend};
use config::Config;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
pub struct AppConfig {
    pub bind_address: SocketAddr,
    pub providers_backend: ProvidersBackend,
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProvidersBackend {
    Fake,
    GitLabRelease(GitLabConfig),
}

#[derive(Deserialize, Serialize, PartialEq, Clone, Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yml as yaml;

    #[test]
    fn test_config_fake_backend() {
        let yaml = "\
bind_address: '127.0.0.1:8000'
providers_backend:
  type: fake";

        let config: AppConfig = yaml::from_str(yaml).unwrap();

        assert_eq!(config.bind_address, SocketAddr::from(([127, 0, 0, 1], 8000)));
        assert_eq!(config.providers_backend, ProvidersBackend::Fake);
    }

    #[test]
    fn test_config_gitlab_release_backend() {
        let yaml = "\
bind_address: '127.0.0.1:8000'
providers_backend:
  type: git_lab_release
  host: gitlab.example.com
  token: secret-token
  project: my-project";

        let config: AppConfig = yaml::from_str(yaml).unwrap();

        assert_eq!(config.bind_address, SocketAddr::from(([127, 0, 0, 1], 8000)));
        assert_eq!(
            config.providers_backend,
            ProvidersBackend::GitLabRelease(GitLabConfig {
                host: "gitlab.example.com".to_string(),
                token: "secret-token".to_string(),
                project: Some("my-project".to_string()),
            })
        );
    }
}