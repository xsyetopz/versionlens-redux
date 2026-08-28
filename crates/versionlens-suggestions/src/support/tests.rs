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
    }
}
