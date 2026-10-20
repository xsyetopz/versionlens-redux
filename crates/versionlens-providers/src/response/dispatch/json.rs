use semver::Version;
use serde_json::Value;
use serde_json::from_str;
use std::collections::BTreeSet;
use versionlens_model::Ecosystem;
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, CocoaPods, Composer, Conan, Cpan, Cpp, Deno, Docker, Dotnet, Dub, GitHub,
    Hackage, Hex, Maven, Nim, Nix, Npm, Pub, Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};
use versionlens_model::latest_version_with_prerelease_tags;

use super::ResponseRequest;
use crate::response::composer::latest_composer_version;
use crate::response::conan::latest_conan_version;
use crate::response::cpp::latest_cpp_json_version;
use crate::response::deno::latest_deno_version;
use crate::response::docker::latest_docker_tag;
use crate::response::dub::latest_dub_version;
use crate::response::github::latest_github_tag;
use crate::response::hackage::latest_hackage_version;
use crate::response::hex::latest_hex_version;
use crate::response::nim::latest_nim_version;
use crate::response::npm::{latest_npm_response, latest_npm_version};
use crate::response::pub_registry::latest_pub_version;
use crate::response::ruby::latest_ruby_version;
use crate::response::swift::latest_swift_version;
use crate::response::vcpkg::latest_vcpkg_version;
use crate::response::versions::{latest_version_strings, object_version_values};

pub(super) fn latest_json_response(
    ecosystem: Ecosystem,
    body: &str,
    request: &ResponseRequest<'_>,
) -> Option<String> {
    if ecosystem == Npm {
        return latest_npm_response(
            body,
            request.requirement,
            request.include_prereleases,
            request.prerelease_tags,
        );
    }
    let value = from_str::<Value>(body).ok()?;
    if ecosystem == GitHub {
        return latest_github_tag(&value, request.include_prereleases, request.prerelease_tags);
    }
    JSON_RESPONSE_PARSERS
        .iter()
        .find_map(|(known, parse)| (*known == ecosystem).then(|| parse(&value, request)))
        .flatten()
}

type JsonResponseParser = for<'a> fn(&Value, &ResponseRequest<'a>) -> Option<String>;

const JSON_RESPONSE_PARSERS: &[(Ecosystem, JsonResponseParser)] = &[
    (AnsibleGalaxy, latest_ansible_json_response),
    (Bazel, latest_bazel_json_response),
    (Composer, |value, request| {
        latest_composer_version(
            value,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        )
    }),
    (CocoaPods, latest_cocoapods_json_response),
    (Conan, |value, request| {
        latest_conan_version(value, request.package)
    }),
    (Cpan, |value, _| {
        value
            .get("version")
            .and_then(Value::as_str)
            .map(str::to_owned)
    }),
    (Cpp, latest_cpp_json_response),
    (Deno, |value, request| {
        latest_deno_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Dotnet, |value, request| {
        latest_version_strings(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Docker, |value, request| {
        latest_docker_tag(
            value,
            request.requirement,
            request.include_prereleases,
            request.prerelease_tags,
        )
    }),
    (Dub, |value, request| {
        latest_dub_version(
            value,
            request.requirement,
            request.include_prereleases,
            request.prerelease_tags,
        )
    }),
    (Hackage, |value, request| {
        latest_hackage_version(
            value,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        )
    }),
    (Hex, |value, request| {
        latest_hex_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Maven, |value, request| {
        latest_version_strings(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Nim, |value, request| {
        latest_nim_version(
            value,
            request.package,
            request.include_prereleases,
            request.prerelease_tags,
        )
    }),
    (Nix, latest_nix_json_response),
    (Npm, latest_npm_json_response),
    (Unity, latest_npm_json_response),
    (Pub, |value, request| {
        latest_pub_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Ruby, |value, request| {
        latest_ruby_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Vcpkg, |value, request| {
        latest_vcpkg_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Swift, |value, request| {
        latest_swift_version(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Zig, |value, request| {
        latest_github_tag(value, request.include_prereleases, request.prerelease_tags)
    }),
    (Terraform, latest_terraform_json_response),
];

fn latest_npm_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    latest_npm_version(
        value,
        request.requirement,
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn latest_cpp_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    let tags = value.as_array()?;
    latest_cpp_json_version(
        &Value::Array(tags.to_owned()),
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn latest_ansible_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    latest_version_with_prerelease_tags(
        ansible_versions(value),
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn ansible_versions(value: &Value) -> impl Iterator<Item = &str> {
    value
        .get("data")
        .or_else(|| value.get("results"))
        .or_else(|| value.get("versions"))
        .and_then(|value| value.as_array())
        .into_iter()
        .flatten()
        .filter_map(ansible_version)
}

fn ansible_version(value: &Value) -> Option<&str> {
    value
        .get("version")
        .or_else(|| value.get("name"))
        .and_then(|value| value.as_str())
        .or_else(|| {
            value
                .get("summary_fields")?
                .get("versions")?
                .as_array()?
                .iter()
                .find_map(|version| version.get("name").and_then(|value| value.as_str()))
        })
}

fn latest_bazel_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    let yanked = yanked_versions(value);
    latest_version_with_prerelease_tags(
        value
            .get("versions")?
            .as_array()?
            .iter()
            .filter_map(|value| value.as_str())
            .filter(|version| !yanked.contains(*version)),
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn yanked_versions(value: &Value) -> BTreeSet<&str> {
    value
        .get("yanked_versions")
        .and_then(|value| value.as_object())
        .map(|versions| versions.keys().map(|value| value.as_str()).collect())
        .unwrap_or_default()
}

fn latest_cocoapods_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    let versions = value
        .get("versions")?
        .as_array()?
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("name").and_then(|value| value.as_str()))
                .or_else(|| entry.get("version").and_then(|value| value.as_str()))
        });

    latest_version_with_prerelease_tags(
        versions,
        request.include_prereleases,
        request.prerelease_tags,
    )
}

fn latest_nix_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    value
        .as_array()?
        .iter()
        .filter_map(|entry| {
            entry
                .as_str()
                .or_else(|| entry.get("name").and_then(|value| value.as_str()))
        })
        .filter_map(normalized_nix_tag)
        .filter_map(|tag| comparable_nix_version(tag, request).map(|version| (version, tag)))
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, tag)| tag.to_owned())
}

fn normalized_nix_tag(tag: &str) -> Option<&str> {
    tag.strip_prefix("nixos-")
        .or_else(|| tag.strip_prefix("release-"))
        .or(Some(tag))
}

fn comparable_nix_version(tag: &str, request: &ResponseRequest<'_>) -> Option<Version> {
    if !request.include_prereleases && tag.contains('-') {
        return None;
    }
    if !request.prerelease_tags.is_empty()
        && tag.contains('-')
        && !request
            .prerelease_tags
            .iter()
            .any(|allowed| tag.contains(allowed))
    {
        return None;
    }
    let core = tag.split_once('-').map_or(tag, |(core, _)| core);
    let parts = core
        .split('.')
        .map(|part| {
            let trimmed = part.trim_start_matches('0');
            if trimmed.is_empty() { "0" } else { trimmed }
        })
        .collect::<Vec<_>>();
    let normalized = match parts.as_slice() {
        [major] => format!("{major}.0.0"),
        [major, minor] => format!("{major}.{minor}.0"),
        _ => parts.join("."),
    };
    crate::parse_semver(&normalized).ok()
}

fn latest_terraform_json_response(value: &Value, request: &ResponseRequest<'_>) -> Option<String> {
    let versions = object_version_values(value)?;
    versionlens_model::latest_version_with_prerelease_tags(
        versions,
        request.include_prereleases,
        request.prerelease_tags,
    )
}
