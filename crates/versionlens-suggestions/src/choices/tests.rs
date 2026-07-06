use super::{find_next_major, release_update_choices};

#[test]
fn release_update_choices_omits_major_when_latest_already_targets_next_major() {
    let releases = ["1.0.0", "1.0.1", "1.1.0", "2.0.0", "2.1.0"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("1.0.0", "2.1.0", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            ("latest", "2.1.0", "update"),
            ("minor", "1.1.0", "updateMinor"),
            ("patch", "1.0.1", "updatePatch")
        ]
    );
}

#[test]
fn release_update_choices_avoid_duplicate_latest_targets() {
    let releases = ["1.0.0", "1.0.1"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("1.0.0", "1.0.1", &releases);

    assert_eq!(choices.len(), 1);
    assert_eq!(choices[0].label, "latest");
}

#[test]
fn release_update_choices_sort_stable_suggestions_by_version_descending() {
    let releases = ["1.0.0", "1.0.1", "1.1.0", "2.0.0"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("1.0.0", "1.0.1", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            ("major", "2.0.0", "updateMajor"),
            ("minor", "1.1.0", "updateMinor"),
            ("latest", "1.0.1", "update")
        ]
    );
}

#[test]
fn release_update_choices_offer_bump_targets_for_ranges() {
    let releases = [
        "2.1.2", "3.0.0", "3.1.0", "4.0.0", "4.0.1", "4.1.10", "5.1.1", "5.2.0", "5.3.3", "5.4.5",
    ]
    .into_iter()
    .map(|value| value.to_owned())
    .collect::<Vec<_>>();

    let choices = release_update_choices("^4.1.0", "5.4.5", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [("latest", "5.4.5", "update"), ("bump", "4.1.10", "update")]
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
    let releases = ["1.0.0", "1.1.0", "2.0.0", "3.0.0"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("^1.0.0", "3.0.0", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            ("latest", "3.0.0", "update"),
            ("major", "2.0.0", "updateMajor"),
            ("bump", "1.1.0", "update")
        ]
    );
}

#[test]
fn release_update_choices_omit_major_when_fixed_requirement_is_missing() {
    let releases = ["0.5.1", "0.6.0", "1.0.0", "2.0.0"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("0.5.0", "2.0.0", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            ("latest", "2.0.0", "update"),
            ("minor", "0.6.0", "updateMinor"),
            ("patch", "0.5.1", "updatePatch")
        ]
    );
}

#[test]
fn release_update_choices_stop_major_discovery_at_invalid_versions() {
    let releases = ["2.0.0", "ABC", "3.0.0", "4.0.0"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("2.0.0", "4.0.0", &releases);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(labels, [("latest", "4.0.0", "update")]);
}

#[test]
fn find_next_major_handles_loose_versions() {
    let releases = ["2.0.0", "3.1.2ar"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();
    let current = crate::parse_semver("2.0.0").unwrap();

    assert_eq!(find_next_major(&current, &releases), Some(3));
}

#[test]
fn release_update_choices_offer_prerelease_targets_by_tag() {
    let versions = [
        "1.0.0-alpha",
        "1.0.1-alpha",
        "1.2.0-alpha",
        "1.2.0-dev",
        "1.2.0-beta",
    ]
    .into_iter()
    .map(|value| value.to_owned())
    .collect::<Vec<_>>();

    let choices = release_update_choices("~1.0.0-alpha", "1.2.0-beta", &versions);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

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
    let versions = ["1.1.0-foo-beta.1", "1.2.0-bar-beta.1"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("1.0.0", "1.2.0-bar-beta.1", &versions);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(labels, [("bar", "1.2.0-bar-beta.1", "update")]);
}

#[test]
fn release_update_choices_use_full_numeric_prerelease_label() {
    let versions = ["1.1.0-123.1", "1.2.0-123.4"]
        .into_iter()
        .map(|value| value.to_owned())
        .collect::<Vec<_>>();

    let choices = release_update_choices("1.0.0", "1.2.0-123.4", &versions);
    let labels = choices
        .iter()
        .map(|choice| {
            (
                choice.label.as_str(),
                choice.version.as_str(),
                choice.command.as_str(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(labels, [("123.4", "1.2.0-123.4", "update")]);
}
