use serde_json::Value;
use std::cmp::Ordering;
use std::cmp::Ordering::{
    Equal as OrderingEqual, Greater as OrderingGreater, Less as OrderingLess,
};

pub(crate) fn latest_hackage_version(
    value: &Value,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    if let Some(snapshot) = latest_stackage_snapshot(value, package) {
        return Some(snapshot);
    }

    value
        .as_object()?
        .iter()
        .filter_map(|(version, status)| {
            let status = status.as_str()?;
            (status != "deprecated"
                && hackage_prerelease_allowed(version, include_prereleases, prerelease_tags))
            .then_some(version.as_str())
        })
        .max_by(compare_hackage_versions)
        .map(|value| value.to_owned())
}

fn latest_stackage_snapshot(value: &Value, package: &str) -> Option<String> {
    let prefix = match package {
        "stackage-lts" => "lts-",
        "stackage-nightly" => "nightly-",
        _ => return None,
    };

    value
        .get("snapshots")?
        .as_array()?
        .iter()
        .flat_map(|group| group.as_array().into_iter().flatten())
        .filter_map(|snapshot| snapshot.as_array()?.first()?.as_str())
        .filter_map(|name| name.strip_prefix(prefix))
        .max_by(compare_stackage_snapshots)
        .map(|value| value.to_owned())
}

fn compare_stackage_snapshots(left: &&str, right: &&str) -> Ordering {
    versionlens_versions::compare_numeric_text(left, right, &['.']).then_with(|| left.cmp(right))
}

fn hackage_prerelease_allowed(
    version: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> bool {
    let Some((_, prerelease)) = version.split_once('-') else {
        return true;
    };
    include_prereleases
        && (prerelease_tags.is_empty()
            || prerelease_tags
                .iter()
                .any(|tag| prerelease.starts_with(tag.as_str())))
}

fn compare_hackage_versions(left: &&str, right: &&str) -> Ordering {
    let (left_release, left_prerelease) = split_prerelease(left);
    let (right_release, right_prerelease) = split_prerelease(right);
    versionlens_versions::compare_numeric_text(left_release, right_release, &['.']).then_with(
        || match (left_prerelease, right_prerelease) {
            (None, None) => OrderingEqual,
            (None, Some(_)) => OrderingGreater,
            (Some(_), None) => OrderingLess,
            (Some(left), Some(right)) => left.cmp(right),
        },
    )
}

fn split_prerelease(version: &str) -> (&str, Option<&str>) {
    version
        .split_once('-')
        .map(|(release, prerelease)| (release, Some(prerelease)))
        .unwrap_or((version, None))
}
