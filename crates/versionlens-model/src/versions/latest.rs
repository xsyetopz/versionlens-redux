use semver::Version;

use crate::versions::VersionDialect;
use crate::versions::parse::parse_version;
use crate::versions::pep440;

mod prerelease;

use prerelease::prerelease_allowed;

/// Return the highest version in `versions`, ignoring prereleases.
pub fn latest_stable<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<String> {
    latest_version(versions, false)
}

/// Return the highest version in `versions`, including prereleases only when `include_prereleases` is set.
pub fn latest_version<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
) -> Option<String> {
    latest_version_with_prerelease_tags(versions, include_prereleases, &[])
}

/// Return the highest semver version in `versions`, where prereleases are allowed when `include_prereleases` is set or their tag is in `prerelease_tags`.
pub fn latest_version_with_prerelease_tags<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    versions
        .into_iter()
        .filter_map(|raw| parse_version_entry(raw, include_prereleases, prerelease_tags))
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, raw)| raw.to_owned())
}

/// Return the highest version in `versions` under `dialect`, applying the same prerelease rules as [`latest_version_with_prerelease_tags`].
pub fn latest_version_for_dialect<'a>(
    versions: impl IntoIterator<Item = &'a str>,
    include_prereleases: bool,
    prerelease_tags: &[String],
    dialect: VersionDialect,
) -> Option<String> {
    if matches!(dialect, VersionDialect::Semver | VersionDialect::Cargo) {
        return latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags);
    }

    versions
        .into_iter()
        .filter_map(|raw| {
            let version = pep440::parse_version(raw)?;
            pep440::prerelease_allowed(&version, include_prereleases, prerelease_tags)
                .then_some((version, raw))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, raw)| raw.to_owned())
}

fn parse_version_entry<'a>(
    raw: &'a str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<(Version, &'a str)> {
    if has_four_numeric_segments(raw) {
        return None;
    }
    let version = parse_version(raw)?;
    prerelease_allowed(&version, include_prereleases, prerelease_tags).then_some((version, raw))
}

fn has_four_numeric_segments(raw: &str) -> bool {
    let Some(start) = raw.find(|char: char| char.is_ascii_digit()) else {
        return false;
    };
    let numeric = &raw[start..];
    let core_end = numeric.find(['-', '+']).unwrap_or(numeric.len());
    let core = &numeric[..core_end];
    let mut segments = 0;
    for part in core.split('.') {
        if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
            return false;
        }
        segments += 1;
        if segments >= 4 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        latest_stable, latest_version, latest_version_for_dialect,
        latest_version_with_prerelease_tags,
    };
    use crate::versions::range::is_update_available;
    use crate::versions::{VersionDialect, normalized_version_for_dialect};

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

    #[test]
    fn normalizes_prerelease_candidate_case() {
        assert_eq!(
            latest_version_with_prerelease_tags(
                ["1.0.0", "2.0.0-BETA.1"],
                true,
                &["beta".to_owned()],
            ),
            Some("2.0.0-BETA.1".to_owned())
        );
    }

    #[test]
    fn pep440_dialect_accepts_extended_python_release_forms() {
        for (raw, normalized) in [
            ("1.0rc1", "1.0.0-rc.1"),
            ("1.0-rc.1", "1.0.0-rc.1"),
            ("1.0.post2", "1.0.0.post2"),
            ("1.0.dev3", "1.0.0.dev3"),
            ("1!2.0.0.4+linux.1", "1!2.0.0.4+linux.1"),
        ] {
            assert_eq!(
                normalized_version_for_dialect(raw, VersionDialect::Pep440).as_deref(),
                Some(normalized)
            );
        }
    }

    #[test]
    fn pep440_dialect_orders_epochs_postreleases_and_four_part_versions() {
        assert_eq!(
            latest_version_for_dialect(
                ["9.0.0.1", "1!1.0", "1!1.0.post1", "1!1.1.dev1"],
                false,
                &[],
                VersionDialect::Pep440,
            ),
            Some("1!1.0.post1".to_owned())
        );
        assert_eq!(
            latest_version_for_dialect(
                ["1.0", "1.1rc1", "1.1-rc.2", "1.1.dev3"],
                true,
                &["RC".to_owned()],
                VersionDialect::Pep440,
            ),
            Some("1.1-rc.2".to_owned())
        );
    }
}
