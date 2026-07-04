use super::{latest_stable, latest_version, latest_version_with_prerelease_tags};
use crate::range::is_update_available;

#[test]
fn picks_latest_stable_semver() {
    assert_eq!(
        latest_stable(["1.0.0", "1.2.0-beta.1", "1.1.9", "v2.0.0"]),
        Some("v2.0.0".to_owned())
    );
}

#[test]
fn can_include_prerelease_versions() {
    assert_eq!(
        latest_version(["1.0.0", "2.0.0-beta.1"], true),
        Some("2.0.0-beta.1".to_owned())
    );
    assert_eq!(
        latest_version(["3.0.0", "4.0.0rc1"], true),
        Some("4.0.0rc1".to_owned())
    );
    assert_eq!(
        latest_version(["8.0.4", "8.1.0.beta1"], true),
        Some("8.1.0.beta1".to_owned())
    );
    assert_eq!(
        latest_version(["1.0", "release-5.6.7-beta.1"], true),
        Some("1.0".to_owned())
    );
}

#[test]
fn ignores_non_semver_registry_version_labels_without_coercing_requirements() {
    assert_eq!(
        latest_stable(["v1.2.3", "release-5.6.7-beta.1", "build-5.6.8"]),
        Some("v1.2.3".to_owned())
    );
    assert!(!is_update_available("2.0.0", "release-1.0.0"));
}

#[test]
fn ignores_four_segment_registry_version_labels() {
    assert_eq!(
        latest_stable(["1.0.0.5", "999.0.0.1", "12.0.0-next.1", "9.5.12"]),
        Some("9.5.12".to_owned())
    );
}

#[test]
fn filters_prerelease_tags() {
    assert_eq!(
        latest_version_with_prerelease_tags(
            ["1.0.0", "2.0.0-beta.1", "3.0.0-rc.1"],
            true,
            &["beta".to_owned()],
        ),
        Some("2.0.0-beta.1".to_owned())
    );
    assert_eq!(
        latest_version_with_prerelease_tags(["1.0.0", "2.0.0rc1"], true, &["rc".to_owned()]),
        Some("2.0.0rc1".to_owned())
    );
}

#[test]
fn normalizes_prerelease_filter_tag_case() {
    assert_eq!(
        latest_version_with_prerelease_tags(
            ["1.0.0", "2.0.0-beta.1", "3.0.0-rc.1"],
            true,
            &["BETA".to_owned()],
        ),
        Some("2.0.0-beta.1".to_owned())
    );
}
