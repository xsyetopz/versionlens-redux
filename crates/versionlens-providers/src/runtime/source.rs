use versionlens_model::Dependency;

use super::{
    RuntimeSource, VersionDialect, requirement_is_parseable_for_dialect, semantic_runtime_source,
    supported_semver_requirement,
};

pub(super) fn python(dependency: &Dependency, requirement: &str) -> Result<RuntimeSource, String> {
    if !dependency.group.starts_with("with.") {
        return Ok(RuntimeSource::Python);
    }
    if requirement.ends_with(['t', 'T'])
        || requirement.to_ascii_lowercase().contains("pypy")
        || requirement.to_ascii_lowercase().contains("graalpy")
        || requirement.ends_with("-dev")
    {
        return Err(
            "setup-python interpreter variants are not supported by the CPython release catalog"
                .to_owned(),
        );
    }
    semantic_runtime_source(requirement, "Python", RuntimeSource::Python)
}

pub(super) fn java(dependency: &Dependency, requirement: &str) -> Result<RuntimeSource, String> {
    match dependency.hosted_name.as_deref() {
        Some(distribution) if distribution.eq_ignore_ascii_case("temurin") => {}
        Some("runtime-context-expression") => {
            return Err("setup-java distribution requires a literal value".to_owned());
        }
        Some(distribution) => {
            return Err(format!(
                "setup-java distribution {distribution} is not supported; only temurin can be checked"
            ));
        }
        None => return Err("setup-java requires the distribution input".to_owned()),
    }
    if !requirement_is_parseable_for_dialect(requirement, requirement, VersionDialect::Pep440) {
        return Err("Java setup input requires a supported numeric version or range".to_owned());
    }
    let feature = requirement
        .trim_start_matches('v')
        .trim_start_matches(['=', '<', '>', '^', '~'])
        .split(|character: char| !character.is_ascii_digit())
        .next()
        .and_then(|part| part.parse::<u64>().ok())
        .filter(|feature| *feature > 0)
        .ok_or_else(|| "Java setup input has no concrete feature version".to_owned())?;
    Ok(RuntimeSource::JavaTemurin(feature))
}

pub(super) fn dotnet(dependency: &Dependency, requirement: &str) -> Result<RuntimeSource, String> {
    let (channel, quality) = dotnet_context(dependency.hosted_name.as_deref());
    if quality.is_some_and(|quality| !quality.eq_ignore_ascii_case("ga")) {
        return Err("setup-dotnet daily and preview quality catalogs are not supported".to_owned());
    }
    if requirement.eq_ignore_ascii_case("latest") {
        return match channel {
            None => Ok(RuntimeSource::DotnetIndex(None)),
            Some(channel) if channel.eq_ignore_ascii_case("lts") => {
                Ok(RuntimeSource::DotnetIndex(Some("lts".to_owned())))
            }
            Some(channel) if channel.eq_ignore_ascii_case("sts") => {
                Ok(RuntimeSource::DotnetIndex(Some("sts".to_owned())))
            }
            Some(channel) if dotnet_channel_name(channel).is_some() => {
                Ok(RuntimeSource::DotnetChannel(
                    dotnet_channel_name(channel).unwrap_or_else(|| channel.to_owned()),
                ))
            }
            Some("runtime-context-expression") => {
                Err("setup-dotnet channel requires a literal value".to_owned())
            }
            Some(channel) => Err(format!("setup-dotnet channel {channel} is not supported")),
        };
    }
    if !dotnet_supported_requirement(requirement) {
        return Err("setup-dotnet input uses an unsupported SDK version syntax".to_owned());
    }
    let numeric = trim_dotnet_wildcard(requirement);
    let parts = numeric.split('.').collect::<Vec<_>>();
    if parts.len() == 1 {
        return Ok(RuntimeSource::DotnetIndex(None));
    }
    Ok(RuntimeSource::DotnetChannel(format!(
        "{}.{}",
        parts[0], parts[1]
    )))
}

pub(super) fn ruby(requirement: &str) -> Result<RuntimeSource, String> {
    if requirement == "ruby" || supported_semver_requirement(requirement) {
        return Ok(RuntimeSource::Ruby);
    }
    Err("setup-ruby input supports only MRI numeric versions".to_owned())
}

fn dotnet_context(context: Option<&str>) -> (Option<&str>, Option<&str>) {
    let Some(context) = context else {
        return (None, None);
    };
    let channel = context
        .split(';')
        .find_map(|part| part.strip_prefix("channel="))
        .filter(|value| !value.is_empty());
    let quality = context
        .split(';')
        .find_map(|part| part.strip_prefix("quality="))
        .filter(|value| !value.is_empty());
    (channel, quality)
}

fn dotnet_channel_name(channel: &str) -> Option<String> {
    let parts = channel.split('.').collect::<Vec<_>>();
    (parts.len() >= 2
        && parts[0].bytes().all(|byte| byte.is_ascii_digit())
        && parts[1].bytes().all(|byte| byte.is_ascii_digit()))
    .then(|| format!("{}.{}", parts[0], parts[1]))
}

fn dotnet_supported_requirement(requirement: &str) -> bool {
    let value = trim_dotnet_wildcard(requirement);
    let parts = value.split('.').collect::<Vec<_>>();
    matches!(parts.len(), 1..=3)
        && parts.iter().all(|part| {
            !part.is_empty()
                && (part.bytes().all(|byte| byte.is_ascii_digit())
                    || parts.len() == 3
                        && part.ends_with("xx")
                        && !part[..part.len() - 2].is_empty()
                        && part[..part.len() - 2]
                            .bytes()
                            .all(|byte| byte.is_ascii_digit()))
        })
}

pub(super) fn dotnet_floating_requirement(requirement: &str) -> bool {
    if dotnet_feature_band(requirement).is_some() {
        return true;
    }
    let value = trim_dotnet_wildcard(requirement);
    value.split('.').count() < 3
}

pub(super) fn dotnet_feature_band(requirement: &str) -> Option<u64> {
    let patch = requirement.split('.').nth(2)?;
    let prefix = patch.strip_suffix("xx")?;
    (prefix.len() == 1 && prefix.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| prefix.parse::<u64>().ok().map(|value| value * 100))
        .flatten()
}

pub(super) fn dotnet_version_in_feature_band(version: &str, band: u64) -> bool {
    version
        .split(['.', '-'])
        .nth(2)
        .and_then(|patch| patch.parse::<u64>().ok())
        .is_some_and(|patch| patch / 100 == band / 100)
}

fn trim_dotnet_wildcard(requirement: &str) -> &str {
    requirement
        .strip_suffix(".x")
        .or_else(|| requirement.strip_suffix(".X"))
        .or_else(|| requirement.strip_suffix(".*"))
        .unwrap_or(requirement)
}
