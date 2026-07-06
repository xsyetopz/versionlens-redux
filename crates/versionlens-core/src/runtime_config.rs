use versionlens_http::{HttpConfig, HttpHeader};
use versionlens_parsers::{Dependency, Ecosystem, ManifestKind};

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
mod tests;
