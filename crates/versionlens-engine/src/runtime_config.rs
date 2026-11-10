use versionlens_fetch::{HttpConfig, HttpHeader};
use versionlens_model::{Dependency, Ecosystem, ManifestKind};

use crate::VersionLensSession;
use crate::prerelease::dependency_allows_prereleases;

impl VersionLensSession {
    pub(crate) fn includes_prereleases(&self, dependency: &Dependency) -> bool {
        self.config.show_prereleases || dependency_allows_prereleases(dependency)
    }

    pub(crate) fn prerelease_tags(&self, ecosystem: Ecosystem) -> &[String] {
        self.config
            .providers
            .prerelease_tags
            .iter()
            .rfind(|config| config.ecosystem == ecosystem)
            .map(|config| config.tags.as_slice())
            .unwrap_or(&[])
    }

    pub(crate) fn http_config(&self, ecosystem: Ecosystem) -> HttpConfig {
        self.http_config_for_manifest(ecosystem, None)
    }

    pub(crate) fn http_config_for_manifest(
        &self,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
    ) -> HttpConfig {
        self.http_config_with_headers(ecosystem, manifest_kind, &[])
    }

    pub(crate) fn http_config_with_headers(
        &self,
        ecosystem: Ecosystem,
        manifest_kind: Option<ManifestKind>,
        extra_headers: &[HttpHeader],
    ) -> HttpConfig {
        let strict_ssl = self
            .config
            .providers
            .provider_http
            .iter()
            .rfind(|config| {
                config.ecosystem == ecosystem && config.applies_to_manifest(manifest_kind)
            })
            .and_then(|config| config.strict_ssl)
            .unwrap_or(self.config.http.strict_ssl);

        HttpConfig {
            timeout_ms: self.config.http.timeout_ms,
            strict_ssl,
            proxy: self
                .config
                .http
                .proxy
                .as_deref()
                .map(|value| value.to_owned()),
            ca_file: self
                .config
                .http
                .ca_file
                .as_deref()
                .map(|value| value.to_owned()),
            ca: self.config.http.ca.as_deref().map(|value| value.to_owned()),
            cert_file: self
                .config
                .http
                .cert_file
                .as_deref()
                .map(|value| value.to_owned()),
            key_file: self
                .config
                .http
                .key_file
                .as_deref()
                .map(|value| value.to_owned()),
            cert: self
                .config
                .http
                .cert
                .as_deref()
                .map(|value| value.to_owned()),
            key: self
                .config
                .http
                .key
                .as_deref()
                .map(|value| value.to_owned()),
            auth_headers: self
                .config
                .http
                .auth_headers
                .iter()
                .chain(extra_headers.iter())
                .map(|header| HttpHeader {
                    name: header.name.as_str().to_owned(),
                    value: header.value.as_str().to_owned(),
                    url: header.url.as_deref().map(|value| value.to_owned()),
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
use crate::{ProviderHttpConfig, ProviderSettings, SessionConfig};
use versionlens_model::Ecosystem::*;
use versionlens_model::ManifestKind::{NpmPackageJson, PnpmYaml};

#[test]
fn provider_http_overrides_global_strict_ssl() {
    let session = session_with_provider_http(vec![
        ProviderHttpConfig {
            ecosystem: Npm,
            manifest_kind: None,
            strict_ssl: Some(false),
        },
        ProviderHttpConfig {
            ecosystem: Npm,
            manifest_kind: None,
            strict_ssl: Some(true),
        },
    ]);

    assert!(session.http_config(Npm).strict_ssl);
    assert!(session.http_config(Cargo).strict_ssl);
}

#[test]
fn manifest_scoped_provider_http_does_not_override_package_json_npm() {
    let session = session_with_provider_http(vec![
        ProviderHttpConfig {
            ecosystem: Npm,
            manifest_kind: None,
            strict_ssl: Some(false),
        },
        ProviderHttpConfig {
            ecosystem: Npm,
            manifest_kind: Some(PnpmYaml),
            strict_ssl: Some(true),
        },
    ]);

    assert!(!session.http_config(Npm).strict_ssl);
    assert!(
        session
            .http_config_for_manifest(Npm, Some(PnpmYaml))
            .strict_ssl
    );
    assert!(
        !session
            .http_config_for_manifest(Npm, Some(NpmPackageJson))
            .strict_ssl
    );
}

fn session_with_provider_http(provider_http: Vec<ProviderHttpConfig>) -> crate::VersionLensSession {
    crate::version_lens_session(SessionConfig {
        update_severity: Some(crate::Severity::Information),
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            provider_http,
            ..Default::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_fetch::standard_http_config(),
    })
}
}
