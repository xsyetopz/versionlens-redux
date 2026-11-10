use semver::Version;
use std::cmp::Ordering::{
    Equal as OrderingEqual, Greater as OrderingGreater, Less as OrderingLess,
};
use std::collections::BTreeSet;
use versionlens_model::VersionDialect;

use crate::suggestion::UpdateChoice;
use crate::support;
use crate::{BumpKind, PlanStatus, has_a_floor};

type UpdateChoices = Vec<UpdateChoice>;

/// The update choices for `requirement` among `versions`, prereleases included.
pub fn release_update_choices(
    requirement: &str,
    latest: &str,
    versions: &[String],
) -> UpdateChoices {
    release_update_choices_with_prereleases(
        requirement,
        latest,
        versions,
        true,
        &[],
        VersionDialect::Semver,
    )
}

/// The update choices for `requirement` among `versions` in `dialect`.
///
/// Prerelease choices are added only when `include_prereleases` is set, for
/// the tags in `prerelease_tags`.
pub fn release_update_choices_with_prereleases(
    requirement: &str,
    latest: &str,
    versions: &[String],
    include_prereleases: bool,
    prerelease_tags: &[String],
    dialect: VersionDialect,
) -> UpdateChoices {
    let requirement = comparable_requirement(requirement);
    let mut choices = stable_update_choices(requirement, latest, versions, dialect);
    if include_prereleases {
        choices.extend(prerelease_update_choices(
            requirement,
            versions,
            prerelease_tags,
        ));
    }
    deduplicate_update_choices(&mut choices);
    choices
}

fn comparable_requirement(requirement: &str) -> &str {
    versionlens_model::registry_alias_requirement(requirement).unwrap_or(requirement)
}

fn stable_update_choices(
    requirement: &str,
    latest: &str,
    versions: &[String],
    dialect: VersionDialect,
) -> UpdateChoices {
    // Cargo reads a bare version as a caret requirement, not a pin.
    let current =
        parse_release_semver(requirement.trim()).filter(|_| dialect != VersionDialect::Cargo);
    let Some(current) = current else {
        return range_update_choices(requirement, latest, versions, dialect);
    };
    let stable_versions = stable_versions(versions);

    let mut choices = vec![];
    if !current_matches_latest(&current, latest)
        && parse_release_semver(latest)
            .is_none_or(|latest| release_precedence(&latest) > release_precedence(&current))
    {
        push_unique_choice(&mut choices, latest_choice_label(latest), latest, "update");
    }

    if stable_versions.is_empty() {
        return if versions.is_empty() { choices } else { vec![] };
    }

    if let Some(version) = next_major(&current, versions, &stable_versions) {
        push_unique_choice(&mut choices, "major", &version, "updateMajor");
    }
    if let Some(version) = next_minor(&current, &stable_versions) {
        push_unique_choice(&mut choices, "minor", &version, "updateMinor");
    }
    if let Some(version) = next_patch(&current, &stable_versions) {
        push_unique_choice(&mut choices, "patch", &version, "updatePatch");
    }

    sort_choices_incrementally(&mut choices);
    choices
}

fn current_matches_latest(current: &Version, latest: &str) -> bool {
    parse_release_semver(latest.trim()).is_some_and(|latest| {
        current.major == latest.major
            && current.minor == latest.minor
            && current.patch == latest.patch
            && current.pre == latest.pre
    })
}

/// Choices for a range requirement, built from the bounded bump set of
/// `update::plan`. With no version list, the registry's latest is the only
/// known release.
fn range_update_choices(
    requirement: &str,
    latest: &str,
    versions: &[String],
    dialect: VersionDialect,
) -> UpdateChoices {
    if !looks_like_range_requirement(requirement) && dialect != VersionDialect::Cargo {
        return vec![];
    }

    let known_latest;
    let versions = if versions.is_empty() {
        known_latest = [latest.to_owned()];
        &known_latest[..]
    } else {
        versions
    };
    let plan = crate::plan(requirement, dialect, versions);
    if plan.status == PlanStatus::AllowsLatest && !has_a_floor(requirement, dialect) {
        return vec![];
    }
    let mut choices = vec![];
    for bump in plan.bumps {
        let (label, command) = match bump.kind {
            BumpKind::Satisfying => ("bump", "update"),
            BumpKind::Patch => ("patch", "updatePatch"),
            BumpKind::Minor => ("minor", "updateMinor"),
            BumpKind::Major => ("major", "updateMajor"),
            BumpKind::Latest => (latest_choice_label(&bump.version), "update"),
        };
        push_unique_choice(&mut choices, label, &bump.version, command);
    }

    sort_choices_incrementally(&mut choices);
    choices
}

fn release_precedence(version: &Version) -> (u64, u64, u64, &semver::Prerelease) {
    (version.major, version.minor, version.patch, &version.pre)
}

fn latest_choice_label(latest: &str) -> &'static str {
    if crate::parse_semver(latest.trim()).is_ok_and(|version| !version.pre.is_empty()) {
        "latest prerelease"
    } else {
        "latest"
    }
}

fn next_major(
    current: &Version,
    versions: &[String],
    stable_versions: &[(&str, Version)],
) -> Option<String> {
    let next_major = find_next_major(current, versions)?;

    latest_release_for_major(next_major, stable_versions)
}

fn latest_release_for_major(major: u64, stable_versions: &[(&str, Version)]) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(_, version)| version.major == major)
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn find_next_major(current: &Version, versions: &[String]) -> Option<u64> {
    let mut found_current = false;

    for raw in versions {
        let Ok(version) = crate::parse_semver(raw) else {
            if found_current {
                let major = loose_major(raw)?;
                if major > current.major {
                    return Some(major);
                }
            }
            continue;
        };
        if !version.pre.is_empty() {
            continue;
        }
        if !found_current {
            if version == *current {
                found_current = true;
            }
            continue;
        }
        if version.major > current.major {
            return Some(version.major);
        }
    }

    None
}

fn loose_major(raw: &str) -> Option<u64> {
    let major = raw.split_once('.')?.0;
    (!major.is_empty() && major.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| major.parse().ok())
        .flatten()
}

fn next_minor(current: &Version, stable_versions: &[(&str, Version)]) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(_, version)| version.major == current.major && version.minor > current.minor)
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn next_patch(current: &Version, stable_versions: &[(&str, Version)]) -> Option<String> {
    stable_versions
        .iter()
        .filter(|(_, version)| {
            version.major == current.major
                && version.minor == current.minor
                && version.patch > current.patch
        })
        .max_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(release, _)| (*release).to_owned())
}

fn stable_versions(versions: &[String]) -> Vec<(&str, Version)> {
    versions
        .iter()
        .filter_map(|version| stable_version(version.as_str()))
        .collect()
}

fn stable_version(release: &str) -> Option<(&str, Version)> {
    let version = parse_release_semver(release)?;
    version.pre.is_empty().then_some((release, version))
}

fn parse_release_semver(value: &str) -> Option<Version> {
    crate::parse_semver(value.trim()).ok().or_else(|| {
        value
            .trim()
            .strip_prefix(['v', 'V'])
            .and_then(|value| crate::parse_semver(value).ok())
    })
}

fn prerelease_update_choices(
    requirement: &str,
    versions: &[String],
    prerelease_tags: &[String],
) -> UpdateChoices {
    let Some(minimum) = minimum_version(requirement) else {
        return vec![];
    };

    let mut groups = vec![];
    for (order, raw) in versions.iter().enumerate() {
        if !prerelease_tag_allowed(raw, prerelease_tags) {
            continue;
        }
        let Some((key, label, version)) = prerelease_parts(raw) else {
            continue;
        };
        upsert_prerelease_group(
            &mut groups,
            PrereleaseGroup {
                key,
                label,
                raw: raw.to_owned(),
                version,
                order,
            },
        );
    }

    groups.sort_by_key(|group| group.order);
    let mut choices = groups
        .into_iter()
        .filter(|group| group.version > minimum)
        .map(|group| UpdateChoice {
            label: group.label,
            version: group.raw,
            replacement: None,
            command: "update".to_owned(),
        })
        .collect::<Vec<_>>();
    choices.reverse();
    choices
}

struct PrereleaseGroup {
    key: String,
    label: String,
    raw: String,
    version: Version,
    order: usize,
}

fn upsert_prerelease_group(groups: &mut Vec<PrereleaseGroup>, next: PrereleaseGroup) {
    if let Some(group) = groups.iter_mut().find(|group| group.key == next.key) {
        group.label = next.label;
        group.raw = next.raw;
        group.version = next.version;
        group.order = next.order;
        return;
    }

    groups.push(next);
}

fn prerelease_parts(raw: &str) -> Option<(String, String, Version)> {
    let version = crate::parse_semver(raw).ok()?;
    if version.pre.is_empty() {
        return None;
    }
    let suffix = prerelease_suffix(raw)?;
    let first = suffix.split('.').next()?;
    let key = friendly_prerelease_name(raw).unwrap_or_else(|| first.to_owned());
    let label = tag_label(first, suffix);
    Some((key, label, version))
}

fn prerelease_tag_allowed(raw: &str, allowed_tags: &[String]) -> bool {
    if allowed_tags.is_empty() {
        return true;
    }

    prerelease_suffix(raw)
        .and_then(|suffix| suffix.split('.').next())
        .is_some_and(|first| {
            allowed_tags
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(first))
        })
}

fn prerelease_suffix(raw: &str) -> Option<&str> {
    raw.split_once('-').map(|(_, suffix)| suffix)
}

fn tag_label(first: &str, suffix: &str) -> String {
    let label = first
        .chars()
        .take_while(|char| !char.is_ascii_digit() && *char != '-')
        .collect::<String>()
        .to_ascii_lowercase();
    if label.is_empty() {
        suffix.to_ascii_lowercase()
    } else {
        label
    }
}

fn friendly_prerelease_name(raw: &str) -> Option<String> {
    let lowered = raw.to_ascii_lowercase();
    for group in COMMON_PRERELEASE_IDENTITIES {
        for common in *group {
            if follows_hyphen(&lowered, common) {
                return Some((*common).to_owned());
            }
        }
    }
    None
}

fn follows_hyphen(value: &str, needle: &str) -> bool {
    value
        .match_indices(needle)
        .any(|(index, _)| index > 0 && value.as_bytes()[index - 1] == b'-')
}

const COMMON_PRERELEASE_IDENTITIES: &[&[&str]] = &[
    &["legacy"],
    &["alpha", "preview", "a"],
    &["beta", "b"],
    &["next"],
    &["milestone", "m"],
    &["rc", "cr"],
    &["snapshot"],
    &["release", "final", "ga"],
    &["sp"],
];

fn minimum_version(requirement: &str) -> Option<Version> {
    let token = support::minimum_version_token(requirement)?;
    let normalized = support::normalize_version_token(token)?;
    crate::parse_semver(&normalized).ok()
}

/// Appends a choice unless one already targets the same version.
pub fn push_unique_choice(
    choices: &mut Vec<UpdateChoice>,
    label: &str,
    version: &str,
    command: &str,
) {
    let target = semantic_update_target(version);
    if choices
        .iter()
        .any(|choice| semantic_update_target(&choice.version) == target)
    {
        return;
    }

    choices.push(UpdateChoice {
        label: label.to_owned(),
        version: version.to_owned(),
        replacement: None,
        command: command.to_owned(),
    });
}

/// The normalized form of `value`, or the trimmed text when it has none.
pub fn semantic_update_target(value: &str) -> String {
    let value = value.trim();
    versionlens_model::normalized_version(value).unwrap_or_else(|| value.to_owned())
}

/// Removes choices that target the same version as an earlier one.
pub fn deduplicate_update_choices(choices: &mut Vec<UpdateChoice>) {
    let mut targets = BTreeSet::new();
    let mut unique = Vec::with_capacity(choices.len());
    for choice in choices.drain(..) {
        let target = semantic_update_target(&choice.version);
        if targets.insert(target.clone()) {
            unique.push(choice);
            continue;
        }
        if let Some(existing) = unique
            .iter_mut()
            .find(|existing| semantic_update_target(&existing.version) == target)
            && existing.label.starts_with("latest")
            && !choice.label.starts_with("latest")
        {
            *existing = choice;
        }
    }
    *choices = unique;
}

fn sort_choices_incrementally(choices: &mut UpdateChoices) {
    choices.sort_by(|left, right| {
        match (
            parse_release_semver(left.version.trim()),
            parse_release_semver(right.version.trim()),
        ) {
            (Some(left), Some(right)) => left.cmp(&right),
            (Some(_), None) => OrderingLess,
            (None, Some(_)) => OrderingGreater,
            (None, None) => OrderingEqual,
        }
    });
    if let Some(index) = choices
        .iter()
        .position(|choice| choice.label.starts_with("latest"))
    {
        let latest = choices.remove(index);
        choices.push(latest);
    }
}

fn looks_like_range_requirement(requirement: &str) -> bool {
    requirement.contains([
        '^', '~', '>', '<', '=', '*', '|', ',', '[', ']', '(', ')', ' ',
    ])
}

#[cfg(test)]
mod tests {
    use super::{find_next_major, release_update_choices, release_update_choices_with_prereleases};

    fn releases(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn labels(choices: &[crate::suggestion::UpdateChoice]) -> Vec<(&str, &str, &str)> {
        choices
            .iter()
            .map(|choice| {
                (
                    choice.label.as_str(),
                    choice.version.as_str(),
                    choice.command.as_str(),
                )
            })
            .collect()
    }

    fn assert_latest_choice(requirement: &str, latest: &str) {
        let choices = release_update_choices(requirement, latest, &[]);

        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].label, "latest");
        assert_eq!(choices[0].version, latest);
        assert_eq!(choices[0].command, "update");
    }

    #[test]
    fn release_update_choices_omits_major_when_latest_already_targets_next_major() {
        let releases = releases(&["1.0.0", "1.0.1", "1.1.0", "2.0.0", "2.1.0"]);

        let choices = release_update_choices("1.0.0", "2.1.0", &releases);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [
                ("patch", "1.0.1", "updatePatch"),
                ("minor", "1.1.0", "updateMinor"),
                ("latest", "2.1.0", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_avoid_duplicate_latest_targets() {
        let releases = releases(&["1.0.0", "1.0.1"]);

        let choices = release_update_choices("1.0.0", "1.0.1", &releases);

        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].label, "latest");
    }

    #[test]
    fn release_update_choices_deduplicate_equivalent_renderings_but_keep_builds_distinct() {
        let mut choices = vec![];

        super::push_unique_choice(&mut choices, "latest", "v1.2.3", "update");
        super::push_unique_choice(&mut choices, "minor", "1.2.3", "updateMinor");
        super::push_unique_choice(&mut choices, "build", "1.2.3+linux", "update");

        assert_eq!(
            labels(&choices),
            [
                ("latest", "v1.2.3", "update"),
                ("build", "1.2.3+linux", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_offer_latest_for_stale_fixed_versions_without_history() {
        assert_latest_choice("1.0.0", "1.2.0");
    }

    #[test]
    fn release_update_choices_raise_the_floor_to_latest_without_history() {
        let choices = release_update_choices("^1.0.0", "1.2.0", &[]);

        assert_eq!(labels(&choices), [("latest", "1.2.0", "update")]);
    }

    #[test]
    fn release_update_choices_omit_noop_latest_for_current_ranges_without_history() {
        let choices = release_update_choices("^2.5.2", "2.5.2", &[]);

        assert_eq!(choices, Vec::new());
    }

    #[test]
    fn release_update_choices_omit_noop_latest_for_current_registry_alias_ranges_without_history() {
        for requirement in ["npm:chalk@^2.5.2", "jsr:@scope/chalk@^2.5.2"] {
            assert_eq!(
                release_update_choices(requirement, "2.5.2", &[]),
                Vec::new()
            );
        }
    }

    #[test]
    fn release_update_choices_raise_the_floor_of_registry_alias_ranges_to_latest() {
        for requirement in ["npm:chalk@^1.0.0", "jsr:@scope/chalk@^1.0.0"] {
            let choices = release_update_choices(requirement, "1.2.0", &[]);

            assert_eq!(labels(&choices), [("latest", "1.2.0", "update")]);
        }
    }

    #[test]
    fn release_update_choices_offer_latest_for_stale_fixed_registry_aliases_without_history() {
        assert_latest_choice("npm:chalk@1.0.0", "1.2.0");
    }

    #[test]
    fn release_update_choices_preserve_prerelease_choices_for_registry_aliases() {
        let versions = releases(&["1.0.0-alpha", "1.0.1-alpha"]);

        let choices = release_update_choices_with_prereleases(
            "jsr:@scope/chalk@~1.0.0-alpha",
            "1.0.0-alpha",
            &versions,
            true,
            &[],
            versionlens_model::VersionDialect::Semver,
        );

        assert_eq!(choices.len(), 1);
        assert_eq!(choices[0].label, "alpha");
        assert_eq!(choices[0].version, "1.0.1-alpha");
        assert_eq!(choices[0].command, "update");
    }

    #[test]
    fn release_update_choices_treat_registry_alias_tags_as_non_actionable_without_history() {
        for requirement in [
            "npm:chalk",
            "npm:chalk@latest",
            "jsr:@scope/chalk@next",
            "jsr:@scope/chalk@unparseable",
        ] {
            assert_eq!(
                release_update_choices(requirement, "2.0.0", &[]),
                Vec::new()
            );
        }
    }

    #[test]
    fn release_update_choices_omit_latest_aliases_and_unparseable_non_ranges_without_history() {
        assert_eq!(release_update_choices("latest", "2.0.0", &[]), Vec::new());
        assert_eq!(
            release_update_choices("unparseable", "2.0.0", &[]),
            Vec::new()
        );
    }

    #[test]
    fn release_update_choices_sort_stable_suggestions_incrementally() {
        let releases = releases(&["1.0.0", "1.0.1", "1.1.0", "2.0.0"]);

        let choices = release_update_choices("1.0.0", "1.0.1", &releases);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [
                ("minor", "1.1.0", "updateMinor"),
                ("major", "2.0.0", "updateMajor"),
                ("latest", "1.0.1", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_offer_bounded_targets_for_ranges() {
        let releases = releases(&[
            "2.1.2", "3.0.0", "3.1.0", "4.0.0", "4.0.1", "4.1.10", "5.1.1", "5.2.0", "5.3.3",
            "5.4.5",
        ]);

        let choices = release_update_choices("^4.1.0", "5.4.5", &releases);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [("bump", "4.1.10", "update"), ("latest", "5.4.5", "update")]
        );
    }

    #[test]
    fn release_update_choices_omit_noop_latest_for_current_ranges() {
        let releases = vec!["2.5.2".to_owned()];

        let choices = release_update_choices("^2.5.2", "2.5.2", &releases);

        assert_eq!(choices, Vec::new());
    }

    #[test]
    fn release_update_choices_offer_intermediate_major_targets_for_ranges() {
        let releases = releases(&["1.0.0", "1.1.0", "2.0.0", "3.0.0"]);

        let choices = release_update_choices("^1.0.0", "3.0.0", &releases);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [
                ("bump", "1.1.0", "update"),
                ("major", "2.0.0", "updateMajor"),
                ("latest", "3.0.0", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_normalize_v_prefixes_and_never_offer_latest_as_a_noop() {
        let releases = releases(&["v6.0.0", "v7.0.0", "v7.0.1"]);

        let choices = release_update_choices("v7.0.1", "v7.0.1", &releases);

        assert_eq!(choices, Vec::new());
    }

    #[test]
    fn release_update_choices_do_not_list_intermediate_releases_for_a_range() {
        let releases = releases(&["0.5.0", "0.6.0", "0.6.1", "0.7.0"]);

        let choices = release_update_choices("^0.5.0", "0.7.0", &releases);
        let versions = choices
            .iter()
            .map(|choice| choice.version.as_str())
            .collect::<Vec<_>>();

        assert_eq!(versions, ["0.7.0"]);
    }

    #[test]
    fn release_update_choices_order_incremental_releases_before_latest_for_fixed_versions() {
        let releases = releases(&["0.6.0", "0.6.1", "0.7.0"]);

        let choices = release_update_choices("0.6.0", "0.7.0", &releases);
        let versions = choices
            .iter()
            .map(|choice| choice.version.as_str())
            .collect::<Vec<_>>();

        assert_eq!(versions, ["0.6.1", "0.7.0"]);
    }

    #[test]
    fn release_update_choices_omit_major_when_fixed_requirement_is_missing() {
        let releases = releases(&["0.5.1", "0.6.0", "1.0.0", "2.0.0"]);

        let choices = release_update_choices("0.5.0", "2.0.0", &releases);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [
                ("patch", "0.5.1", "updatePatch"),
                ("minor", "0.6.0", "updateMinor"),
                ("latest", "2.0.0", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_stop_major_discovery_at_invalid_versions() {
        let releases = releases(&["2.0.0", "ABC", "3.0.0", "4.0.0"]);

        let choices = release_update_choices("2.0.0", "4.0.0", &releases);
        let labels = labels(&choices);

        assert_eq!(labels, [("latest", "4.0.0", "update")]);
    }

    #[test]
    fn find_next_major_handles_loose_versions() {
        let releases = releases(&["2.0.0", "3.1.2ar"]);
        let current = crate::parse_semver("2.0.0").unwrap();

        assert_eq!(find_next_major(&current, &releases), Some(3));
    }

    #[test]
    fn release_update_choices_offer_prerelease_targets_by_tag() {
        let versions = releases(&[
            "1.0.0-alpha",
            "1.0.1-alpha",
            "1.2.0-alpha",
            "1.2.0-dev",
            "1.2.0-beta",
        ]);

        let choices = release_update_choices("~1.0.0-alpha", "1.2.0-beta", &versions);
        let labels = labels(&choices);

        assert_eq!(
            labels,
            [
                ("beta", "1.2.0-beta", "update"),
                ("dev", "1.2.0-dev", "update"),
                ("alpha", "1.2.0-alpha", "update")
            ]
        );
    }

    #[test]
    fn release_update_choices_group_prereleases_by_common_identity_after_hyphen() {
        let versions = releases(&["1.1.0-foo-beta.1", "1.2.0-bar-beta.1"]);

        let choices = release_update_choices("1.0.0", "1.2.0-bar-beta.1", &versions);
        let labels = labels(&choices);

        assert_eq!(labels, [("bar", "1.2.0-bar-beta.1", "update")]);
    }

    #[test]
    fn release_update_choices_use_full_numeric_prerelease_label() {
        let versions = releases(&["1.1.0-123.1", "1.2.0-123.4"]);

        let choices = release_update_choices("1.0.0", "1.2.0-123.4", &versions);
        let labels = labels(&choices);

        assert_eq!(labels, [("123.4", "1.2.0-123.4", "update")]);
    }
}
