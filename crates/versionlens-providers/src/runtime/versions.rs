use serde_json::Value;
use std::iter::once;
use versionlens_versions::{VersionDialect, normalized_version_for_dialect};

use super::{
    RuntimeSource,
    source::{dotnet_feature_band, dotnet_version_in_feature_band},
};

pub(super) fn parse(
    source: &RuntimeSource,
    body: &str,
    requirement: &str,
) -> Result<Vec<String>, String> {
    match source {
        RuntimeSource::Go => Ok(go_releases(body)?
            .into_iter()
            .map(|(version, _)| version)
            .collect()),
        RuntimeSource::DenoVersions => deno_versions(body),
        RuntimeSource::DenoChannel(channel) => Ok(vec![deno_channel_version(channel, body)?]),
        RuntimeSource::Python => manifest_versions(body, "Python"),
        RuntimeSource::JavaTemurin(_) => temurin_versions(body),
        RuntimeSource::DotnetIndex(release_type) => {
            dotnet_index_versions(body, requirement, release_type.as_deref())
        }
        RuntimeSource::DotnetChannel(_) => dotnet_channel_versions(body, requirement),
        RuntimeSource::Ruby => ruby_versions(body),
        RuntimeSource::RustChannel { channel, date } => {
            rust_channel_versions(body, channel, date.as_deref())
        }
        RuntimeSource::BunCanary => bun_canary_versions(body),
        RuntimeSource::PackageManager(_) => package_manager_versions(body),
        RuntimeSource::YarnModern => yarn_versions(body),
        RuntimeSource::Node | RuntimeSource::GitHubTags { .. } => {
            array_versions(source, body, requirement)
        }
    }
}

fn array_versions(
    source: &RuntimeSource,
    body: &str,
    requirement: &str,
) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid runtime release response: {error}"))?;
    let entries = value
        .as_array()
        .ok_or("runtime release response is not an array")?;
    Ok(entries
        .iter()
        .filter_map(|entry| match source {
            RuntimeSource::Node => {
                if let Some(lts) = requirement.strip_prefix("lts/") {
                    let name = entry.get("lts").and_then(Value::as_str)?;
                    if lts != "*" && !lts.eq_ignore_ascii_case(name) {
                        return None;
                    }
                }
                entry
                    .get("version")
                    .and_then(Value::as_str)
                    .map(|version| version.trim_start_matches('v').to_owned())
            }
            RuntimeSource::GitHubTags { prefix, .. } => entry
                .get("name")
                .and_then(Value::as_str)
                .and_then(|tag| tag.strip_prefix(prefix))
                .map(str::to_owned),
            _ => None,
        })
        .collect())
}

fn rust_channel_versions(
    body: &str,
    channel: &str,
    date: Option<&str>,
) -> Result<Vec<String>, String> {
    let document = body
        .parse::<toml_edit::DocumentMut>()
        .map_err(|error| format!("invalid Rust channel manifest: {error}"))?;
    let manifest_date = document
        .get("date")
        .and_then(toml_edit::Item::as_str)
        .ok_or("Rust channel manifest has no date")?;
    if date.is_some_and(|date| date != manifest_date) {
        return Err("Rust channel manifest date does not match the requested date".to_owned());
    }
    let version = document
        .get("pkg")
        .and_then(|pkg| pkg.get("rust"))
        .and_then(|rust| rust.get("version"))
        .and_then(toml_edit::Item::as_str)
        .and_then(|version| version.split_whitespace().next())
        .ok_or("Rust channel manifest has no compiler version")?;
    let parsed = semver::Version::parse(version).map_err(|_| "invalid Rust compiler version")?;
    if channel == "stable" && !parsed.pre.is_empty()
        || channel != "stable" && !parsed.pre.as_str().starts_with(channel)
    {
        return Err("Rust channel manifest does not match the requested channel".to_owned());
    }
    Ok(vec![version.to_owned()])
}

fn bun_canary_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid runtime release response: {error}"))?;
    if value.get("tag_name").and_then(Value::as_str) != Some("canary") {
        return Err("Bun canary release was not returned".to_owned());
    }
    let commit = value
        .get("name")
        .and_then(Value::as_str)
        .and_then(|name| name.strip_prefix("Canary ("))
        .and_then(|name| name.strip_suffix(')'))
        .filter(|sha| sha.len() == 40 && sha.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or("Bun canary release has no immutable revision")?;
    Ok(vec![format!("canary@{commit}")])
}

fn package_manager_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid runtime release response: {error}"))?;
    value
        .get("versions")
        .and_then(Value::as_object)
        .map(|versions| versions.keys().cloned().collect())
        .ok_or_else(|| "package-manager response has no published versions".to_owned())
}

fn yarn_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid runtime release response: {error}"))?;
    value
        .get("tags")
        .and_then(Value::as_array)
        .ok_or_else(|| "Yarn release response has no published versions".to_owned())?
        .iter()
        .map(|version| {
            version
                .as_str()
                .filter(|version| semver::Version::parse(version).is_ok())
                .map(str::to_owned)
                .ok_or_else(|| "Yarn release response contains an invalid version".to_owned())
        })
        .collect()
}

fn manifest_versions(body: &str, label: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid {label} release response: {error}"))?;
    value
        .as_array()
        .ok_or_else(|| format!("{label} release response is not an array"))?
        .iter()
        .map(|entry| {
            entry
                .get("version")
                .and_then(Value::as_str)
                .filter(|version| semver::Version::parse(version).is_ok())
                .map(str::to_owned)
                .ok_or_else(|| format!("{label} release response contains an invalid version"))
        })
        .collect()
}

fn temurin_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid Temurin release response: {error}"))?;
    value
        .get("releases")
        .and_then(Value::as_array)
        .ok_or_else(|| "Temurin release response has no releases".to_owned())?
        .iter()
        .map(|release| {
            release
                .as_str()
                .and_then(normalize_temurin_version)
                .ok_or_else(|| "Temurin release response contains an invalid version".to_owned())
        })
        .collect()
}

fn ruby_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid Ruby release response: {error}"))?;
    let versions = value
        .get("ruby")
        .and_then(Value::as_array)
        .ok_or_else(|| "Ruby release response has no MRI versions".to_owned())?
        .iter()
        .filter_map(Value::as_str)
        .filter(|version| semver::Version::parse(version).is_ok())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if versions.is_empty() {
        Err("Ruby release response has no numeric MRI versions".to_owned())
    } else {
        Ok(versions)
    }
}

fn normalize_temurin_version(release: &str) -> Option<String> {
    if let Some(version) = release.strip_prefix("jdk-") {
        let (version, build) = version.split_once('+')?;
        if build.is_empty()
            || !version
                .split('.')
                .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
        {
            return None;
        }
        return normalized_version_for_dialect(version, VersionDialect::Pep440);
    }
    let update = release.strip_prefix("jdk")?;
    let (feature, rest) = update.split_once('u')?;
    let (patch, build) = rest.split_once("-b")?;
    if ![feature, patch, build]
        .iter()
        .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    let version = format!("{feature}.0.{patch}");
    normalized_version_for_dialect(&version, VersionDialect::Pep440)
}

fn dotnet_index_versions(
    body: &str,
    requirement: &str,
    release_type: Option<&str>,
) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid .NET release index: {error}"))?;
    let major = requirement.trim_end_matches(".x").parse::<u64>().ok();
    let versions = value
        .get("releases-index")
        .and_then(Value::as_array)
        .ok_or_else(|| ".NET release index has no releases".to_owned())?
        .iter()
        .filter(|entry| entry.get("support-phase").and_then(Value::as_str) != Some("eol"))
        .filter(|entry| {
            release_type.is_none_or(|expected| {
                entry
                    .get("release-type")
                    .and_then(Value::as_str)
                    .is_some_and(|actual| actual.eq_ignore_ascii_case(expected))
            })
        })
        .filter_map(|entry| {
            let channel = entry.get("channel-version").and_then(Value::as_str)?;
            if major.is_some_and(|major| !channel.starts_with(&format!("{major}."))) {
                return None;
            }
            let version = entry.get("latest-sdk").and_then(Value::as_str)?;
            (requirement != "latest" || semver::Version::parse(version).ok()?.pre.is_empty())
                .then(|| version.to_owned())
        })
        .collect::<Vec<_>>();
    if versions.is_empty() {
        return Err(".NET release index has no matching active SDK versions".to_owned());
    }
    Ok(versions)
}

fn dotnet_channel_versions(body: &str, requirement: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid .NET channel response: {error}"))?;
    let feature_band = dotnet_feature_band(requirement);
    let mut versions = Vec::new();
    for release in value
        .get("releases")
        .and_then(Value::as_array)
        .ok_or_else(|| ".NET channel response has no releases".to_owned())?
    {
        for sdk in once(release.get("sdk")).chain(
            release
                .get("sdks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .map(Some),
        ) {
            let Some(version) = sdk
                .and_then(|sdk| sdk.get("version"))
                .and_then(Value::as_str)
            else {
                continue;
            };
            if feature_band.is_none_or(|band| dotnet_version_in_feature_band(version, band))
                && !versions.iter().any(|existing| existing == version)
            {
                versions.push(version.to_owned());
            }
        }
    }
    if versions.is_empty() {
        return Err(".NET channel response has no matching SDK versions".to_owned());
    }
    Ok(versions)
}

pub(super) fn go_releases(body: &str) -> Result<Vec<(String, bool)>, String> {
    let entries: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid Go release response: {error}"))?;
    entries
        .as_array()
        .ok_or_else(|| "Go release response is not an array".to_owned())?
        .iter()
        .map(|entry| {
            let version = entry
                .get("version")
                .and_then(Value::as_str)
                .and_then(normalize_go_version)
                .ok_or_else(|| "Go release response contains an invalid version".to_owned())?;
            let stable = entry
                .get("stable")
                .and_then(Value::as_bool)
                .ok_or_else(|| "Go release response contains no stability marker".to_owned())?;
            Ok((version, stable))
        })
        .collect()
}

fn deno_versions(body: &str) -> Result<Vec<String>, String> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| format!("invalid Deno release response: {error}"))?;
    value
        .get("cli")
        .and_then(Value::as_array)
        .ok_or_else(|| "Deno release response has no CLI versions".to_owned())?
        .iter()
        .map(|entry| {
            entry
                .as_str()
                .and_then(|version| {
                    versionlens_versions::normalized_version(version.trim_start_matches('v'))
                })
                .ok_or_else(|| "Deno release response contains an invalid version".to_owned())
        })
        .collect()
}

fn deno_channel_version(channel: &str, body: &str) -> Result<String, String> {
    let value = body.trim();
    if channel == "canary" {
        return (value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| format!("0.0.0-{value}"))
            .ok_or_else(|| "Deno canary response has no immutable revision".to_owned());
    }
    let version = versionlens_versions::normalized_version(value.trim_start_matches('v'))
        .ok_or_else(|| format!("Deno {channel} response contains an invalid version"))?;
    let parsed = semver::Version::parse(&version)
        .map_err(|_| format!("Deno {channel} response contains an invalid version"))?;
    if channel == "release-rc" {
        if !parsed.pre.as_str().starts_with("rc.") {
            return Err("Deno release candidate response is not an RC version".to_owned());
        }
    } else if !parsed.pre.is_empty() {
        return Err(format!("Deno {channel} response is not a stable version"));
    }
    Ok(version)
}

fn normalize_go_version(value: &str) -> Option<String> {
    let value = value.strip_prefix("go")?;
    for label in ["beta", "rc"] {
        if let Some((base, number)) = value.split_once(label) {
            if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
                return None;
            }
            let base = versionlens_versions::normalized_version(base)?;
            return Some(format!("{base}-{label}.{number}"));
        }
    }
    versionlens_versions::normalized_version(value)
}
