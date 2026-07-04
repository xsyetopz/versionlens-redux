use versionlens_parsers::Dependency;
use versionlens_suggestions::Suggestion;

use super::latest::latest_text;

pub(in crate::presentation::title) fn update_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    title_with_version("latest", latest_text(suggestion))
}

pub(in crate::presentation::title) fn current_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    let label = if latest_text(suggestion).contains('-') {
        "latest prerelease"
    } else {
        "latest"
    };
    title_with_version(label, latest_text(suggestion))
}

pub(in crate::presentation::title) fn satisfies_latest_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    title_with_version("satisfies latest", latest_text(suggestion))
}

pub(in crate::presentation::title) fn satisfies_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    title_with_version("satisfies", latest_text(suggestion))
}

pub(in crate::presentation::title) fn fixed_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    title_with_version("fixed", latest_text(suggestion))
}

pub(in crate::presentation::title) fn directory_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    format!("file://{}", latest_text(suggestion))
}

pub(in crate::presentation::title) fn directory_not_found_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    title_with_version("not found", latest_text(suggestion))
}

pub(in crate::presentation::title) fn error_title_text(
    _: &Dependency,
    suggestion: &Suggestion,
) -> String {
    latest_text(suggestion).to_owned()
}

pub(in crate::presentation::title) fn invalid_range_title_text(
    _: &Dependency,
    _: &Suggestion,
) -> String {
    "invalid version range".to_owned()
}

pub(in crate::presentation::title) fn invalid_title_text(_: &Dependency, _: &Suggestion) -> String {
    "invalid version".to_owned()
}

pub(in crate::presentation::title) fn not_supported_title_text(
    _: &Dependency,
    _: &Suggestion,
) -> String {
    "not supported".to_owned()
}

pub(in crate::presentation::title) fn build_title_text(_: &Dependency, _: &Suggestion) -> String {
    "change build".to_owned()
}

pub(in crate::presentation::title) fn unresolved_title_text(
    dependency: &Dependency,
    _: &Suggestion,
) -> String {
    format!("{} unresolved", dependency.name)
}

fn title_with_version(label: &str, version: &str) -> String {
    if version.is_empty() {
        return label.to_owned();
    }
    format!("{label} {version}")
}
