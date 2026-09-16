use reqwest::{RequestBuilder, Url};

use crate::config::HttpHeader;

const USER_AGENT_VALUE: &str = "versionlens-redux (github.com/xsyetopz/versionlens-redux)";

pub(crate) fn request_with_headers(
    mut request: RequestBuilder,
    url: &str,
    headers: &[HttpHeader],
    accept: Option<&str>,
) -> RequestBuilder {
    request = request
        .header(reqwest::header::USER_AGENT, USER_AGENT_VALUE)
        .header(reqwest::header::ACCEPT_ENCODING, "gzip");
    if let Some(accept) = accept {
        request = request.header(reqwest::header::ACCEPT, accept);
    }

    for header in headers {
        if matches_header_url(header, url) {
            request = request.header(header.name.as_str(), header.value.as_str());
        }
    }
    request
}

fn matches_header_url(header: &HttpHeader, url: &str) -> bool {
    match header.url.as_deref() {
        None => true,
        Some(auth_url) => urls_share_auth_scope(auth_url.trim(), url),
    }
}

fn urls_share_auth_scope(auth_url: &str, request_url: &str) -> bool {
    let Ok(auth_url) = Url::parse(auth_url) else {
        return false;
    };
    let Ok(request_url) = Url::parse(request_url) else {
        return false;
    };

    auth_url.scheme().eq_ignore_ascii_case(request_url.scheme())
        && auth_url.host_str().is_some_and(|host| {
            request_url
                .host_str()
                .is_some_and(|request_host| host.eq_ignore_ascii_case(request_host))
        })
        && auth_url.port_or_known_default() == request_url.port_or_known_default()
        && path_contains(auth_url.path(), request_url.path())
}

fn path_contains(auth_path: &str, request_path: &str) -> bool {
    let auth_path = auth_path.trim_end_matches('/');
    let request_path = request_path.trim_end_matches('/');

    auth_path.is_empty()
        || request_path == auth_path
        || request_path
            .strip_prefix(auth_path)
            .is_some_and(|suffix| suffix.starts_with('/'))
}
