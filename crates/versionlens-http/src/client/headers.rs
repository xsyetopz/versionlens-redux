use ureq::{RequestBuilder, http::Uri};

use crate::config::HttpHeader;

const ACCEPT_HEADER: &str = "accept";
const USER_AGENT_HEADER: &str = "user-agent";
const USER_AGENT_VALUE: &str = "versionlens-redux (github.com/xsyetopz/versionlens-redux)";

pub(crate) fn request_with_headers<B>(
    request: RequestBuilder<B>,
    url: &str,
    headers: &[HttpHeader],
    accept: Option<&str>,
) -> RequestBuilder<B> {
    let mut request = request.header(USER_AGENT_HEADER, USER_AGENT_VALUE);
    if let Some(accept) = accept {
        request = request.header(ACCEPT_HEADER, accept);
    }

    for header in headers {
        if !matches_header_url(header, url) {
            continue;
        }
        request = request.header(header.name.as_str(), header.value.as_str());
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
    let Ok(auth_url) = auth_url.parse::<Uri>() else {
        return false;
    };
    let Ok(request_url) = request_url.parse::<Uri>() else {
        return false;
    };

    same_origin(&auth_url, &request_url) && path_contains(auth_url.path(), request_url.path())
}

fn same_origin(auth_url: &Uri, request_url: &Uri) -> bool {
    let Some(auth_scheme) = auth_url.scheme_str() else {
        return false;
    };
    let Some(request_scheme) = request_url.scheme_str() else {
        return false;
    };
    let Some(auth_host) = auth_url.host() else {
        return false;
    };
    let Some(request_host) = request_url.host() else {
        return false;
    };

    auth_scheme.eq_ignore_ascii_case(request_scheme)
        && auth_host.eq_ignore_ascii_case(request_host)
        && effective_port(auth_url) == effective_port(request_url)
}

fn effective_port(url: &Uri) -> Option<u16> {
    url.port_u16().or_else(|| match url.scheme_str() {
        Some(scheme) if scheme.eq_ignore_ascii_case("http") => Some(80),
        Some(scheme) if scheme.eq_ignore_ascii_case("https") => Some(443),
        _ => None,
    })
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
