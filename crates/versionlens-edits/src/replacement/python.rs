use versionlens_versions::{
    VersionDialect, requirement_is_parseable_for_dialect, requirement_satisfies_latest_for_dialect,
};

pub(super) fn python_replacement(requirement: &str, latest: &str) -> String {
    if requirement.contains(',') {
        return replace_python_multi_constraint(requirement, latest);
    }

    leading_python_operator(requirement.trim_start()).map_or_else(
        || latest.to_owned(),
        |operator| python_operator_replacement(operator, latest),
    )
}

fn python_operator_replacement(operator: &str, latest: &str) -> String {
    let latest_public = python_public_version(latest);
    match operator {
        "<" | "<=" | "!=" => format!("=={latest}"),
        ">" | ">=" => format!(">={latest_public}"),
        "~=" => format!("~={latest_public}"),
        _ => format!("{operator}{latest}"),
    }
}

fn replace_python_multi_constraint(requirement: &str, latest: &str) -> String {
    let latest_public = python_public_version(latest);
    let parts = requirement
        .split(',')
        .map(|value| value.trim())
        .filter(|part| !python_exclusion_conflicts_with_latest(part, latest))
        .collect::<Vec<_>>();
    let has_upper_bound = parts.iter().any(|part| part.starts_with('<'));
    let has_lower_bound = parts.iter().any(|part| part.starts_with('>'));
    let has_upgrade_selector = parts.iter().any(|part| {
        matches!(
            leading_python_operator(part),
            Some(">" | ">=" | "===" | "==" | "~=")
        )
    });
    if !has_upgrade_selector {
        return format!("=={latest}");
    }
    let positive_bounds = parts
        .iter()
        .copied()
        .filter(|part| !part.starts_with("!="))
        .collect::<Vec<_>>()
        .join(", ");
    let update_upper_bound = has_upper_bound
        && (!has_lower_bound
            || !requirement_satisfies_latest_for_dialect(
                &positive_bounds,
                latest,
                VersionDialect::Pep440,
            ));

    parts
        .into_iter()
        .map(|part| {
            if part.starts_with('>') {
                format!(">={latest_public}")
            } else if update_upper_bound && part.starts_with('<') {
                format!("<={latest_public}")
            } else if let Some(operator @ ("===" | "==")) = leading_python_operator(part) {
                format!("{operator}{latest}")
            } else if leading_python_operator(part) == Some("~=") {
                format!("~={latest_public}")
            } else {
                part.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn python_public_version(version: &str) -> &str {
    version
        .split_once('+')
        .map_or(version, |(public, _)| public)
}

fn python_exclusion_conflicts_with_latest(part: &str, latest: &str) -> bool {
    if part.strip_prefix("!=").is_none() {
        return false;
    }

    requirement_is_parseable_for_dialect(part, latest, VersionDialect::Pep440)
        && !requirement_satisfies_latest_for_dialect(part, latest, VersionDialect::Pep440)
}

fn leading_python_operator(version: &str) -> Option<&'static str> {
    const OPERATORS: [&str; 8] = ["===", "==", "!=", "<=", ">=", "<", ">", "~="];

    OPERATORS
        .iter()
        .copied()
        .find(|operator| version.starts_with(operator))
}

#[cfg(test)]
mod tests {
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
}
