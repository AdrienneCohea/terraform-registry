use serde::{Deserialize, Serialize};

/// Service discovery response for Terraform registry protocol
#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceDiscovery {
    #[serde(rename = "providers.v1")]
    pub providers_v1: String,
}

/// Provider versions response
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionsResponse {
    pub versions: Vec<VersionInfo>,
}

/// Information about a specific provider version
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub protocols: Vec<String>,
    pub platforms: Vec<Platform>,
}

/// Platform information
#[derive(Debug, Serialize, Deserialize)]
pub struct Platform {
    pub os: String,
    pub arch: String,
}

/// Provider download response
#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadResponse {
    pub protocols: Vec<String>,
    pub os: String,
    pub arch: String,
    pub filename: String,
    pub download_url: String,
    pub shasums_url: String,
    pub shasums_signature_url: String,
    pub shasum: String,
    pub signing_keys: SigningKeys,
}

pub type Package = DownloadResponse;

/// GPG signing keys
#[derive(Debug, Serialize, Deserialize)]
pub struct SigningKeys {
    pub gpg_public_keys: Vec<GpgPublicKey>,
}

/// GPG public key information
#[derive(Debug, Serialize, Deserialize)]
pub struct GpgPublicKey {
    pub key_id: String,
    pub ascii_armor: String,
}
