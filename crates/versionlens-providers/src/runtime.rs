use versionlens_model::{
    Dependency, VersionDialect, normalized_version_for_dialect,
    requirement_is_parseable_for_dialect,
};

mod channels;
mod integrity;
mod source;
mod versions;

use source::{
    dotnet as dotnet_source, dotnet_floating_requirement, java as java_source,
    python as python_source, ruby as ruby_source,
};

#[cfg(test)]
mod tests {
use versionlens_model::{Dependency, Ecosystem, Position, Range};

use super::{RuntimeSource, RuntimeSourceError};

fn yarn(requirement: &str) -> Dependency {
    runtime("yarn", requirement, Ecosystem::Npm, "packageManager", None)
}

fn runtime(
    name: &str,
    requirement: &str,
    ecosystem: Ecosystem,
    group: &str,
    hosted_name: Option<&str>,
) -> Dependency {
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: hosted_name.map(str::to_owned),
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

#[test]
fn inputs_without_a_release_source_are_not_supported() {
    let not_supported = [
        runtime("vscode", "^1.75.0", Ecosystem::Npm, "engines", None),
        runtime(
            "dotnet",
            "8.0.100.1",
            Ecosystem::Dotnet,
            "with.dotnet-version",
            None,
        ),
        runtime(
            "dotnet",
            "8.0",
            Ecosystem::Dotnet,
            "with.dotnet-version",
            Some("quality=preview"),
        ),
        runtime(
            "java",
            "17",
            Ecosystem::Maven,
            "with.java-version",
            Some("zulu"),
        ),
        runtime(
            "python",
            "pypy3.10",
            Ecosystem::Python,
            "with.python-version",
            None,
        ),
        runtime(
            "ruby",
            "jruby-9.4",
            Ecosystem::Ruby,
            "with.ruby-version",
            None,
        ),
        runtime(
            "deno",
            &"a".repeat(40),
            Ecosystem::Deno,
            "with.deno-version",
            None,
        ),
    ];
    for dependency in &not_supported {
        assert!(
            matches!(
                RuntimeSource::for_dependency(dependency),
                Err(RuntimeSourceError::NotSupported(_))
            ),
            "{} {}",
            dependency.name,
            dependency.requirement
        );
    }
    let invalid = [
        runtime("java", "17", Ecosystem::Maven, "with.java-version", None),
        runtime(
            "go",
            "not a version",
            Ecosystem::Go,
            "with.go-version",
            None,
        ),
    ];
    for dependency in &invalid {
        assert!(
            matches!(
                RuntimeSource::for_dependency(dependency),
                Err(RuntimeSourceError::Invalid(_))
            ),
            "{} {}",
            dependency.name,
            dependency.requirement
        );
    }
}

#[test]
fn go_and_deno_use_their_official_release_catalogs() {
    let go = RuntimeSource::for_dependency(&runtime(
        "go",
        "stable",
        Ecosystem::Go,
        "with.go-version",
        None,
    ))
    .unwrap();
    assert_eq!(go.url(), "https://go.dev/dl/?mode=json&include=all");
    let body = r#"[{"version":"go1.25.1","stable":true},{"version":"go1.24.7","stable":true},{"version":"go1.26rc1","stable":false}]"#;
    assert_eq!(
        go.preferred_release(body, "stable").unwrap().as_deref(),
        Some("1.25.1")
    );
    assert_eq!(
        go.preferred_release(body, "oldstable").unwrap().as_deref(),
        Some("1.24.7")
    );

    let deno = RuntimeSource::for_dependency(&runtime(
        "deno",
        "^2",
        Ecosystem::Deno,
        "with.deno-version",
        None,
    ))
    .unwrap();
    assert_eq!(deno.url(), "https://dl.deno.land/versions.json");
    assert_eq!(
        deno.versions(r#"{"cli":["v2.1.0","v2.0.0"]}"#, "^2")
            .unwrap(),
        ["2.1.0", "2.0.0"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "deno",
            &"a".repeat(40),
            Ecosystem::Deno,
            "with.deno-version",
            None,
        ))
        .is_err()
    );
}

#[test]
fn python_and_ruby_catalogs_accept_only_supported_interpreters() {
    let python = RuntimeSource::for_dependency(&runtime(
        "python",
        "3.12",
        Ecosystem::Python,
        "with.python-version",
        None,
    ))
    .unwrap();
    assert!(
        python
            .url()
            .ends_with("actions/python-versions/main/versions-manifest.json")
    );
    assert_eq!(
        python
            .versions(r#"[{"version":"3.12.2","stable":true}]"#, "3.12")
            .unwrap(),
        ["3.12.2"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "python",
            "pypy3.10",
            Ecosystem::Python,
            "with.python-version",
            None,
        ))
        .is_err()
    );

    let ruby = RuntimeSource::for_dependency(&runtime(
        "ruby",
        "3.3",
        Ecosystem::Ruby,
        "with.ruby-version",
        None,
    ))
    .unwrap();
    assert!(
        ruby.url()
            .ends_with("ruby/setup-ruby/master/ruby-builder-versions.json")
    );
    assert_eq!(
        ruby.versions(r#"{"ruby":["3.3.0","head"],"jruby":["9.4.0.0"]}"#, "3.3")
            .unwrap(),
        ["3.3.0"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "ruby",
            "jruby-9.4",
            Ecosystem::Ruby,
            "with.ruby-version",
            None,
        ))
        .is_err()
    );
}

#[test]
fn temurin_and_dotnet_parse_sdk_release_metadata() {
    let java = RuntimeSource::for_dependency(&runtime(
        "java",
        "17",
        Ecosystem::Maven,
        "with.java-version",
        Some("temurin"),
    ))
    .unwrap();
    assert!(
        java.url()
            .contains("api.adoptium.net/v3/info/release_names")
    );
    assert_eq!(
        java.versions(
            r#"{"releases":["jdk-21+35","jdk-21.0.12+8","jdk-21.0.12.1+1","jdk8u504-b01"]}"#,
            "21"
        )
        .unwrap(),
        ["21.0.0", "21.0.12", "21.0.12.1", "8.0.504"]
    );
    for release in ["jdk-not-a-version", "jdk-21rc1+1", "jdk8u504-bx"] {
        let body = format!(r#"{{"releases":["{release}"]}}"#);
        let error = java.versions(&body, "21").unwrap_err();
        assert_eq!(
            error, "Temurin release response contains an invalid version",
            "{release}"
        );
    }
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "java",
            "17",
            Ecosystem::Maven,
            "with.java-version",
            Some("zulu"),
        ))
        .is_err()
    );

    let dotnet = RuntimeSource::for_dependency(&runtime(
        "dotnet",
        "8.0.4xx",
        Ecosystem::Dotnet,
        "with.dotnet-version",
        None,
    ))
    .unwrap();
    assert!(dotnet.url().ends_with("/8.0/releases.json"));
    let body = r#"{"releases":[{"sdk":{"version":"8.0.405"},"sdks":[{"version":"8.0.309"},{"version":"8.0.406"}]}]}"#;
    assert_eq!(
        dotnet.versions(body, "8.0.4xx").unwrap(),
        ["8.0.405", "8.0.406"]
    );
}

#[test]
fn yarn_uses_the_release_source_for_its_version_family() {
    let classic = RuntimeSource::for_dependency(&yarn("1.22.22")).unwrap();
    assert!(matches!(classic, RuntimeSource::PackageManager("yarn")));
    assert_eq!(classic.url(), "https://registry.npmjs.org/yarn");

    for requirement in ["4.14.1", "stable", "^4.0.0", ">=2.0.0, <5.0.0"] {
        let modern = RuntimeSource::for_dependency(&yarn(requirement)).unwrap();
        assert!(matches!(modern, RuntimeSource::YarnModern));
        assert_eq!(modern.url(), "https://repo.yarnpkg.com/tags");
    }

    assert!(matches!(
        RuntimeSource::for_dependency(&yarn("^1.22.0")).unwrap(),
        RuntimeSource::PackageManager("yarn")
    ));
    assert!(RuntimeSource::for_dependency(&yarn(">=1.0.0")).is_err());
}

#[test]
fn modern_yarn_uses_published_tags_and_aliases() {
    let source = RuntimeSource::YarnModern;
    let body =
        r#"{"aliases":{"latest":"4.2.0","stable":"4.1.1"},"tags":["4.2.0","4.1.1","3.8.7"]}"#;
    assert_eq!(
        source.versions(body, "4.0.0").unwrap(),
        ["4.2.0", "4.1.1", "3.8.7"]
    );
    assert_eq!(
        source.preferred_release(body, "4.0.0").unwrap(),
        Some("4.2.0".to_owned())
    );
    assert_eq!(
        source.preferred_release(body, "stable").unwrap(),
        Some("4.1.1".to_owned())
    );
}

#[test]
fn modern_yarn_requires_aliases_to_reference_published_tags() {
    let source = RuntimeSource::YarnModern;
    for body in [
        r#"{"tags":["4.2.0"]}"#,
        r#"{"aliases":{"latest":"invalid"},"tags":["invalid"]}"#,
        r#"{"aliases":{"latest":"4.3.0"},"tags":["4.2.0"]}"#,
    ] {
        assert!(source.preferred_release(body, "4.0.0").is_err());
    }
}
}

/// Why a runtime input has no release source to check against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeSourceError {
    /// No upstream catalog covers this runtime or input form, so it is skipped.
    NotSupported(String),
    /// The input is malformed or needs a value that is not written literally.
    Invalid(String),
}

impl std::fmt::Display for RuntimeSourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotSupported(message) | Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for RuntimeSourceError {}

pub enum RuntimeSource {
    Node,
    Go,
    DenoVersions,
    DenoChannel(&'static str),
    Python,
    JavaTemurin(u64),
    DotnetIndex(Option<String>),
    DotnetChannel(String),
    Ruby,
    GitHubTags {
        repository: &'static str,
        prefix: &'static str,
    },
    PackageManager(&'static str),
    YarnModern,
    RustChannel {
        channel: String,
        date: Option<String>,
    },
    BunCanary,
}

impl RuntimeSource {
    pub fn for_dependency(dependency: &Dependency) -> Result<Self, RuntimeSourceError> {
        let requirement = dependency.requirement.trim();
        match dependency.name.as_str() {
            "node" => Ok(Self::Node),
            "go" => go_source(requirement),
            "deno" => deno_source(requirement),
            "python" => python_source(dependency, requirement),
            "java" => java_source(dependency, requirement),
            "dotnet" => dotnet_source(dependency, requirement),
            "ruby" => ruby_source(requirement),
            "bun" if requirement == "canary" => Ok(Self::BunCanary),
            "bun" => Ok(Self::GitHubTags {
                repository: "oven-sh/bun",
                prefix: "bun-v",
            }),
            "rust" => {
                if let Some((channel, date)) = rust_channel(requirement) {
                    Ok(Self::RustChannel {
                        channel: channel.to_owned(),
                        date: date.map(str::to_owned),
                    })
                } else {
                    Ok(Self::GitHubTags {
                        repository: "rust-lang/rust",
                        prefix: "",
                    })
                }
            }
            "npm" => Ok(Self::PackageManager("npm")),
            "pnpm" => Ok(Self::PackageManager("pnpm")),
            "yarn" => yarn_source(requirement),
            name => Err(RuntimeSourceError::NotSupported(format!(
                "upstream releases are not configured for runtime {name}"
            ))),
        }
    }

    pub fn url(&self) -> String {
        match self {
            Self::Node => "https://nodejs.org/dist/index.json".to_owned(),
            Self::Go => "https://go.dev/dl/?mode=json&include=all".to_owned(),
            Self::DenoVersions => "https://dl.deno.land/versions.json".to_owned(),
            Self::DenoChannel(channel) => {
                format!("https://dl.deno.land/{channel}-latest.txt")
            }
            Self::Python => {
                "https://raw.githubusercontent.com/actions/python-versions/main/versions-manifest.json"
                    .to_owned()
            }
            Self::JavaTemurin(feature) => format!(
                "https://api.adoptium.net/v3/info/release_names?project=jdk&release_type=ga&vendor=eclipse&version=%5B{feature},{}%29&page_size=100&page=0",
                feature + 1
            ),
            Self::DotnetIndex(_) => {
                "https://builds.dotnet.microsoft.com/dotnet/release-metadata/releases-index.json"
                    .to_owned()
            }
            Self::DotnetChannel(channel) => format!(
                "https://builds.dotnet.microsoft.com/dotnet/release-metadata/{channel}/releases.json"
            ),
            Self::Ruby => {
                "https://raw.githubusercontent.com/ruby/setup-ruby/master/ruby-builder-versions.json"
                    .to_owned()
            }
            Self::GitHubTags { repository, .. } => {
                format!("https://api.github.com/repos/{repository}/tags")
            }
            Self::PackageManager(name) => format!("https://registry.npmjs.org/{name}"),
            Self::YarnModern => "https://repo.yarnpkg.com/tags".to_owned(),
            Self::RustChannel { channel, date } => {
                let directory = date
                    .as_ref()
                    .map_or(String::new(), |date| format!("{date}/"));
                format!("https://static.rust-lang.org/dist/{directory}channel-rust-{channel}.toml")
            }
            Self::BunCanary => {
                "https://api.github.com/repos/oven-sh/bun/releases/tags/canary".to_owned()
            }
        }
    }

    pub fn is_channel(&self, requirement: &str) -> bool {
        let requirement = requirement
            .split_once("+sha")
            .map_or(requirement, |(version, _)| version);
        match self {
            Self::RustChannel { .. }
            | Self::BunCanary
            | Self::DenoChannel(_)
            | Self::DotnetIndex(_) => true,
            Self::Go => matches!(requirement, "stable" | "oldstable"),
            Self::DotnetChannel(_) => dotnet_floating_requirement(requirement),
            Self::Node => {
                matches!(requirement, "latest" | "node" | "current")
                    || requirement.starts_with("lts/")
            }
            Self::PackageManager(_) => {
                versionlens_model::is_npm_dist_tag_requirement(requirement)
                    && semver::VersionReq::parse(requirement.trim_start_matches('v')).is_err()
            }
            Self::YarnModern => {
                semver::VersionReq::parse(requirement.trim_start_matches('v')).is_err()
            }
            Self::GitHubTags { .. } => requirement == "latest",
            Self::Ruby => requirement == "ruby",
            Self::DenoVersions | Self::Python | Self::JavaTemurin(_) => false,
        }
    }

    pub fn is_constraint(&self, requirement: &str) -> bool {
        matches!(
            self,
            Self::Go | Self::DenoVersions | Self::Python | Self::JavaTemurin(_) | Self::Ruby
        ) && !self.is_exact_version(requirement)
            && !self.is_channel(requirement)
    }

    fn is_exact_version(&self, requirement: &str) -> bool {
        if matches!(self, Self::JavaTemurin(_)) {
            let requirement = requirement.trim_start_matches(['v', 'V']);
            requirement.split('.').count() >= 3
                && normalized_version_for_dialect(requirement, VersionDialect::Pep440).is_some()
        } else {
            is_exact_semver(requirement)
        }
    }

    pub fn versions(&self, body: &str, requirement: &str) -> Result<Vec<String>, String> {
        versions::parse(self, body, requirement)
    }
}

fn go_source(requirement: &str) -> Result<RuntimeSource, RuntimeSourceError> {
    if matches!(requirement, "stable" | "oldstable") {
        Ok(RuntimeSource::Go)
    } else {
        semantic_runtime_source(requirement, "Go", RuntimeSource::Go)
    }
}

fn deno_source(requirement: &str) -> Result<RuntimeSource, RuntimeSourceError> {
    match requirement {
        "latest" => Ok(RuntimeSource::DenoChannel("release")),
        "lts" => Ok(RuntimeSource::DenoChannel("release-lts")),
        "rc" => Ok(RuntimeSource::DenoChannel("release-rc")),
        "canary" => Ok(RuntimeSource::DenoChannel("canary")),
        value if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) => {
            Err(RuntimeSourceError::NotSupported(
                "Deno commit hashes cannot be checked against the published release catalog"
                    .to_owned(),
            ))
        }
        _ => semantic_runtime_source(requirement, "Deno", RuntimeSource::DenoVersions),
    }
}

fn semantic_runtime_source(
    requirement: &str,
    label: &str,
    source: RuntimeSource,
) -> Result<RuntimeSource, RuntimeSourceError> {
    if supported_semver_requirement(requirement) {
        Ok(source)
    } else {
        Err(RuntimeSourceError::Invalid(format!(
            "{label} setup input requires a supported version, range, or release channel"
        )))
    }
}

fn is_exact_semver(requirement: &str) -> bool {
    semver::Version::parse(requirement.trim_start_matches('v')).is_ok()
}

fn supported_semver_requirement(requirement: &str) -> bool {
    let requirement = requirement.trim_start_matches('v');
    !requirement.is_empty()
        && requirement.split("||").all(|part| {
            let normalized = part.split_whitespace().collect::<Vec<_>>().join(", ");
            semver::Version::parse(part).is_ok()
                || semver::VersionReq::parse(part).is_ok()
                || semver::VersionReq::parse(&normalized).is_ok()
        })
}

fn yarn_source(requirement: &str) -> Result<RuntimeSource, RuntimeSourceError> {
    let version = requirement
        .split_once("+sha")
        .map_or(requirement, |(version, _)| version)
        .trim_start_matches('v');
    if let Ok(version) = semver::Version::parse(version) {
        return Ok(if version.major < 2 {
            RuntimeSource::PackageManager("yarn")
        } else {
            RuntimeSource::YarnModern
        });
    }
    if matches!(version, "latest" | "stable" | "canary") {
        return Ok(RuntimeSource::YarnModern);
    }
    let requirement = semver::VersionReq::parse(version).map_err(|_| {
        RuntimeSourceError::Invalid(
            "Yarn release source requires a valid version, range, or channel".to_owned(),
        )
    })?;
    let mut family = None;
    for comparator in &requirement.comparators {
        let candidate = match comparator.op {
            semver::Op::Exact | semver::Op::Tilde | semver::Op::Caret | semver::Op::Wildcard => {
                Some(comparator.major < 2)
            }
            semver::Op::Greater | semver::Op::GreaterEq if comparator.major >= 2 => Some(false),
            semver::Op::Less
                if comparator.major == 2
                    && comparator.minor.unwrap_or(0) == 0
                    && comparator.patch.unwrap_or(0) == 0
                    && comparator.pre.is_empty() =>
            {
                Some(true)
            }
            semver::Op::Less | semver::Op::LessEq if comparator.major < 2 => Some(true),
            _ => None,
        };
        if let Some(candidate) = candidate {
            if family.is_some_and(|family| family != candidate) {
                return Err(yarn_spans_release_sources());
            }
            family = Some(candidate);
        }
    }
    match family {
        Some(true) => Ok(RuntimeSource::PackageManager("yarn")),
        Some(false) => Ok(RuntimeSource::YarnModern),
        None => Err(yarn_spans_release_sources()),
    }
}

fn yarn_spans_release_sources() -> RuntimeSourceError {
    RuntimeSourceError::Invalid(
        "Yarn version range spans release sources and cannot be checked safely".to_owned(),
    )
}

fn rust_channel(requirement: &str) -> Option<(&str, Option<&str>)> {
    for channel in ["stable", "beta", "nightly"] {
        if requirement == channel {
            return Some((channel, None));
        }
        if let Some(date) = requirement
            .strip_prefix(channel)
            .and_then(|value| value.strip_prefix('-'))
            && date.len() == 10
            && date.bytes().enumerate().all(|(index, byte)| {
                if matches!(index, 4 | 7) {
                    byte == b'-'
                } else {
                    byte.is_ascii_digit()
                }
            })
        {
            return Some((channel, Some(date)));
        }
    }
    None
}
