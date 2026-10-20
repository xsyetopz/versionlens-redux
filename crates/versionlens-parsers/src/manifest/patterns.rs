//! URI and content tests for file types that a name or an extension cannot match.

use versionlens_model::DocumentInput;

use crate::json_manifest::looks_like_package_json;

pub(crate) fn file_name(uri: &str) -> Option<&str> {
    uri.rsplit('/').next()
}

fn has_extension(name: &str, extensions: &[&str]) -> bool {
    std::path::Path::new(name)
        .extension()
        .is_some_and(|extension| {
            extensions
                .iter()
                .any(|item| extension.eq_ignore_ascii_case(item))
        })
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|end| end.eq_ignore_ascii_case(suffix))
}

pub(super) fn is_unity_project_manifest_uri(uri: &str) -> bool {
    uri.ends_with("/Packages/manifest.json") || uri.ends_with("/packages/manifest.json")
}

/// `Manifest-v1.11.toml` and other versioned Julia manifests.
pub(super) fn is_julia_versioned_manifest_uri(uri: &str) -> bool {
    let Some(version) = file_name(uri)
        .and_then(|name| name.strip_prefix("Manifest-v"))
        .and_then(|value| value.strip_suffix(".toml"))
    else {
        return false;
    };
    version.split_once('.').is_some_and(|(major, minor)| {
        !major.is_empty()
            && !minor.is_empty()
            && major.bytes().all(|byte| byte.is_ascii_digit())
            && minor.bytes().all(|byte| byte.is_ascii_digit())
    })
}

/// .NET project files, except copies under `obj` and `bin`.
/// `.cargo/config.toml`, or the older `.cargo/config`.
pub(super) fn is_cargo_config_uri(uri: &str) -> bool {
    let mut segments = uri.rsplit('/');
    let name = segments.next().unwrap_or_default();
    (name == "config.toml" || name == "config") && segments.next() == Some(".cargo")
}

pub(super) fn is_dotnet_xml_uri(uri: &str) -> bool {
    let generated = uri
        .split('/')
        .any(|segment| segment.eq_ignore_ascii_case("obj") || segment.eq_ignore_ascii_case("bin"));
    !generated
        && file_name(uri).is_some_and(|name| {
            has_extension(name, &["csproj", "fsproj", "vbproj", "targets", "props"])
        })
}

/// `compose.yaml`, `docker-compose.override.yml`, `app.compose.yaml` and similar.
pub(super) fn is_docker_compose_uri(uri: &str) -> bool {
    let Some(name) = file_name(uri) else {
        return false;
    };
    let fixed = [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yaml",
        "docker-compose.yml",
    ]
    .iter()
    .any(|item| name.eq_ignore_ascii_case(item));
    let yaml = has_extension(name, &["yaml", "yml"]);
    let variant = (starts_with_ignore_ascii_case(name, "docker-compose.")
        || starts_with_ignore_ascii_case(name, "compose."))
        && yaml;
    let dot_compose = yaml
        && name
            .rsplit_once('.')
            .is_some_and(|(stem, _)| ends_with_ignore_ascii_case(stem, ".compose"));
    fixed || variant || dot_compose
}

pub(super) fn is_dockerfile_uri(uri: &str) -> bool {
    file_name(uri).is_some_and(|name| {
        name.eq_ignore_ascii_case("Dockerfile") || ends_with_ignore_ascii_case(name, ".dockerfile")
    })
}

/// Workflows under `.github/workflows` and composite actions under `.github/actions`.
pub(super) fn is_github_actions_uri(uri: &str) -> bool {
    let Some(path) = uri.strip_prefix("file:") else {
        return false;
    };
    let segments = path.split('/').collect::<Vec<_>>();
    let under = |directory: &str| {
        segments.windows(2).any(|window| {
            window[0].eq_ignore_ascii_case(".github") && window[1].eq_ignore_ascii_case(directory)
        })
    };
    let name = file_name(uri).unwrap_or_default();
    let workflow = (ends_with_ignore_ascii_case(name, ".yml")
        || ends_with_ignore_ascii_case(name, ".yaml"))
        && under("workflows");
    let action = under("actions")
        && (name.eq_ignore_ascii_case("action.yml") || name.eq_ignore_ascii_case("action.yaml"));
    workflow || action
}

/// `requirements*.txt` and `constraints*.txt` files.
pub(super) fn is_requirements_txt_uri(uri: &str) -> bool {
    file_name(uri).is_some_and(|name| {
        let lowercase = name.to_lowercase();
        (lowercase.contains("requirements") || lowercase.contains("constraints"))
            && has_extension(name, &["txt"])
    })
}

/// A JSON document with another name whose content looks like `package.json`.
pub(super) fn is_package_json_content(input: &DocumentInput, uri: &str) -> bool {
    let name = file_name(uri).unwrap_or(uri);
    matches!(input.language_id.as_str(), "json" | "jsonc")
        && has_extension(name, &["json", "jsonc"])
        && looks_like_package_json(&input.text)
}
