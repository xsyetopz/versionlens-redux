//! One pooled HTTP/2 client for every registry, with a concurrency cap and a per-request deadline
//! that starts when the request is sent, not while it waits for a slot.

use std::error::Error;
use std::sync::Arc;

use reqwest::{Client, RequestBuilder, Response, StatusCode, Url, header};
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::config::Settings;
use crate::registry::{self, Registry, Versions, expand};

mod gh;

/// The most `Link` pages a paged registry is followed through, past its first.
const PAGES: usize = 30;

/// The `rel="next"` target of a response's `Link` header, resolved against its URL.
fn next_page(response: &Response) -> Option<Url> {
    let link = response.headers().get(header::LINK)?.to_str().ok()?;
    link.split(',').find_map(|part| {
        let (target, params) = part.split_once(';')?;
        params.contains("rel=\"next\"").then_some(())?;
        let target = target.trim().strip_prefix('<')?.strip_suffix('>')?;
        response.url().join(target).ok()
    })
}

/// A cheap-to-clone handle to the shared client.
#[derive(Clone)]
pub(crate) struct Http {
    client: Client,
    /// The client with the other `strictSsl`, built only when a registry overrides it.
    other: Option<Client>,
    permits: Arc<Semaphore>,
    settings: Arc<Settings>,
}

/// Renders an error with every cause, so a log names the real failure (DNS, TLS, proxy) rather
/// than reqwest's generic "error sending request".
pub(crate) fn chain(error: &dyn Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        let cause_text = cause.to_string();
        if !text.contains(&cause_text) {
            text = format!("{text}: {cause_text}");
        }
        source = cause.source();
    }
    text
}

impl Http {
    /// Builds the client for these settings.
    ///
    /// # Errors
    /// Reports a TLS setup failure.
    pub(crate) fn new(settings: Settings) -> Result<Self, String> {
        // reqwest is built without a bundled crypto provider; the first install wins and later calls are no-ops.
        drop(rustls::crypto::ring::default_provider().install_default());
        let build = |strict: bool| {
            Client::builder()
                .user_agent(concat!("versionlens/", env!("CARGO_PKG_VERSION")))
                .connect_timeout(settings.timeout)
                .tls_danger_accept_invalid_certs(!strict)
                .build()
                .map_err(|e| chain(&e))
        };
        let client = build(settings.strict_ssl)?;
        let overridden = settings
            .providers
            .values()
            .any(|p| p.strict_ssl.is_some_and(|s| s != settings.strict_ssl));
        let other = overridden
            .then(|| build(!settings.strict_ssl))
            .transpose()?;
        Ok(Self {
            client,
            other,
            permits: Arc::new(Semaphore::new(settings.concurrency)),
            settings: Arc::new(settings),
        })
    }

    /// Sends a request and returns the body with the URL of the next page its `Link` header names.
    /// An `anonymous` request answers a `WWW-Authenticate: Bearer` challenge with the anonymous
    /// token the challenge's realm issues, as OCI registries such as ghcr.io do for public images.
    async fn send(
        &self,
        request: RequestBuilder,
        url: &str,
        anonymous: bool,
    ) -> Result<(String, Option<Url>), String> {
        let _permit = self.permits.acquire().await.map_err(|e| e.to_string())?;
        let work = async {
            let retry = request.try_clone().filter(|_| anonymous);
            let mut response = request.send().await.map_err(|e| chain(&e))?;
            if response.status() == StatusCode::UNAUTHORIZED
                && let Some(retry) = retry
                && let Some(token) = self.anonymous_token(&response).await
            {
                response = retry
                    .bearer_auth(token)
                    .send()
                    .await
                    .map_err(|e| chain(&e))?;
            }
            let status = response.status();
            if !status.is_success() {
                let hint = match status.as_u16() {
                    403 if url.contains("api.github.com") => {
                        " (rate limited; set versionlens.githubToken or GITHUB_TOKEN, or sign in with gh auth login)"
                    }
                    401 | 403 => " (add credentials for this URL with versionlens.authorization)",
                    _ => "",
                };
                return Err(format!("HTTP {status}{hint}"));
            }
            let next = next_page(&response);
            let body = response.text().await.map_err(|e| chain(&e))?;
            Ok((body, next))
        };
        match timeout(self.settings.timeout, work).await {
            Ok(result) => result.map_err(|e| format!("{url}: {e}")),
            Err(_) => Err(format!(
                "{url}: no response within {:?}",
                self.settings.timeout
            )),
        }
    }

    async fn anonymous_token(&self, response: &Response) -> Option<String> {
        let challenge = response
            .headers()
            .get(header::WWW_AUTHENTICATE)?
            .to_str()
            .ok()?
            .strip_prefix("Bearer ")?;
        let field = |key: &str| {
            challenge.split(',').find_map(|part| {
                part.trim()
                    .strip_prefix(key)?
                    .strip_prefix("=\"")?
                    .strip_suffix('"')
            })
        };
        let mut realm = Url::parse(field("realm")?).ok()?;
        for key in ["service", "scope"] {
            if let Some(value) = field(key) {
                realm.query_pairs_mut().append_pair(key, value);
            }
        }
        let body = self
            .client
            .get(realm)
            .send()
            .await
            .ok()?
            .text()
            .await
            .ok()?;
        let token: serde_json::Value = serde_json::from_str(&body).ok()?;
        token
            .get("token")
            .or_else(|| token.get("access_token"))?
            .as_str()
            .map(str::to_owned)
    }

    /// Fetches a URL as text, with the next page's URL where the response names one.
    ///
    /// # Errors
    /// Names the URL with the failure: connection, status, or deadline.
    async fn get(
        &self,
        client: &Client,
        url: &str,
        accept: &str,
    ) -> Result<(String, Option<Url>), String> {
        let mut request = client.get(url).header(header::ACCEPT, accept);
        // The longest matching prefix wins, so a host-wide value can be narrowed per path.
        let configured = self
            .settings
            .authorization
            .iter()
            .filter(|(prefix, _)| url.starts_with(prefix.as_str()))
            .max_by_key(|(prefix, _)| prefix.len());
        if let Some((_, value)) = configured {
            request = request.header(header::AUTHORIZATION, value);
        } else if url.starts_with("https://api.github.com/") {
            let token = match &self.settings.github_token {
                Some(token) => Some(token.clone()),
                None => gh::token().await,
            };
            if let Some(token) = token {
                request = request.bearer_auth(token);
            }
        }
        self.send(request, url, configured.is_none()).await
    }

    /// Fetches a document as text.
    ///
    /// # Errors
    /// Names the URL with the failure.
    pub(crate) async fn text(&self, url: &str, accept: &str) -> Result<String, String> {
        let fetched = self.get(&self.client, url, accept).await;
        fetched.map(|(body, _)| body)
    }

    /// Posts a JSON body and returns the response text.
    ///
    /// # Errors
    /// Names the URL with the failure.
    pub(crate) async fn post(&self, url: &str, body: String) -> Result<String, String> {
        let request = self
            .client
            .post(url)
            .header(header::CONTENT_TYPE, "application/json")
            .body(body);
        self.send(request, url, false).await.map(|(body, _)| body)
    }

    /// Lists a package's versions from its registry, honouring a configured URL override. Each of
    /// `filters` fetches the registry's filtered page alongside, and its versions join the list.
    ///
    /// # Errors
    /// Explains why the registry could not answer.
    pub(crate) async fn versions(
        &self,
        registry: &'static Registry,
        name: &str,
        source: Option<&str>,
        filters: &[String],
    ) -> Result<Versions, String> {
        let registry = registry::route(registry, name);
        let template = self
            .settings
            .registries
            .get(registry.id)
            .map_or(registry.url, String::as_str);
        let url = expand(template, name, source)?;
        let client = match &self.other {
            Some(other)
                if self.settings.strict_ssl_for(registry.id) != self.settings.strict_ssl =>
            {
                other
            }
            _ => &self.client,
        };
        // Filtered pages belong to the registry's own host; an overridden URL points elsewhere.
        let own = template == registry.url && !registry.filtered.is_empty();
        let mut more = Vec::new();
        for filter in filters.iter().filter(|_| own) {
            let url = expand(registry.filtered, name, source)?.replace("{filter}", filter);
            let (http, client) = (self.clone(), client.clone());
            more.push(tokio::spawn(async move {
                let body = http.get(&client, &url, registry.accept).await;
                (url, body.map(|(body, _)| body))
            }));
        }
        let (url, (body, mut next)) = match self.get(client, &url, registry.accept).await {
            Err(error) if !registry.fallback.is_empty() && template == registry.url => {
                let url = expand(registry.fallback, name, source)?;
                let fetched = self.get(client, &url, registry.accept).await;
                (url, fetched.map_err(|e| format!("{error}; {e}"))?)
            }
            fetched => (url, fetched?),
        };
        let mut versions = registry
            .parse(&body, name)
            .map_err(|e| format!("{url}: {e}"))?;
        // OCI registries such as quay.io cap a tag page however many tags `n` asks for.
        for _ in 0..PAGES {
            let Some(page) = next.take().filter(|_| registry.paged) else {
                break;
            };
            let (body, after) = self.get(client, page.as_str(), registry.accept).await?;
            versions.list.extend(registry.parse(&body, name)?.list);
            next = after;
        }
        for page in more {
            let (url, body) = page.await.map_err(|e| e.to_string())?;
            let extra = registry
                .parse(&body?, name)
                .map_err(|e| format!("{url}: {e}"))?;
            for version in extra.list {
                if !versions.list.contains(&version) {
                    versions.list.push(version);
                }
            }
        }
        if versions.list.is_empty() && versions.yanked.is_empty() {
            return Err(format!("{url}: lists no versions for {name}"));
        }
        Ok(versions)
    }
}

#[cfg(test)]
mod tests;
