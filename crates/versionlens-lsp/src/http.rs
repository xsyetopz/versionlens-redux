//! One pooled HTTP/2 client for every registry, with a concurrency cap and a per-request deadline
//! that starts when the request is sent, not while it waits for a slot.

use std::error::Error;
use std::sync::Arc;

use reqwest::{Client, RequestBuilder, header};
use tokio::sync::Semaphore;
use tokio::time::timeout;

use crate::config::Settings;
use crate::registry::{Registry, Versions, expand};

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

    async fn send(&self, request: RequestBuilder, url: &str) -> Result<String, String> {
        let _permit = self.permits.acquire().await.map_err(|e| e.to_string())?;
        let work = async {
            let response = request.send().await.map_err(|e| chain(&e))?;
            let status = response.status();
            if !status.is_success() {
                let hint = match status.as_u16() {
                    403 if url.contains("api.github.com") => {
                        " (rate limited; set GITHUB_TOKEN or versionlens.githubToken)"
                    }
                    401 | 403 => " (add credentials for this URL with versionlens.authorization)",
                    _ => "",
                };
                return Err(format!("HTTP {status}{hint}"));
            }
            response.text().await.map_err(|e| chain(&e))
        };
        match timeout(self.settings.timeout, work).await {
            Ok(result) => result.map_err(|e| format!("{url}: {e}")),
            Err(_) => Err(format!(
                "{url}: no response within {:?}",
                self.settings.timeout
            )),
        }
    }

    /// Fetches a URL as text.
    ///
    /// # Errors
    /// Names the URL with the failure: connection, status, or deadline.
    async fn get(&self, client: &Client, url: &str, accept: &str) -> Result<String, String> {
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
        } else if url.starts_with("https://api.github.com/")
            && let Some(token) = &self.settings.github_token
        {
            request = request.bearer_auth(token);
        }
        self.send(request, url).await
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
        self.send(request, url).await
    }

    /// Lists a package's versions from its registry, honouring a configured URL override.
    ///
    /// # Errors
    /// Explains why the registry could not answer.
    pub(crate) async fn versions(
        &self,
        registry: &Registry,
        name: &str,
        source: Option<&str>,
    ) -> Result<Versions, String> {
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
        let body = self.get(client, &url, registry.accept).await?;
        let versions = registry
            .parse(&body, name)
            .map_err(|e| format!("{url}: {e}"))?;
        if versions.list.is_empty() && versions.yanked.is_empty() {
            return Err(format!("{url}: lists no versions for {name}"));
        }
        Ok(versions)
    }
}
