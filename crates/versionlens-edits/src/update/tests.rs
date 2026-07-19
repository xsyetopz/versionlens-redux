use semver::{Version, VersionReq};
use versionlens_model::Dependency;
use versionlens_model::{Position, Range};
use versionlens_suggestions::Suggestion;
use versionlens_suggestions::SuggestionStatus::{
    Current as StatusCurrent, UpdateAvailable as StatusUpdateAvailable,
};

use super::update_edits;
use versionlens_model::Ecosystem::{Cargo, Cpp, Dotnet, Go, Npm, Python, Ruby};

#[test]
fn replaces_requirement_range_with_latest_version() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "serde".to_owned(),
            requirement: "1.0.0".to_owned(),
            ecosystem: Cargo,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 0, 0, 5),
            requirement_range: range(0, 9, 0, 14),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("1.0.228".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].range, range(0, 9, 0, 14));
    assert_eq!(edits[0].new_text, "1.0.228");
}

#[test]
fn inserts_missing_dotnet_version_attribute() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "NoVersionAttribute".to_owned(),
            requirement: "*".to_owned(),
            ecosystem: Dotnet,
            group: "PackageReference".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 27, 0, 45),
            requirement_range: range(0, 46, 0, 46),
            requirement_prefix: "Version=\"".to_owned(),
            requirement_suffix: "\"".to_owned(),
        },
        latest: Some("8.0.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].range, range(0, 46, 0, 46));
    assert_eq!(edits[0].new_text, "Version=\"8.0.0\"");
}

#[test]
fn preserves_python_requirement_operators() {
    for (requirement, expected) in [
        ("==1.2.3", "==2.0.0"),
        ("!=1.2.3", "==2.0.0"),
        (">1.2.3", ">=2.0.0"),
        ("<=1.2.3", "==2.0.0"),
        ("~=1.2.3", "~=2.0.0"),
        (">=1.0.0, <3.0.0", ">=2.0.0, <3.0.0"),
        (">=1.0.0, <2.0.0", ">=2.0.0, <=2.0.0"),
        ("<3.0.0, !=1.5.0", "==2.0.0"),
        (">=1.0.0, !=1.1.0", ">=2.0.0, !=1.1.0"),
        ("~=1.0.0, !=1.1.0", "~=2.0.0, !=1.1.0"),
        ("==1.0.0, !=1.1.0", "==2.0.0, !=1.1.0"),
        (">=1, <3, !=2", ">=2.0.0, <3"),
        (">=1, <3, !=2.*, !=3.*", ">=2.0.0, <3, !=3.*"),
    ] {
        let edits = update_edits(&[Suggestion {
            dependency: python_dependency(requirement),
            latest: Some("2.0.0".to_owned()),
            resolved: None,
            status: StatusUpdateAvailable,
            builds: vec![],
            choices: vec![],
        }]);

        assert_eq!(edits[0].new_text, expected);
    }
}

#[test]
fn preserves_pep440_extended_bounds_and_local_exclusions() {
    for (requirement, latest, expected) in [
        (">=1!1.0, <1!2.0", "1!1.5", ">=1!1.5, <1!2.0"),
        (
            ">=1.0.post1, <1.0.post3",
            "1.0.post2",
            ">=1.0.post2, <1.0.post3",
        ),
        (">=1.0.dev1, <1.0", "1.0.dev2", ">=1.0.dev2, <=1.0.dev2"),
        (
            ">=1.0, <2.0, !=1.5+linux",
            "1.5+mac",
            ">=1.5, <2.0, !=1.5+linux",
        ),
    ] {
        let edits = update_edits(&[Suggestion {
            dependency: python_dependency(requirement),
            latest: Some(latest.to_owned()),
            resolved: None,
            status: StatusUpdateAvailable,
            builds: vec![],
            choices: vec![],
        }]);

        assert_eq!(edits[0].new_text, expected);
    }
}

#[test]
fn preserves_ruby_requirement_operators() {
    for (requirement, expected) in [
        ("~> 1.2.3", "~> 2.0.0"),
        (">= 1.2.3", ">= 2.0.0"),
        ("!=1.2.3", "==2.0.0"),
    ] {
        let edits = update_edits(&[Suggestion {
            dependency: ruby_dependency(requirement),
            latest: Some("2.0.0".to_owned()),
            resolved: None,
            status: StatusUpdateAvailable,
            builds: vec![],
            choices: vec![],
        }]);

        assert_eq!(edits[0].new_text, expected);
    }
}

#[test]
fn inserts_missing_ruby_version_argument() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "nokogiri".to_owned(),
            requirement: "*".to_owned(),
            ecosystem: Ruby,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 5, 0, 13),
            requirement_range: range(0, 14, 0, 14),
            requirement_prefix: ", '".to_owned(),
            requirement_suffix: "'".to_owned(),
        },
        latest: Some("1.18.10".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].range, range(0, 14, 0, 14));
    assert_eq!(edits[0].new_text, ", '1.18.10'");
}

#[test]
fn inserts_missing_xmake_requirement_with_separator() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "openssl".to_owned(),
            requirement: "*".to_owned(),
            ecosystem: Cpp,
            group: "add_requires".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 14, 0, 21),
            requirement_range: range(0, 21, 0, 21),
            requirement_prefix: " ".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("3.0.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].range, range(0, 21, 0, 21));
    assert_eq!(edits[0].new_text, " 3.0.0");
}

#[test]
fn switches_ruby_github_branches_to_ref_updates() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "rails/rails".to_owned(),
            requirement: "main".to_owned(),
            ecosystem: Ruby,
            group: "dependencies".to_owned(),
            hosted_url: Some("https://api.github.com/repos/rails/rails/commits".to_owned()),
            hosted_name: Some("rails".to_owned()),
            range: range(0, 5, 0, 12),
            requirement_range: range(0, 30, 0, 44),
            requirement_prefix: r#"ref: ""#.to_owned(),
            requirement_suffix: r#"""#.to_owned(),
        },
        latest: Some("a1b2c3d4e5f6".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, r#"ref: "a1b2c3d4e5f6""#);
}

#[test]
fn switches_ruby_github_tags_to_ref_updates_for_sha_latest() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "rails/rails".to_owned(),
            requirement: "v6.0.0".to_owned(),
            ecosystem: Ruby,
            group: "dependencies".to_owned(),
            hosted_url: Some("https://api.github.com/repos/rails/rails/tags".to_owned()),
            hosted_name: Some("rails".to_owned()),
            range: range(0, 5, 0, 12),
            requirement_range: range(0, 30, 0, 44),
            requirement_prefix: r#"tag: ""#.to_owned(),
            requirement_suffix: r#"""#.to_owned(),
        },
        latest: Some("a1b2c3d4e5f6".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, r#"ref: "a1b2c3d4e5f6""#);
}

#[test]
fn preserves_go_incompatible_suffix() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "github.com/docker/cli".to_owned(),
            requirement: "v26.1.3+incompatible".to_owned(),
            ecosystem: Go,
            group: "require".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 1, 0, 22),
            requirement_range: range(0, 23, 0, 43),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "+incompatible".to_owned(),
        },
        latest: Some("v27.0.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, "v27.0.0+incompatible");
}

#[test]
fn preserves_semver_requirement_operators() {
    for (requirement, expected) in [
        ("^1.2.3", "^2.0.0"),
        ("~1.2.3", "~2.0.0"),
        (">=1.2.3", ">=2.0.0"),
        ("v1.2.3", "2.0.0"),
    ] {
        let edits = update_edits(&[Suggestion {
            dependency: Dependency {
                name: "left-pad".to_owned(),
                requirement: requirement.to_owned(),
                ecosystem: Npm,
                group: "dependencies".to_owned(),
                hosted_url: None,
                hosted_name: None,
                range: range(0, 0, 0, 8),
                requirement_range: range(0, 8, 0, 8 + u32::try_from(requirement.len()).unwrap()),
                requirement_prefix: "".to_owned(),
                requirement_suffix: "".to_owned(),
            },
            latest: Some("2.0.0".to_owned()),
            resolved: None,
            status: StatusUpdateAvailable,
            builds: vec![],
            choices: vec![],
        }]);

        assert_eq!(edits[0].new_text, expected);
    }
}

#[test]
fn rewritten_requirements_select_the_requested_upgrade() {
    for (dependency, latest, expected) in [
        (python_dependency("<=1.2.3"), "2.0.0", "==2.0.0"),
        (python_dependency("<3.0.0, !=1.5.0"), "2.0.0", "==2.0.0"),
        (ruby_dependency("!=1.2.3"), "2.0.0", "==2.0.0"),
        (npm_dependency("<2.0.0"), "1.2.4", "1.2.4"),
        (npm_dependency(">=1.2.3 <2.0.0"), "1.2.4", ">=1.2.4"),
    ] {
        let edits = update_edits(&[Suggestion {
            dependency,
            latest: Some(latest.to_owned()),
            resolved: None,
            status: StatusUpdateAvailable,
            builds: vec![],
            choices: vec![],
        }]);

        assert_eq!(edits[0].new_text, expected);
        assert_requirement_matches(&edits[0].new_text, latest);
        assert_requirement_rejects(&edits[0].new_text, "1.2.3");
    }
}

#[test]
fn preserves_npm_alias_specifier_when_replacing_versions() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "chalk".to_owned(),
            requirement: "npm:chalk@^5.3.0".to_owned(),
            ecosystem: Npm,
            group: "imports".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 14, 0, 21),
            requirement_range: range(0, 24, 0, 41),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("6.0.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, "npm:chalk@^6.0.0");
}

#[test]
fn replaces_empty_ranges_with_latest_version() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "left-pad".to_owned(),
            requirement: ">1 <1".to_owned(),
            ecosystem: Npm,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 0, 0, 8),
            requirement_range: range(0, 8, 0, 13),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("5.0.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, "5.0.0");
}

#[test]
fn strips_v_prefix_from_github_semver_tag_updates() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "octokit/core.js".to_owned(),
            requirement: "^1".to_owned(),
            ecosystem: Npm,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 0, 0, 42),
            requirement_range: range(0, 12, 0, 42),
            requirement_prefix: "github:octokit/core.js#semver:".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("v2.5.0".to_owned()),
        resolved: None,
        status: StatusUpdateAvailable,
        builds: vec![],
        choices: vec![],
    }]);

    assert_eq!(edits[0].new_text, "github:octokit/core.js#semver:2.5.0");
}

#[test]
fn skips_current_dependencies() {
    let edits = update_edits(&[Suggestion {
        dependency: Dependency {
            name: "serde".to_owned(),
            requirement: "^1.0.0".to_owned(),
            ecosystem: Cargo,
            group: "dependencies".to_owned(),
            hosted_url: None,
            hosted_name: None,
            range: range(0, 0, 0, 5),
            requirement_range: range(0, 9, 0, 15),
            requirement_prefix: "".to_owned(),
            requirement_suffix: "".to_owned(),
        },
        latest: Some("1.0.228".to_owned()),
        resolved: None,
        status: StatusCurrent,
        builds: vec![],
        choices: vec![],
    }]);

    assert!(edits.is_empty());
}

fn python_dependency(requirement: &str) -> Dependency {
    Dependency {
        name: "requests".to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Python,
        group: "requirements".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: range(0, 0, 0, 8),
        requirement_range: range(0, 8, 0, 8 + u32::try_from(requirement.len()).unwrap()),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn ruby_dependency(requirement: &str) -> Dependency {
    Dependency {
        name: "rails".to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ruby,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: range(0, 0, 0, 5),
        requirement_range: range(0, 5, 0, 5 + u32::try_from(requirement.len()).unwrap()),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn npm_dependency(requirement: &str) -> Dependency {
    Dependency {
        name: "left-pad".to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Npm,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range: range(0, 0, 0, 8),
        requirement_range: range(0, 8, 0, 8 + u32::try_from(requirement.len()).unwrap()),
        requirement_prefix: "".to_owned(),
        requirement_suffix: "".to_owned(),
    }
}

fn assert_requirement_matches(requirement: &str, version: &str) {
    let requirement = normalize_exact_requirement(requirement);
    let requirement = VersionReq::parse(&requirement).expect("parse rewritten requirement");
    let version = Version::parse(version).expect("parse selected version");
    assert!(requirement.matches(&version));
}

fn assert_requirement_rejects(requirement: &str, version: &str) {
    let requirement = normalize_exact_requirement(requirement);
    let requirement = VersionReq::parse(&requirement).expect("parse rewritten requirement");
    let version = Version::parse(version).expect("parse obsolete version");
    assert!(!requirement.matches(&version));
}

fn normalize_exact_requirement(requirement: &str) -> String {
    requirement
        .strip_prefix("==")
        .map_or_else(|| requirement.to_owned(), |version| format!("={version}"))
}

fn range(start_line: u32, start_character: u32, end_line: u32, end_character: u32) -> Range {
    Range {
        start: Position {
            line: start_line,
            character: start_character,
        },
        end: Position {
            line: end_line,
            character: end_character,
        },
    }
}
