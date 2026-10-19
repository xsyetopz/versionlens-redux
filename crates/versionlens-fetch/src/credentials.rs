//! Credential sources for registry requests, in priority order.
//!
//! For GitHub hosts the chain is: a credential the client supplied, then
//! `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token`, and only then the
//! registry configuration files, which already arrive as
//! [`HttpConfig::auth_headers`](crate::HttpConfig). `gh` runs at most once per
//! session, under a short timeout, and only after GitHub refused a request.
//! Tokens are held in [`Secret`], which never prints its value.

use std::collections::HashMap;
use std::fmt;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use reqwest::Url;

use crate::http::HttpHeader;

/// How long `gh auth token` may run before it is killed.
pub const GH_TIMEOUT: Duration = Duration::from_secs(2);

const GITHUB_API_HOST: &str = "api.github.com";

/// A token that prints as `[redacted]` in `Debug` and has no `Display`,
/// `Serialize` or `Deserialize`, so it cannot reach a log or the cache by
/// accident.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The raw value, for building a request header only.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

/// An authorization scheme and its token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credential {
    pub scheme: String,
    pub token: Secret,
}

impl Credential {
    pub fn new(scheme: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            scheme: scheme.into(),
            token: Secret::new(token),
        }
    }

    fn bearer(token: String) -> Self {
        Self::new("Bearer", token)
    }

    /// The `Authorization` header that carries this credential. An empty
    /// scheme sends the token as the whole value.
    pub fn authorization_header(&self) -> HttpHeader {
        let value = if self.scheme.is_empty() {
            self.token.expose().to_owned()
        } else {
            format!("{} {}", self.scheme, self.token.expose())
        };
        HttpHeader {
            name: "Authorization".to_owned(),
            value,
            url: None,
        }
    }
}

type EnvReader = dyn Fn(&str) -> Option<String> + Send + Sync;
type GhRunner = dyn Fn() -> Option<String> + Send + Sync;

/// Session-scoped credential sources. Clones share state.
#[derive(Clone)]
pub struct CredentialChain {
    client: Arc<RwLock<HashMap<String, Credential>>>,
    env: Arc<EnvReader>,
    gh: Arc<GhRunner>,
    /// Unset until `gh` has run; then its result, including "no token".
    gh_result: Arc<OnceLock<Option<Credential>>>,
    github_host: String,
}

impl fmt::Debug for CredentialChain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialChain")
            .field("client_hosts", &self.client_hosts())
            .finish_non_exhaustive()
    }
}

impl Default for CredentialChain {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialChain {
    /// A chain that reads the process environment and runs the real `gh`.
    pub fn new() -> Self {
        Self::with_sources(
            |name| std::env::var(name).ok(),
            || run_gh_auth_token(GH_TIMEOUT),
        )
    }

    /// A chain with injected environment and `gh` sources.
    pub fn with_sources(
        env: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
        gh: impl Fn() -> Option<String> + Send + Sync + 'static,
    ) -> Self {
        Self {
            client: Arc::default(),
            env: Arc::new(env),
            gh: Arc::new(gh),
            gh_result: Arc::default(),
            github_host: GITHUB_API_HOST.to_owned(),
        }
    }

    /// Treats `host` as the GitHub API host, for a test server or an
    /// Enterprise instance.
    #[must_use]
    pub fn with_github_host(mut self, host: &str) -> Self {
        self.github_host = host.to_ascii_lowercase();
        self
    }

    /// Records the credential a client supplied for `host`, used for the rest
    /// of the session. The server calls this when `versionlens/authorization`
    /// is answered.
    pub fn set_client_credential(&self, host: &str, credential: Credential) {
        self.client
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(host.to_ascii_lowercase(), credential);
    }

    /// The credential to send to `url` from the client, the environment or an
    /// earlier `gh` run, or `None` to keep the configuration-file headers.
    pub fn resolve(&self, url: &str) -> Option<Credential> {
        let host = host_of(url)?;
        self.preset_credential(&host).or_else(|| {
            if host == self.github_host {
                self.gh_result.get().cloned().flatten()
            } else {
                None
            }
        })
    }

    /// Called after `status` refused a request to `url`. For GitHub with no
    /// client or environment token, runs `gh auth token` (once per session)
    /// and returns its credential. Blocks for up to [`GH_TIMEOUT`] on the
    /// first call; run it on a blocking thread.
    pub fn resolve_after_denied(&self, url: &str, status: u16) -> Option<Credential> {
        let host = host_of(url)?;
        if !matches!(status, 401 | 403 | 429)
            || host != self.github_host
            || self.preset_credential(&host).is_some()
        {
            return None;
        }
        self.gh_result
            .get_or_init(|| {
                (self.gh)()
                    .map(|token| token.trim().to_owned())
                    .filter(|token| !token.is_empty())
                    .map(Credential::bearer)
            })
            .clone()
    }

    /// The client or environment credential for `host`, which need no
    /// subprocess.
    fn preset_credential(&self, host: &str) -> Option<Credential> {
        let supplied = self
            .client
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(host)
            .cloned();
        if supplied.is_some() || host != self.github_host {
            return supplied;
        }
        ["GITHUB_TOKEN", "GH_TOKEN"]
            .into_iter()
            .filter_map(|name| (self.env)(name))
            .map(|token| token.trim().to_owned())
            .find(|token| !token.is_empty())
            .map(Credential::bearer)
    }

    fn client_hosts(&self) -> Vec<String> {
        let mut hosts: Vec<_> = self
            .client
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .cloned()
            .collect();
        hosts.sort();
        hosts
    }
}

/// Puts the chain's credential in place of any configuration-file
/// `Authorization` header, which ranks below it.
pub fn apply_credential(headers: &mut Vec<HttpHeader>, credential: &Credential) {
    headers.retain(|header| !header.name.eq_ignore_ascii_case("authorization"));
    headers.push(credential.authorization_header());
}

/// The lower-case host of `url` without its port, which is how credentials
/// are keyed.
pub fn host_of(url: &str) -> Option<String> {
    Some(Url::parse(url).ok()?.host_str()?.to_ascii_lowercase())
}

/// Runs `gh auth token`, killing it after `timeout`. Returns `None` when `gh`
/// is missing, fails, times out or prints nothing. Its stderr is discarded.
fn run_gh_auth_token(timeout: Duration) -> Option<String> {
    run_for_output("gh", &["auth", "token"], timeout)
}

fn run_for_output(program: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let deadline = Instant::now() + timeout;
    let status = loop {
        let Ok(polled) = child.try_wait() else {
            let _ = child.kill();
            let _ = child.wait();
            return None;
        };
        match polled {
            Some(status) => break status,
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    if !status.success() {
        return None;
    }
    let mut output = String::new();
    child.stdout.take()?.read_to_string(&mut output).ok()?;
    Some(output)
}

#[cfg(test)]
mod tests;
