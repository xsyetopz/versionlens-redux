//! Live checks of every registry row, its package page, and its OSV ecosystem. They need the
//! network, so they are ignored by default; `just registries-live` runs them. A registry that moves
//! its API, a page template that stops resolving, or an ecosystem OSV renames fails here first.

use std::time::Duration;

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
    ("jsr", "@hono/hono", None, Some("4.0.0")),
    ("unity", "com.unity.textmeshpro", None, None),
    ("pypi", "jinja2", None, Some("2.10")),
    ("composer", "symfony/http-kernel", None, Some("v4.4.0")),
    ("go", "golang.org/x/text", None, Some("v0.3.5")),
    ("rubygems", "nokogiri", None, Some("1.10.0")),
    ("cocoapods", "Alamofire", None, None),
    ("cocoapods", "SQLCipher", None, Some("4.0.0")),
    ("pub", "http", None, Some("0.13.0")),
    ("nuget", "Newtonsoft.Json", None, Some("12.0.1")),
    (
        "maven",
        "org.apache.logging.log4j:log4j-core",
        None,
        Some("2.14.1"),
    ),
    ("gradle-plugins", "org.springframework.boot", None, None),
    ("clojars", "org.clojure/clojure", None, Some("1.8.0")),
    ("clojars", "ring/ring-core", None, None),
    ("hex", "plug", None, Some("1.3.0")),
    ("julia", "HTTP", None, Some("0.9.0")),
    ("haxelib", "hxcpp", None, Some("4.3.2")),
    ("docker", "node", None, Some("20.0.0")),
    ("oci", "mcr.microsoft.com/dotnet/sdk", None, None),
    (
        "oci",
        "quay.io/prometheus/prometheus",
        None,
        Some("v2.40.0"),
    ),
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
        Some("25.0.0"),
    ),
    ("galaxy", "community.general", None, Some("1.0.0")),
    ("terraform", "hashicorp/aws", None, None),
    ("terraform", "hashicorp/vault", None, Some("2.0.0")),
    ("terraform", "terraform-aws-modules/vpc/aws", None, None),
    ("bazel", "rules_go", None, None),
    ("bazel", "zlib", None, Some("1.2.11")),
    ("vcpkg", "fmt", None, None),
    ("vcpkg", "zlib", None, Some("1.2.11")),
    ("conan", "openssl", None, Some("1.1.1f")),
    ("cran", "commonmark", None, Some("1.7")),
    ("cpan", "Moose", None, None),
    ("cpan", "DBI", None, Some("1.640")),
    ("dub", "vibe-d", None, Some("0.9.0")),
    ("hackage", "aeson", None, Some("1.0.0.0")),
    ("opam", "tls", None, Some("0.1.0")),
    ("puppet", "puppetlabs:stdlib", None, None),
    ("puppet", "puppetlabs:apache", None, Some("1.0.0")),
    ("chef", "nginx", None, Some("2.0.0")),
    ("rust", "rust", None, Some("1.77.1")),
    ("python", "python", None, Some("3.8.0")),
    ("ruby", "ruby", None, Some("3.0.0")),
    ("nodejs", "node", None, Some("20.0.0")),
    ("ghc", "ghc", None, Some("9.2.1")),
    ("golang", "golang", None, Some("1.20.0")),
    ("terraform-cli", "terraform", None, Some("1.0.0")),
    ("gradle-dist", "gradle", None, Some("7.0.0")),
    ("conda", "numpy", None, None),
    ("conda", "zlib", None, Some("1.2.11")),
    ("elm", "elm/core", None, Some("1.0.2")),
    ("luarocks", "luasocket", None, None),
    ("luarocks", "lua-resty-jwt", None, Some("0.2.0-0")),
    ("xmake", "zlib", None, Some("v1.2.11")),
    (
        "artifacthub",
        "prometheus-community/prometheus",
        None,
        Some("25.0.0"),
    ),
    ("nixhub", "nodejs", None, None),
    ("vagrant", "bento/ubuntu-22.04", None, None),
    ("packer", "hashicorp/amazon", None, Some("v1.0.0")),
    ("spack", "zlib", None, None),
    ("pear", "Console_Table", None, None),
    ("pecl", "igbinary", None, None),
    ("hunter", "Boost", None, None),
    ("platformio", "bblanchon/ArduinoJson", None, None),
    ("platformio", "knolleary/PubSubClient", None, Some("2.6")),
    ("platformio-platform", "espressif32", None, None),
    ("platformio-tool", "platformio/toolchain-xtensa", None, None),
    ("esp", "espressif/led_strip", None, None),
];

/// Registries whose advisory route works but whose packages have no advisory on file that a
/// sample could show: OSV's GIT ecosystem lists none for vibe.d, the Chef cookbooks, Elm's core
/// packages, or hxcpp, and OSV's Go ecosystem none for the Packer plugins `hashicorp` publishes. Their samples
/// check only that the route answers and names a repository.
const NO_KNOWN_ADVISORY: &[&str] = &["dub", "chef", "elm", "haxelib", "packer"];

/// How many times a page behind a bot challenge is asked before the check fails.
const CHALLENGE_RETRIES: u64 = 10;

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
        .map(|r| r.id)
        .filter(|id| !SAMPLES.iter().any(|(sample, ..)| sample == id))
        .collect();
    assert!(missing.is_empty(), "no live sample for {missing:?}");
    let silent: Vec<&str> = REGISTRIES
        .iter()
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
    let unopened: Vec<String> = REGISTRIES
        .iter()
        .flat_map(|r| {
            let hosts = r
                .pages
                .iter()
                .map(|&(host, t)| (format!("{} {host}", r.id), t));
            std::iter::once((r.id.to_owned(), r.page)).chain(hosts)
        })
        .filter(|(_, template)| !template.is_empty())
        .filter(|(_, template)| {
            let stem = template.split('{').next().unwrap_or(template);
            !opened.iter().any(|url| url.starts_with(stem))
        })
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
                    http.versions(get(id).unwrap(), name, source, &[], &[])
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
                        let mut request = client.get(&url).header("accept", "text/html");
                        if let Some((agent, cookie)) = npm_clearance(&url) {
                            request = request.header("user-agent", agent).header("cookie", cookie);
                        }
                        match request.send().await {
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

/// The browser and `cf_clearance` cookie of a Cloudflare challenge the user solved on
/// npmjs.com, from `VERSIONLENS_NPM_USER_AGENT` and `VERSIONLENS_NPM_COOKIE`. Cloudflare binds the
/// cookie to that browser's user agent and IP, and it goes to npmjs.com alone.
fn npm_clearance(url: &str) -> Option<(String, String)> {
    url.starts_with("https://www.npmjs.com/").then_some(())?;
    let agent = std::env::var("VERSIONLENS_NPM_USER_AGENT").ok()?;
    Some((agent, std::env::var("VERSIONLENS_NPM_COOKIE").ok()?))
}

#[test]
#[ignore = "needs the network"]
fn osv_reports_each_ecosystem() {
    let (ids, queries): (Vec<&str>, Vec<Query>) = SAMPLES
        .iter()
        .filter_map(|&(id, name, source, vulnerable)| {
            let registry = get(id).unwrap();
            let (ecosystem, version) = (registry.osv?, vulnerable?.to_owned());
            let name = expand(registry.osv_name, name, source).unwrap();
            let query = Query {
                ecosystem,
                name: name.replace("{version}", &version),
                version,
                upstream: registry.upstream,
            };
            Some((id, query))
        })
        .unzip();
    let http = Http::new(Settings::default()).unwrap();
    let resolved = block_on(osv::upstream::resolve(&http, &queries)).unwrap();
    let unnamed: Vec<&str> = queries
        .iter()
        .zip(&resolved)
        .filter(|(_, r)| r.is_none())
        .map(|(q, _)| q.name.as_str())
        .collect();
    assert!(unnamed.is_empty(), "no repository found in {unnamed:?}");
    let results = block_on(osv::ids(&http, &resolved)).unwrap();
    let silent: Vec<String> = ids
        .iter()
        .zip(queries.iter().zip(&results))
        .filter(|(id, (_, found))| found.is_empty() && !NO_KNOWN_ADVISORY.contains(id))
        .map(|(_, (q, _))| format!("{} {}@{}", q.ecosystem, q.name, q.version))
        .collect();
    assert!(silent.is_empty(), "OSV reports nothing for {silent:?}");
}
