use versionlens_model::Dependency;
use versionlens_versions::{
    VersionDialect, normalized_version_for_dialect, requirement_is_parseable_for_dialect,
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
mod tests;

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
    pub fn for_dependency(dependency: &Dependency) -> Result<Self, String> {
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
            name => Err(format!(
                "upstream releases are not configured for runtime {name}"
            )),
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

fn go_source(requirement: &str) -> Result<RuntimeSource, String> {
    if matches!(requirement, "stable" | "oldstable") {
        Ok(RuntimeSource::Go)
    } else {
        semantic_runtime_source(requirement, "Go", RuntimeSource::Go)
    }
}

fn deno_source(requirement: &str) -> Result<RuntimeSource, String> {
    match requirement {
        "latest" => Ok(RuntimeSource::DenoChannel("release")),
        "lts" => Ok(RuntimeSource::DenoChannel("release-lts")),
        "rc" => Ok(RuntimeSource::DenoChannel("release-rc")),
        "canary" => Ok(RuntimeSource::DenoChannel("canary")),
        value if value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) => Err(
            "Deno commit hashes cannot be checked against the published release catalog".to_owned(),
        ),
        _ => semantic_runtime_source(requirement, "Deno", RuntimeSource::DenoVersions),
    }
}

fn semantic_runtime_source(
    requirement: &str,
    label: &str,
    source: RuntimeSource,
) -> Result<RuntimeSource, String> {
    if supported_semver_requirement(requirement) {
        Ok(source)
    } else {
        Err(format!(
            "{label} setup input requires a supported version, range, or release channel"
        ))
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

fn yarn_source(requirement: &str) -> Result<RuntimeSource, String> {
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
        "Yarn release source requires a valid version, range, or channel".to_owned()
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
                return Err(
                    "Yarn version range spans release sources and cannot be checked safely"
                        .to_owned(),
                );
            }
            family = Some(candidate);
        }
    }
    match family {
        Some(true) => Ok(RuntimeSource::PackageManager("yarn")),
        Some(false) => Ok(RuntimeSource::YarnModern),
        None => {
            Err("Yarn version range spans release sources and cannot be checked safely".to_owned())
        }
    }
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
