use super::{python_exclusion_conflicts_with_latest, python_replacement};

#[test]
fn standalone_exclusion_becomes_an_exact_target() {
    assert_eq!(python_replacement("!=1.0.0", "2.0.0"), "==2.0.0");
}

#[test]
fn advances_explicit_composite_python_selectors() {
    for (requirement, expected) in [
        ("~=1.0.0, !=1.1.0", "~=2.0.0, !=1.1.0"),
        ("==1.0.0, !=1.1.0", "==2.0.0, !=1.1.0"),
        ("===1.0.0, !=1.1.0", "===2.0.0, !=1.1.0"),
    ] {
        assert_eq!(python_replacement(requirement, "2.0.0"), expected);
    }
}

#[test]
fn removes_only_exclusions_that_reject_the_selected_latest() {
    assert_eq!(
        python_replacement(">=1, <3, !=2, !=1.5", "2.0.0"),
        ">=2.0.0, <3, !=1.5"
    );
    assert_eq!(
        python_replacement(">=1, <3, !=2.*, !=3.*", "2.0.0"),
        ">=2.0.0, <3, !=3.*"
    );
    assert!(python_exclusion_conflicts_with_latest(
        "!=1!2.0.0",
        "1!2.0.0"
    ));
}

#[test]
fn preserves_bounded_range_replacement_behavior() {
    for (requirement, expected) in [
        (">=1.0.0, <3.0.0", ">=2.0.0, <3.0.0"),
        (">=1.0.0, <3.0.0, !=1.5.0", ">=2.0.0, <3.0.0, !=1.5.0"),
        (">=1.0.0, <2.0.0", ">=2.0.0, <=2.0.0"),
        ("<3.0.0, !=1.5.0", "==2.0.0"),
    ] {
        assert_eq!(python_replacement(requirement, "2.0.0"), expected);
    }
}

#[test]
fn preserves_or_repairs_pep440_extended_bounds_for_the_selection() {
    for (requirement, latest, expected) in [
        (">=1!1.0, <1!2.0", "1!1.5", ">=1!1.5, <1!2.0"),
        (
            ">=1.0.post1, <1.0.post3",
            "1.0.post2",
            ">=1.0.post2, <1.0.post3",
        ),
        (">=1.0.dev1, <1.0", "1.0.dev2", ">=1.0.dev2, <=1.0.dev2"),
    ] {
        assert_eq!(python_replacement(requirement, latest), expected);
    }
}

#[test]
fn applies_pep440_local_label_semantics_to_replacements_and_exclusions() {
    assert_eq!(
        python_replacement(">=1.0, <2.0, !=1.5+linux", "1.5+mac"),
        ">=1.5, <2.0, !=1.5+linux"
    );
    assert_eq!(
        python_replacement(">=1.0, <2.0, !=1.5", "1.5+mac"),
        ">=1.5, <2.0"
    );
    assert_eq!(python_replacement("~=1.0", "1.5+linux"), "~=1.5");
    assert_eq!(python_replacement("==1.0", "1.5+linux"), "==1.5+linux");
}

#[test]
fn targets_latest_when_composite_contains_only_exclusions() {
    assert_eq!(python_replacement("!=1.0.0, !=1.5.0", "2.0.0"), "==2.0.0");
}
