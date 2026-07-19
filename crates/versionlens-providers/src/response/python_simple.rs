use serde_json::Value;
use serde_json::from_str;
use versionlens_versions::{
    VersionDialect::Pep440, compare_versions_for_dialect, latest_version_for_dialect,
    normalized_version_for_dialect,
};

use crate::python::normalized_project_name;

type DistributionFilenames = Vec<String>;

pub(crate) fn latest_python_simple_version(
    body: &str,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = python_simple_release_versions(body, package);
    latest_version_for_dialect(
        versions.iter().map(String::as_str),
        include_prereleases,
        prerelease_tags,
        Pep440,
    )
    .and_then(|version| normalized_version_for_dialect(&version, Pep440))
}

pub(crate) fn python_simple_release_versions(body: &str, package: &str) -> Vec<String> {
    let filenames = from_str::<Value>(body)
        .ok()
        .and_then(|value| simple_json_filenames(&value))
        .or_else(|| simple_html_filenames(body))
        .unwrap_or_default();
    let mut versions = filenames
        .iter()
        .filter_map(|filename| version_from_distribution_filename(filename, package))
        .collect::<Vec<_>>();
    versions.sort_by(|left, right| {
        compare_versions_for_dialect(left, right, Pep440)
            .unwrap_or_else(|| left.as_str().cmp(right.as_str()))
    });
    versions.dedup();
    versions
}

fn simple_json_filenames(value: &Value) -> Option<DistributionFilenames> {
    let files = value.get("files")?.as_array()?;
    Some(
        files
            .iter()
            .filter(|file| !simple_json_file_is_yanked(file))
            .filter_map(|file| file.get("filename")?.as_str().map(str::to_owned))
            .collect(),
    )
}

fn simple_json_file_is_yanked(file: &Value) -> bool {
    match file.get("yanked") {
        Some(Value::Bool(value)) => *value,
        Some(Value::String(reason)) => !reason.is_empty(),
        _ => false,
    }
}

fn simple_html_filenames(body: &str) -> Option<DistributionFilenames> {
    if !contains_ascii_case_insensitive(body, "<html")
        && !contains_ascii_case_insensitive(body, "<!doctype")
    {
        return None;
    }

    let mut filenames = vec![];
    let mut cursor = 0;
    while let Some(anchor_offset) = find_ascii_case_insensitive(&body[cursor..], "<a") {
        let anchor_start = cursor + anchor_offset;
        let Some(tag_end_offset) = body[anchor_start..].find('>') else {
            break;
        };
        let tag_end = anchor_start + tag_end_offset;
        let tag = &body[anchor_start..=tag_end];
        let text_start = tag_end + 1;
        let Some(close_offset) = find_ascii_case_insensitive(&body[text_start..], "</a>") else {
            break;
        };
        let close_start = text_start + close_offset;

        if !contains_ascii_case_insensitive(tag, "data-yanked") {
            let text = body[text_start..close_start].trim();
            if !text.is_empty() && !text.contains('<') {
                filenames.push(text.to_owned());
            }
        }
        cursor = close_start + 4;
    }
    Some(filenames)
}

fn version_from_distribution_filename(filename: &str, package: &str) -> Option<String> {
    let filename = filename
        .split(['?', '#'])
        .next()
        .unwrap_or(filename)
        .rsplit('/')
        .next()
        .unwrap_or(filename);
    let (stem, wheel) = distribution_stem(filename)?;
    let normalized_package = normalized_project_name(package);

    stem.match_indices('-')
        .filter_map(|(separator, _)| {
            (normalized_project_name(&stem[..separator]) == normalized_package)
                .then_some(&stem[separator + 1..])
        })
        .filter_map(|remainder| {
            let raw_version = if wheel {
                remainder.split('-').next().unwrap_or(remainder)
            } else {
                remainder
            };
            normalized_version_for_dialect(raw_version, Pep440)
        })
        .next_back()
}

fn distribution_stem(filename: &str) -> Option<(&str, bool)> {
    if let Some(stem) = filename.strip_suffix(".whl") {
        return Some((stem, true));
    }
    for suffix in [
        ".tar.gz", ".tar.bz2", ".tar.xz", ".tar.Z", ".tgz", ".tbz", ".zip",
    ] {
        if let Some(stem) = filename.strip_suffix(suffix) {
            return Some((stem, false));
        }
    }
    None
}

fn contains_ascii_case_insensitive(haystack: &str, needle: &str) -> bool {
    find_ascii_case_insensitive(haystack, needle).is_some()
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle.as_bytes()))
}
