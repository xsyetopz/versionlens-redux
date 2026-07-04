use versionlens_http::{
    ACCEPT_GITHUB_V3, ACCEPT_JSON, HttpError, RetryPolicy, get_text_with_accept_and_retry,
};
use versionlens_parsers::Ecosystem;
use versionlens_providers::http_status_message_from_code;

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;

impl VersionLensSession {
    pub(in crate::fetch) fn get_text_or_status_with_context(
        &self,
        url: &str,
        ecosystem: Ecosystem,
        context: &RegistryContext,
    ) -> Result<Option<String>, FetchError> {
        if let Some(body) = self.cached_request_body(url) {
            return Ok(Some(body));
        }

        let request_lock = self.request_lock(url);
        let _request_guard = request_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(body) = self.cached_request_body(url) {
            return Ok(Some(body));
        }

        let auth_headers = context.auth_headers_for_url(ecosystem, url);
        let base_config =
            self.http_config_with_headers(ecosystem, context.manifest_kind(), &auth_headers);
        let http_config = context.http_config_for_request(ecosystem, url, base_config);
        match get_text_with_accept_and_retry(
            url,
            &http_config,
            accept_header_for_request(ecosystem, url),
            retry_policy_for_request(ecosystem, url),
        ) {
            Ok(body) => {
                self.cache_request_body(url, &body, ecosystem, context.manifest_kind());
                Ok(Some(body))
            }
            Err(error) => match http_status_message(&error) {
                Some(message) => {
                    if error.status_code() == Some(401) {
                        let auth_url = self.authorization_url_for_request(url);
                        self.record_authorization_request(auth_url, url.to_owned());
                    }
                    Err(FetchError::RegistryStatus(message.to_owned()))
                }
                None => Err(anyhow::Error::new(error)
                    .context(format!("failed to fetch registry URL {url}"))
                    .into()),
            },
        }
    }
}

fn accept_header_for_request(ecosystem: Ecosystem, url: &str) -> Option<&'static str> {
    if starts_with_ignore_ascii_case(url, "https://api.github.com/repos/") {
        return Some(ACCEPT_GITHUB_V3);
    }
    match ecosystem {
        Ecosystem::Go | Ecosystem::Maven | Ecosystem::Npm | Ecosystem::Python => None,
        _ => Some(ACCEPT_JSON),
    }
}

fn retry_policy_for_request(ecosystem: Ecosystem, url: &str) -> RetryPolicy {
    match ecosystem {
        Ecosystem::Npm if !starts_with_ignore_ascii_case(url, "https://api.github.com/repos/") => {
            RetryPolicy::npm_registry_fetch()
        }
        _ => RetryPolicy::disabled(),
    }
}

fn http_status_message(error: &HttpError) -> Option<&'static str> {
    http_status_message_from_code(error.status_code()?)
}

impl VersionLensSession {
    pub(crate) fn authorization_url_for_request(&self, request_url: &str) -> String {
        self.config
            .http
            .auth_headers
            .iter()
            .filter_map(|header| header.url.as_deref())
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .find(|url| starts_with_ignore_ascii_case(request_url, url))
            .map(str::to_owned)
            .or_else(|| url_origin(request_url))
            .unwrap_or_else(|| request_url.to_owned())
    }
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}

fn url_origin(url: &str) -> Option<String> {
    let scheme_end = url.find("://")? + 3;
    let path_start = url[scheme_end..]
        .find('/')
        .map_or(url.len(), |index| scheme_end + index);
    Some(url[..path_start].to_owned())
}

#[cfg(test)]
mod tests;
