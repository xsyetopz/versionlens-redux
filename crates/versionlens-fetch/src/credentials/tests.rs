use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use super::{Credential, CredentialChain, Secret, apply_credential, run_for_output};
use crate::http::{HttpConfig, HttpConfigInput, HttpHeader, HttpHeaderInput};

const API: &str = "https://api.github.com/repos/actions/checkout/tags";
const OTHER: &str = "https://registry.npmjs.org/left-pad";

fn chain(env: &[(&str, &str)], gh: Option<&str>) -> (CredentialChain, Arc<AtomicUsize>) {
    let env: HashMap<String, String> = env
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    let calls = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&calls);
    let gh = gh.map(str::to_owned);
    let chain = CredentialChain::with_sources(
        move |name| env.get(name).cloned(),
        move || {
            counter.fetch_add(1, Ordering::SeqCst);
            gh.clone()
        },
    );
    (chain, calls)
}

fn token(credential: Option<Credential>) -> Option<String> {
    credential.map(|credential| credential.token.expose().to_owned())
}

#[test]
fn credentials_client_supplied_wins_over_environment() {
    let (chain, _) = chain(&[("GITHUB_TOKEN", "env")], Some("gh"));
    chain.set_client_credential("api.github.com", Credential::new("Bearer", "client"));
    assert_eq!(token(chain.resolve(API)), Some("client".to_owned()));
}

#[test]
fn credentials_github_token_wins_over_gh_token() {
    let (chain, _) = chain(&[("GH_TOKEN", "second"), ("GITHUB_TOKEN", "first")], None);
    assert_eq!(token(chain.resolve(API)), Some("first".to_owned()));
}

#[test]
fn credentials_gh_token_is_used_without_github_token() {
    let (chain, _) = chain(&[("GITHUB_TOKEN", "  "), ("GH_TOKEN", "second")], None);
    assert_eq!(token(chain.resolve(API)), Some("second".to_owned()));
}

#[test]
fn credentials_environment_token_becomes_a_bearer_header() {
    let (chain, _) = chain(&[("GITHUB_TOKEN", "abc")], None);
    let header = chain.resolve(API).unwrap().authorization_header();
    assert_eq!(header.name, "Authorization");
    assert_eq!(header.value, "Bearer abc");
}

#[test]
fn credentials_empty_scheme_sends_the_token_alone() {
    let header = Credential::new("", "raw-token").authorization_header();
    assert_eq!(header.value, "raw-token");
}

#[test]
fn credentials_environment_tokens_are_only_for_github() {
    let (chain, _) = chain(&[("GITHUB_TOKEN", "env")], Some("gh"));
    assert_eq!(chain.resolve(OTHER), None);
    assert_eq!(chain.resolve_after_denied(OTHER, 401), None);
}

#[test]
fn credentials_client_token_applies_to_its_own_host_only() {
    let (chain, _) = chain(&[], None);
    chain.set_client_credential("Registry.Example.com", Credential::new("Basic", "x"));
    assert!(chain.resolve("https://registry.example.com/a").is_some());
    assert_eq!(chain.resolve(OTHER), None);
}

#[test]
fn credentials_gh_does_not_run_until_a_request_is_denied() {
    let (chain, calls) = chain(&[], Some("ghtoken"));
    assert_eq!(chain.resolve(API), None);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(chain.resolve_after_denied(API, 404), None);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn credentials_gh_runs_after_401_and_403_and_only_once() {
    let (chain, calls) = chain(&[], Some("ghtoken\n"));
    assert_eq!(
        token(chain.resolve_after_denied(API, 401)),
        Some("ghtoken".to_owned())
    );
    assert_eq!(
        token(chain.resolve_after_denied(API, 403)),
        Some("ghtoken".to_owned())
    );
    assert_eq!(token(chain.resolve(API)), Some("ghtoken".to_owned()));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn credentials_missing_gh_token_is_remembered() {
    let (chain, calls) = chain(&[], None);
    assert_eq!(chain.resolve_after_denied(API, 403), None);
    assert_eq!(chain.resolve_after_denied(API, 401), None);
    assert_eq!(chain.resolve(API), None);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn credentials_gh_is_skipped_when_an_earlier_source_exists() {
    let (chain, calls) = chain(&[("GH_TOKEN", "env")], Some("gh"));
    assert_eq!(chain.resolve_after_denied(API, 401), None);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

#[test]
fn credentials_gh_runs_once_across_clones() {
    let (chain, calls) = chain(&[], Some("gh"));
    let clone = chain.clone();
    chain.resolve_after_denied(API, 401);
    clone.resolve_after_denied(API, 401);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[test]
fn credentials_chain_replaces_config_file_authorization() {
    let mut headers = vec![
        HttpHeader {
            name: "authorization".to_owned(),
            value: "Bearer from-npmrc".to_owned(),
            url: None,
        },
        HttpHeader {
            name: "X-Other".to_owned(),
            value: "keep".to_owned(),
            url: None,
        },
    ];
    apply_credential(&mut headers, &Credential::new("Bearer", "chain"));
    let values: Vec<_> = headers.iter().map(|header| header.value.as_str()).collect();
    assert_eq!(values, ["keep", "Bearer chain"]);
}

#[test]
fn credentials_subprocess_is_killed_after_the_timeout() {
    let started = Instant::now();
    let output = run_for_output("sleep", &["30"], Duration::from_millis(200));
    assert_eq!(output, None);
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn credentials_subprocess_output_is_returned() {
    let output = run_for_output("echo", &["abc"], Duration::from_secs(5));
    assert_eq!(output.as_deref().map(str::trim), Some("abc"));
}

#[test]
fn credentials_missing_subprocess_yields_nothing() {
    assert_eq!(
        run_for_output("versionlens-no-such-program", &[], Duration::from_secs(1)),
        None
    );
}

const SECRET: &str = "ghp_SuperSecretTokenValue123";

fn assert_clean(output: &str) {
    assert!(!output.contains(SECRET), "token leaked: {output}");
}

#[test]
fn credentials_redaction_secret_and_credential_debug() {
    assert_clean(&format!("{:?}", Secret::new(SECRET)));
    assert_clean(&format!("{:#?}", Credential::new("Bearer", SECRET)));
    assert!(format!("{:?}", Secret::new(SECRET)).contains("[redacted]"));
}

#[test]
fn credentials_redaction_chain_debug() {
    let (chain, _) = chain(&[("GITHUB_TOKEN", SECRET)], Some(SECRET));
    chain.set_client_credential("api.github.com", Credential::new("Bearer", SECRET));
    chain.resolve_after_denied(API, 401);
    assert_clean(&format!("{chain:?}"));
    assert_clean(&format!("{chain:#?}"));
}

#[test]
fn credentials_redaction_http_config_debug() {
    let mut headers = Vec::new();
    apply_credential(&mut headers, &Credential::new("Bearer", SECRET));
    let config = HttpConfig {
        proxy: Some(format!("http://user:{SECRET}@proxy.example:8080")),
        key: Some(SECRET.to_owned()),
        auth_headers: headers,
        ..HttpConfig::default()
    };
    assert_clean(&format!("{config:?}"));
    assert_clean(&format!("{config:#?}"));

    let input = HttpConfigInput {
        timeout_ms: None,
        strict_ssl: None,
        proxy: Some(format!("http://user:{SECRET}@proxy.example:8080")),
        ca_file: None,
        ca: None,
        cert_file: None,
        key_file: None,
        cert: None,
        key: Some(SECRET.to_owned()),
        auth_headers: Some(vec![HttpHeaderInput::new(
            "Authorization",
            format!("Bearer {SECRET}"),
            None,
        )]),
    };
    assert_clean(&format!("{input:?}"));
}
