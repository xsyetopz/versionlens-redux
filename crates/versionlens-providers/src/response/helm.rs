use versionlens_model::latest_version_with_prerelease_tags;

pub(crate) fn latest_helm_version(
    body: &str,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        helm_versions(body, package),
        include_prereleases,
        prerelease_tags,
    )
}

fn helm_versions<'a>(body: &'a str, package: &str) -> Vec<&'a str> {
    let package_key = format!("{package}:");
    let mut in_entries = false;
    let mut in_package = false;
    let mut versions = vec![];

    for line in body.lines() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if trimmed == "entries:" {
            in_entries = true;
            in_package = false;
            continue;
        }
        if !in_entries {
            continue;
        }
        if indent == 0 && !trimmed.is_empty() {
            in_entries = false;
            in_package = false;
            continue;
        }
        if indent == 2 && trimmed == package_key {
            in_package = true;
            continue;
        }
        if indent == 2 && trimmed.ends_with(':') {
            in_package = false;
            continue;
        }
        if in_package && let Some(version) = trimmed.strip_prefix("- version:") {
            versions.push(version.trim().trim_matches('"').trim_matches('\''));
        }
    }

    versions
}
