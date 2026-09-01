use super::{Dependency, VersionableKind};
use crate::{Ecosystem, Position, Range};

fn dependency(ecosystem: Ecosystem, group: &str, name: &str, requirement: &str) -> Dependency {
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 1,
        },
    };
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
    }
}

#[test]
fn version_category_preserves_exact_project_predicates() {
    assert_eq!(
        dependency(Ecosystem::Deno, "version", "plain", "1.0.0").versionable_kind(),
        VersionableKind::Dependency
    );
    assert_eq!(
        dependency(Ecosystem::Deno, "version", "@scope/pkg", "1.0.0").versionable_kind(),
        VersionableKind::ProjectVersion
    );
    assert_eq!(
        dependency(Ecosystem::Pub, "version", "other", "1.0.0").versionable_kind(),
        VersionableKind::Dependency
    );
    assert_eq!(
        dependency(Ecosystem::Pub, "version", "version", "1.0.0").versionable_kind(),
        VersionableKind::ProjectVersion
    );
}
