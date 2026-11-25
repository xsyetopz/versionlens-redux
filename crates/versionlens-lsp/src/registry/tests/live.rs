//! Live checks of every registry row, its package page, and its OSV ecosystem. They need the
//! network, so they are ignored by default; `just registries-live` runs them. A registry that moves
//! its API, a page template that stops resolving, or an ecosystem OSV renames fails here first.

use std::time::Duration;

use super::super::tools::{self, ROWS};
use super::*;
use crate::config::Settings;
use crate::http::Http;
use crate::osv::{self, Query};

/// A registry id, a package it lists, the repository a manifest would name for it, and a
/// version OSV reports a vulnerability for, where the registry has an OSV ecosystem.
const SAMPLES: &[(&str, &str, Option<&str>, Option<&str>)] = &[
    ("cargo", "time", None, Some("0.1.43")),
    ("npm", "lodash", None, Some("4.17.15")),
    ("jsr", "@std/path", None, None),
    ("unity", "com.unity.textmeshpro", None, None),
    ("pypi", "jinja2", None, Some("2.10")),
    ("composer", "symfony/http-kernel", None, Some("v4.4.0")),
    ("go", "golang.org/x/text", None, Some("v0.3.5")),
    ("rubygems", "nokogiri", None, Some("1.10.0")),
    ("cocoapods", "Alamofire", None, None),
    ("pub", "http", None, Some("0.13.0")),
    ("nuget", "Newtonsoft.Json", None, Some("12.0.1")),
    (
        "maven",
        "org.apache.logging.log4j:log4j-core",
        None,
        Some("2.14.1"),
    ),
    ("clojars", "org.clojure/clojure", None, Some("1.8.0")),
    ("clojars", "ring/ring-core", None, None),
    ("hex", "plug", None, Some("1.3.0")),
    ("julia", "HTTP", None, Some("0.9.0")),
    ("haxelib", "hxcpp", None, None),
    ("docker", "node", None, None),
    ("oci", "mcr.microsoft.com/dotnet/sdk", None, None),
    ("oci", "quay.io/prometheus/prometheus", None, None),
    ("oci", "ghcr.io/home-assistant/home-assistant", None, None),
    ("oci", "registry.k8s.io/pause", None, None),
    ("oci", "gcr.io/distroless/static", None, None),
    ("oci", "public.ecr.aws/docker/library/node", None, None),
    (
        "oci",
        "registry.gitlab.com/gitlab-org/gitlab-runner",
        None,
        None,
    ),
    ("github", "madler/zlib", None, Some("v1.2.11")),
    ("swift", "vapor/vapor", None, Some("4.0.0")),
    (
        "helm",
        "prometheus",
        Some("https://prometheus-community.github.io/helm-charts"),
        None,
    ),
    ("galaxy", "community.general", None, None),
    ("terraform", "hashicorp/aws", None, None),
    ("terraform", "terraform-aws-modules/vpc/aws", None, None),
    ("bazel", "rules_go", None, None),
    ("vcpkg", "fmt", None, None),
    ("conan", "zlib", None, None),
    ("cran", "commonmark", None, Some("1.7")),
    ("cpan", "Moose", None, None),
    ("dub", "vibe-d", None, None),
    ("hackage", "aeson", None, Some("1.0.0.0")),
    ("opam", "tls", None, Some("0.1.0")),
    ("puppet", "puppetlabs:stdlib", None, None),
    ("chef", "nginx", None, None),
    ("rust", "rust", None, None),
    ("python", "python", None, Some("3.8.0")),
    ("ruby", "ruby", None, Some("3.0.0")),
    ("nodejs", "node", None, Some("20.0.0")),
    ("ghc", "ghc", None, Some("9.2.1")),
    ("golang", "golang", None, Some("1.20.0")),
    ("terraform-cli", "terraform", None, None),
    ("gradle-dist", "gradle", None, Some("7.0.0")),
    ("conda", "numpy", None, None),
    ("elm", "elm/core", None, None),
    ("luarocks", "luasocket", None, None),
    ("xmake", "zlib", None, None),
];

/// How many times a page behind a bot challenge is asked before the check fails.
const CHALLENGE_RETRIES: u64 = 6;

fn block_on<F: Future>(work: F) -> F::Output {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(work)
}

#[test]
fn samples_cover_every_registry() {
    let missing: Vec<&str> = REGISTRIES
        .iter()
        .chain(ROWS)
        .map(|r| r.id)
        .filter(|id| !SAMPLES.iter().any(|(sample, ..)| sample == id))
        .collect();
    assert!(missing.is_empty(), "no live sample for {missing:?}");
    let silent: Vec<&str> = REGISTRIES
        .iter()
        .chain(ROWS)
        .filter(|r| r.osv.is_some())
        .map(|r| r.id)
        .filter(|id| {
            !SAMPLES
                .iter()
                .any(|(sample, _, _, vulnerable)| sample == id && vulnerable.is_some())
        })
        .collect();
    assert!(silent.is_empty(), "no vulnerable sample for {silent:?}");
}

/// Every page template, host-specific ones included, is one a sample opens, so the live page
/// check covers each link the server can show.
#[test]
fn samples_open_every_page_template() {
    let opened: Vec<String> = SAMPLES
        .iter()
        .map(|&(id, name, ..)| page(id, name))
        .collect();
    let unopened: Vec<&str> = PAGES
        .iter()
        .chain(tools::PAGES)
        .map(|&(key, template)| (key, template.split('{').next().unwrap_or(template)))
        .filter(|(_, stem)| !opened.iter().any(|url| url.starts_with(stem)))
        .map(|(key, _)| key)
        .collect();
    assert!(
        unopened.is_empty(),
        "no sample opens the page for {unopened:?}"
    );
}

#[test]
#[ignore = "needs the network"]
fn every_registry_lists_versions() {
    let http = Http::new(Settings::default()).unwrap();
    let failures: Vec<String> = block_on(async {
        let tasks: Vec<_> = SAMPLES
            .iter()
            .map(|&(id, name, source, _)| {
                let http = http.clone();
                tokio::spawn(async move {
                    http.versions(get(id).unwrap(), name, source, &[])
                        .await
                        .err()
                        .map(|e| format!("{id} {name}: {e}"))
                })
            })
            .collect();
        let mut failures = Vec::new();
        for task in tasks {
            failures.extend(task.await.unwrap());
        }
        failures
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "needs the network"]
fn every_package_page_resolves() {
    // Run alone, the test builds the first client, so it installs the provider `Http::new` would.
    drop(rustls::crypto::ring::default_provider().install_default());
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (versionlens live check)")
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();
    let failures: Vec<String> = block_on(async {
        let tasks: Vec<_> = SAMPLES
            .iter()
            .map(|&(id, name, ..)| {
                let (client, url) = (client.clone(), page(id, name));
                tokio::spawn(async move {
                    // A Cloudflare challenge, as npmjs.com sends some requests, says nothing about
                    // the page, so it is asked again until the page itself answers.
                    let mut last = String::new();
                    for attempt in 0..CHALLENGE_RETRIES {
                        tokio::time::sleep(Duration::from_secs(attempt)).await;
                        match client.get(&url).header("accept", "text/html").send().await {
                            Ok(response) if response.status().is_success() => return None,
                            Ok(response) if response.headers().contains_key("cf-mitigated") => {
                                last = format!("{id} {url}: Cloudflare challenge on every try");
                            }
                            Ok(response) => {
                                return Some(format!("{id} {url}: HTTP {}", response.status()));
                            }
                            Err(e) => {
                                return Some(format!("{id} {url}: {}", crate::http::chain(&e)));
                            }
                        }
                    }
                    Some(last)
                })
            })
            .collect();
        let mut failures = Vec::new();
        for task in tasks {
            failures.extend(task.await.unwrap());
        }
        failures
    });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
#[ignore = "needs the network"]
fn osv_reports_each_ecosystem() {
    let queries: Vec<Query> = SAMPLES
        .iter()
        .filter_map(|&(id, name, _, vulnerable)| {
            let registry = get(id).unwrap();
            Some(Query {
                ecosystem: registry.osv?,
                name: expand(registry.osv_name, name, None).unwrap(),
                version: vulnerable?.to_owned(),
            })
        })
        .collect();
    let http = Http::new(Settings::default()).unwrap();
    let results = block_on(osv::query(&http, &queries)).unwrap();
    let silent: Vec<String> = queries
        .iter()
        .zip(&results)
        .filter(|(_, ids)| ids.is_empty())
        .map(|(q, _)| format!("{} {}@{}", q.ecosystem, q.name, q.version))
        .collect();
    assert!(silent.is_empty(), "OSV reports nothing for {silent:?}");
}
