use crate::types::{GpgPublicKey, Package, Platform, SigningKeys, VersionInfo};

use super::{Backend, Result};

#[derive(Clone)]
pub struct FakeBackend;

impl Backend for FakeBackend {
    fn list_provider_versions(
        &self,
        _: String,
        _provider_type: String,
    ) -> Result<Vec<VersionInfo>> {
        Ok(vec![
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
        ])
    }

    fn find_provider_package(
        &self,
        namespace: String,
        provider_type: String,
        version: String,
        os: String,
        arch: String,
    ) -> Result<Package> {
        // Stub response with example download data
        let filename = format!("terraform-provider-{provider_type}_{version}_{os}_{arch}.zip");

        Ok(Package {
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
        })
    }
}
