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
fn release_update_choices_offer_latest_for_stale_fixed_versions_without_history() {
    assert_latest_choice("1.0.0", "1.2.0");
}

#[test]
fn release_update_choices_omit_latest_when_range_already_resolves_latest_without_history() {
    assert!(release_update_choices("^1.0.0", "1.2.0", &[]).is_empty());
}

#[test]
fn release_update_choices_omit_noop_latest_for_current_ranges_without_history() {
    let choices = release_update_choices("^2.5.2", "2.5.2", &[]);

    assert!(choices.is_empty());
}

#[test]
fn release_update_choices_omit_noop_latest_for_current_registry_alias_ranges_without_history() {
    for requirement in ["npm:chalk@^2.5.2", "jsr:@scope/chalk@^2.5.2"] {
        assert!(release_update_choices(requirement, "2.5.2", &[]).is_empty());
    }
}

#[test]
fn release_update_choices_omit_latest_for_registry_alias_ranges_resolving_latest() {
    for requirement in ["npm:chalk@^1.0.0", "jsr:@scope/chalk@^1.0.0"] {
        assert!(release_update_choices(requirement, "1.2.0", &[]).is_empty());
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
        assert!(release_update_choices(requirement, "2.0.0", &[]).is_empty());
    }
}

#[test]
fn release_update_choices_omit_latest_aliases_and_unparseable_non_ranges_without_history() {
    assert!(release_update_choices("latest", "2.0.0", &[]).is_empty());
    assert!(release_update_choices("unparseable", "2.0.0", &[]).is_empty());
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
fn release_update_choices_offer_bump_targets_for_ranges() {
    let releases = releases(&[
        "2.1.2", "3.0.0", "3.1.0", "4.0.0", "4.0.1", "4.1.10", "5.1.1", "5.2.0", "5.3.3", "5.4.5",
    ]);

    let choices = release_update_choices("^4.1.0", "5.4.5", &releases);
    let labels = labels(&choices);

    assert_eq!(
        labels,
        [
            ("downgrade", "2.1.2", "update"),
            ("downgrade", "3.0.0", "update"),
            ("downgrade", "3.1.0", "update"),
            ("downgrade", "4.0.0", "update"),
            ("downgrade", "4.0.1", "update"),
            ("bump", "4.1.10", "update"),
            ("version", "5.1.1", "update"),
            ("version", "5.2.0", "update"),
            ("version", "5.3.3", "update"),
            ("latest", "5.4.5", "update")
        ]
    );
}

#[test]
fn release_update_choices_omit_noop_latest_for_current_ranges() {
    let releases = vec!["2.5.2".to_owned()];

    let choices = release_update_choices("^2.5.2", "2.5.2", &releases);

    assert!(choices.is_empty());
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
fn release_update_choices_offer_every_older_stable_release_as_a_downgrade() {
    let releases = releases(&["1.0.0", "1.5.0", "2.0.0"]);

    let choices = release_update_choices("2.0.0", "2.0.0", &releases);

    assert_eq!(
        labels(&choices),
        [
            ("downgrade", "1.0.0", "update"),
            ("downgrade", "1.5.0", "update")
        ]
    );
}

#[test]
fn release_update_choices_normalize_v_prefixes_and_never_offer_latest_as_a_noop() {
    let releases = releases(&["v6.0.0", "v7.0.0", "v7.0.1"]);

    let choices = release_update_choices("v7.0.1", "v7.0.1", &releases);

    assert_eq!(
        labels(&choices),
        [
            ("downgrade", "v6.0.0", "update"),
            ("downgrade", "v7.0.0", "update")
        ]
    );
}

#[test]
fn release_update_choices_offer_every_intermediate_release_for_a_range() {
    let releases = releases(&["0.5.0", "0.6.0", "0.6.1", "0.7.0"]);

    let choices = release_update_choices("^0.5.0", "0.7.0", &releases);
    let versions = choices
        .iter()
        .map(|choice| choice.version.as_str())
        .collect::<Vec<_>>();

    assert_eq!(versions, ["0.6.0", "0.6.1", "0.7.0"]);
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
