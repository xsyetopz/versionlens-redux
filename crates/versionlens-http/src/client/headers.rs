use ureq::RequestBuilder;

use crate::config::HttpHeader;

const ACCEPT_HEADER: &str = "accept";
const USER_AGENT_HEADER: &str = "user-agent";
const USER_AGENT_VALUE: &str = "vscode-versionlens (gitlab.com/versionlens/vscode-versionlens)";

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
    header
        .url
        .as_deref()
        .map(|value| value.trim())
        .filter(|pattern| !pattern.is_empty())
        .is_none_or(|pattern| starts_with_ignore_ascii_case(url, pattern))
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(prefix))
}
