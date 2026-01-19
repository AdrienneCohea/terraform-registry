use crate::types::{Package, Platform, VersionInfo};
use std::str::FromStr;
use std::sync::Arc;

use super::{Backend, Error, Result};
use crate::providers::gitlabrelease::TryFromLinkForPlatformError::{
    InvalidFileNameFormat, UnsupportedArch, UnsupportedOS,
};
use gitlab::Gitlab;
use gitlab::api::Query;
use gitlab::api::projects::releases::ProjectReleases;
use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Clone)]
#[allow(dead_code)]
pub struct GitLabBackend {
    client: Arc<Gitlab>,
    project: Option<String>,
}

impl Backend for GitLabBackend {
    fn list_provider_versions(
        &self,
        _namespace: String,
        _provider_type: String,
    ) -> Result<Vec<VersionInfo>> {
        if let Some(project) = &self.project {
            return match self.list_project_releases(project) {
                Ok(releases) => Ok(releases
                    .iter()
                    .filter_map(|rel| VersionInfo::try_from(rel).ok())
                    .collect()),
                Err(Error::NotFound) => Err(Error::NotFound),
                Err(Error::StorageError) => Err(Error::StorageError),
            };
        }

        Err(Error::StorageError)
    }

    fn find_provider_package(
        &self,
        _namespace: String,
        _provider_type: String,
        _version: String,
        _os: String,
        _arch: String,
    ) -> Result<Package> {
        todo!()
    }
}

impl GitLabBackend {
    fn list_project_releases(&self, project: &str) -> Result<Vec<GitLabRelease>> {
        let endpoint = ProjectReleases::builder()
            .project(urlencoding::encode(project).to_string())
            .build()
            .map_err(|_| Error::StorageError)?;

        let releases: Vec<GitLabRelease> = endpoint
            .query(&*self.client)
            .map_err(|_| Error::StorageError)?;

        Ok(releases)
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TryFromGitLabError {
    InvalidVersion(String, String),
    MissingSignatureLink,
    MissingShaSumsLink,
    InvalidPackageLink(TryFromLinkForPlatformError),
}

impl TryFrom<&GitLabRelease> for VersionInfo {
    type Error = TryFromGitLabError;

    fn try_from(value: &GitLabRelease) -> std::result::Result<Self, Self::Error> {
        let mut tag_name = value.tag_name.clone();
        if !value.tag_name.starts_with('v') {
            return Err(TryFromGitLabError::InvalidVersion(
                value.tag_name.clone(),
                "tag name must be in the format `v{semver}`, i.e. v1.0.3".to_string(),
            ));
        }

        let tag_end = tag_name.split_off(1);

        let version = semver::Version::parse(&tag_end).map_err(|_| {
            TryFromGitLabError::InvalidVersion(
                value.tag_name.clone(),
                "tag name must be a valid semantic version".to_string(),
            )
        })?;

        value
            .assets
            .links
            .iter()
            .find(|link| link.name.ends_with("SUMS.sig"))
            .ok_or(TryFromGitLabError::MissingSignatureLink)?;
        value
            .assets
            .links
            .iter()
            .find(|link| link.name.ends_with("SUMS"))
            .ok_or(TryFromGitLabError::MissingShaSumsLink)?;

        let zip_file_urls: Vec<Link> = value
            .assets
            .links
            .iter()
            .filter(|l| {
                std::path::Path::new(&l.name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("zip"))
            })
            .cloned()
            .collect();

        // Gather the platforms Iterate through the file names, parse their platforms, and discard cases where parsing fails.
        let platforms: Vec<Platform> = zip_file_urls
            .iter()
            .map(|x| Platform::try_from(x.clone()))
            .filter_map(std::result::Result::ok)
            .collect();

        Ok(Self {
            version: version.to_string(),
            protocols: vec!["5.0".to_string()],
            platforms,
        })
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum TryFromLinkForPlatformError {
    NotZipFile,
    InvalidFileNameFormat,
    UnsupportedOS(String),
    UnsupportedArch(String),
}

impl TryFrom<Link> for Platform {
    type Error = TryFromLinkForPlatformError;

    fn try_from(link: Link) -> std::result::Result<Self, Self::Error> {
        let without_extension = link
            .name
            .strip_suffix(".zip")
            .ok_or(TryFromLinkForPlatformError::NotZipFile)?;

        let segments: Vec<&str> = without_extension.split('_').collect();
        if segments.len() != 3 {
            return Err(InvalidFileNameFormat);
        }

        let os = SupportedOS::from_str(segments[1])
            .map_err(|_| UnsupportedOS(segments[1].to_string()))?;
        let arch = SupportedArch::from_str(segments[2])
            .map_err(|_| UnsupportedArch(segments[2].to_string()))?;

        Ok(Self {
            os: os.0,
            arch: arch.0,
        })
    }
}

struct SupportedOS(String);

impl FromStr for SupportedOS {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "linux" | "darwin" | "windows" | "freebsd" | "openbsd" | "solaris" => {
                Ok(SupportedOS(s.to_string()))
            }
            _ => Err(format!("unsupported os: {s}")),
        }
    }
}

struct SupportedArch(String);

impl FromStr for SupportedArch {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "arm" | "arm64" | "386" | "amd64" => Ok(SupportedArch(s.to_string())),
            _ => Err(format!("unsupported architecture: {s}")),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GitLabRelease {
    pub tag_name: String,
    pub assets: Assets,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Assets {
    pub links: Vec<Link>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Link {
    pub name: String,
    pub url: String,
    #[serde(rename = "direct_asset_url")]
    pub direct_asset_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_link(name: &str) -> Link {
        Link {
            name: name.to_string(),
            url: format!("https://example.com/{name}"),
            direct_asset_url: format!("https://example.com/direct/{name}"),
        }
    }

    fn make_release(tag_name: &str, link_names: Vec<&str>) -> GitLabRelease {
        GitLabRelease {
            tag_name: tag_name.to_string(),
            assets: Assets {
                links: link_names.into_iter().map(make_link).collect(),
            },
        }
    }

    #[test]
    fn try_from_valid_release_with_platforms() {
        let release = make_release(
            "v1.2.3",
            vec![
                "terraform-provider-example_SHA256SUMS",
                "terraform-provider-example_SHA256SUMS.sig",
                "terraform-provider-example_linux_amd64.zip",
                "terraform-provider-example_darwin_arm64.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.version, "1.2.3");
        assert_eq!(version_info.protocols, vec!["5.0".to_string()]);
        assert_eq!(version_info.platforms.len(), 2);
        assert!(version_info
            .platforms
            .iter()
            .any(|p| p.os == "linux" && p.arch == "amd64"));
        assert!(version_info
            .platforms
            .iter()
            .any(|p| p.os == "darwin" && p.arch == "arm64"));
    }

    #[test]
    fn try_from_valid_release_with_prerelease_version() {
        let release = make_release(
            "v2.0.0-beta.1",
            vec![
                "provider_SHA256SUMS",
                "provider_SHA256SUMS.sig",
                "provider_linux_386.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.version, "2.0.0-beta.1");
    }

    #[test]
    fn try_from_tag_missing_v_prefix() {
        let release = make_release(
            "1.0.0",
            vec!["provider_SHA256SUMS", "provider_SHA256SUMS.sig"],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_err());

        match result.unwrap_err() {
            TryFromGitLabError::InvalidVersion(tag, msg) => {
                assert_eq!(tag, "1.0.0");
                assert!(msg.contains("v{semver}"));
            }
            _ => panic!("expected InvalidVersion error"),
        }
    }

    #[test]
    fn try_from_tag_invalid_semver() {
        let release = make_release(
            "vnotaversion",
            vec!["provider_SHA256SUMS", "provider_SHA256SUMS.sig"],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_err());

        match result.unwrap_err() {
            TryFromGitLabError::InvalidVersion(tag, msg) => {
                assert_eq!(tag, "vnotaversion");
                assert!(msg.contains("semantic version"));
            }
            _ => panic!("expected InvalidVersion error"),
        }
    }

    #[test]
    fn try_from_missing_shasums_sig() {
        let release = make_release(
            "v1.0.0",
            vec![
                "provider_SHA256SUMS",
                "provider_linux_amd64.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_err());

        match result.unwrap_err() {
            TryFromGitLabError::MissingSignatureLink => {}
            _ => panic!("expected MissingSignatureLink error"),
        }
    }

    #[test]
    fn try_from_missing_shasums() {
        let release = make_release(
            "v1.0.0",
            vec![
                "provider_SHA256SUMS.sig",
                "provider_linux_amd64.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_err());

        match result.unwrap_err() {
            TryFromGitLabError::MissingShaSumsLink => {}
            _ => panic!("expected MissingShaSumsLink error"),
        }
    }

    #[test]
    fn try_from_no_zip_files_returns_empty_platforms() {
        let release = make_release(
            "v1.0.0",
            vec!["provider_SHA256SUMS", "provider_SHA256SUMS.sig"],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.version, "1.0.0");
        assert!(version_info.platforms.is_empty());
    }

    #[test]
    fn try_from_filters_out_invalid_platform_zip_files() {
        let release = make_release(
            "v1.0.0",
            vec![
                "provider_SHA256SUMS",
                "provider_SHA256SUMS.sig",
                "provider_linux_amd64.zip",
                "provider_unsupportedos_amd64.zip",
                "provider_linux_unsupportedarch.zip",
                "invalid_format.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.platforms.len(), 1);
        assert_eq!(version_info.platforms[0].os, "linux");
        assert_eq!(version_info.platforms[0].arch, "amd64");
    }

    #[test]
    fn try_from_all_supported_os_and_arch() {
        let release = make_release(
            "v1.0.0",
            vec![
                "provider_SHA256SUMS",
                "provider_SHA256SUMS.sig",
                "provider_linux_amd64.zip",
                "provider_linux_arm64.zip",
                "provider_linux_arm.zip",
                "provider_linux_386.zip",
                "provider_darwin_amd64.zip",
                "provider_darwin_arm64.zip",
                "provider_windows_amd64.zip",
                "provider_freebsd_amd64.zip",
                "provider_openbsd_amd64.zip",
                "provider_solaris_amd64.zip",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.platforms.len(), 10);
    }

    #[test]
    fn try_from_ignores_non_zip_asset_files() {
        let release = make_release(
            "v1.0.0",
            vec![
                "provider_SHA256SUMS",
                "provider_SHA256SUMS.sig",
                "provider_linux_amd64.zip",
                "provider_linux_arm64.tar.gz",
                "README.md",
            ],
        );

        let result = VersionInfo::try_from(&release);
        assert!(result.is_ok());

        let version_info = result.unwrap();
        assert_eq!(version_info.platforms.len(), 1);
        assert_eq!(version_info.platforms[0].os, "linux");
        assert_eq!(version_info.platforms[0].arch, "amd64");
    }
}
