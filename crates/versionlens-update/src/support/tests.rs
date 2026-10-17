pub(crate) fn empty_range() -> versionlens_model::Range {
    versionlens_model::Range {
        start: versionlens_model::Position {
            line: 0,
            character: 0,
        },
        end: versionlens_model::Position {
            line: 0,
            character: 0,
        },
    }
}

pub(crate) fn test_dependency(name: &str, requirement: &str) -> versionlens_model::Dependency {
    versionlens_model::Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: versionlens_model::Ecosystem::Cargo,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: empty_range(),
        requirement_range: empty_range(),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
        canonical_reference: None,
    }
}

pub(crate) fn fixture(base: &str, name: &str) -> &'static str {
    versionlens_test_support::static_fixture!(base, name).expect("test fixture must be readable")
}

pub(crate) fn range(
    start_line: u32,
    start_character: u32,
    end_line: u32,
    end_character: u32,
) -> versionlens_model::Range {
    versionlens_model::Range {
        start: versionlens_model::Position {
            line: start_line,
            character: start_character,
        },
        end: versionlens_model::Position {
            line: end_line,
            character: end_character,
        },
    }
}
