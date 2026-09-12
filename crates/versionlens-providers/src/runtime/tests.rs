use versionlens_model::{Dependency, Ecosystem, Position, Range};

use super::RuntimeSource;

fn yarn(requirement: &str) -> Dependency {
    runtime("yarn", requirement, Ecosystem::Npm, "packageManager", None)
}

fn runtime(
    name: &str,
    requirement: &str,
    ecosystem: Ecosystem,
    group: &str,
    hosted_name: Option<&str>,
) -> Dependency {
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 0,
        },
    };
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem,
        group: group.to_owned(),
        hosted_url: None,
        hosted_name: hosted_name.map(str::to_owned),
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}

#[test]
fn go_and_deno_use_their_official_release_catalogs() {
    let go = RuntimeSource::for_dependency(&runtime(
        "go",
        "stable",
        Ecosystem::Go,
        "with.go-version",
        None,
    ))
    .unwrap();
    assert_eq!(go.url(), "https://go.dev/dl/?mode=json&include=all");
    let body = r#"[{"version":"go1.25.1","stable":true},{"version":"go1.24.7","stable":true},{"version":"go1.26rc1","stable":false}]"#;
    assert_eq!(
        go.preferred_release(body, "stable").unwrap().as_deref(),
        Some("1.25.1")
    );
    assert_eq!(
        go.preferred_release(body, "oldstable").unwrap().as_deref(),
        Some("1.24.7")
    );

    let deno = RuntimeSource::for_dependency(&runtime(
        "deno",
        "^2",
        Ecosystem::Deno,
        "with.deno-version",
        None,
    ))
    .unwrap();
    assert_eq!(deno.url(), "https://dl.deno.land/versions.json");
    assert_eq!(
        deno.versions(r#"{"cli":["v2.1.0","v2.0.0"]}"#, "^2")
            .unwrap(),
        ["2.1.0", "2.0.0"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "deno",
            &"a".repeat(40),
            Ecosystem::Deno,
            "with.deno-version",
            None,
        ))
        .is_err()
    );
}

#[test]
fn python_and_ruby_catalogs_accept_only_supported_interpreters() {
    let python = RuntimeSource::for_dependency(&runtime(
        "python",
        "3.12",
        Ecosystem::Python,
        "with.python-version",
        None,
    ))
    .unwrap();
    assert!(
        python
            .url()
            .ends_with("actions/python-versions/main/versions-manifest.json")
    );
    assert_eq!(
        python
            .versions(r#"[{"version":"3.12.2","stable":true}]"#, "3.12")
            .unwrap(),
        ["3.12.2"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "python",
            "pypy3.10",
            Ecosystem::Python,
            "with.python-version",
            None,
        ))
        .is_err()
    );

    let ruby = RuntimeSource::for_dependency(&runtime(
        "ruby",
        "3.3",
        Ecosystem::Ruby,
        "with.ruby-version",
        None,
    ))
    .unwrap();
    assert!(
        ruby.url()
            .ends_with("ruby/setup-ruby/master/ruby-builder-versions.json")
    );
    assert_eq!(
        ruby.versions(r#"{"ruby":["3.3.0","head"],"jruby":["9.4.0.0"]}"#, "3.3")
            .unwrap(),
        ["3.3.0"]
    );
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "ruby",
            "jruby-9.4",
            Ecosystem::Ruby,
            "with.ruby-version",
            None,
        ))
        .is_err()
    );
}

#[test]
fn temurin_and_dotnet_parse_sdk_release_metadata() {
    let java = RuntimeSource::for_dependency(&runtime(
        "java",
        "17",
        Ecosystem::Maven,
        "with.java-version",
        Some("temurin"),
    ))
    .unwrap();
    assert!(
        java.url()
            .contains("api.adoptium.net/v3/info/release_names")
    );
    assert_eq!(
        java.versions(
            r#"{"releases":["jdk-21+35","jdk-21.0.12+8","jdk-21.0.12.1+1","jdk8u504-b01"]}"#,
            "21"
        )
        .unwrap(),
        ["21.0.0", "21.0.12", "21.0.12.1", "8.0.504"]
    );
    for release in ["jdk-not-a-version", "jdk-21rc1+1", "jdk8u504-bx"] {
        let body = format!(r#"{{"releases":["{release}"]}}"#);
        let error = java.versions(&body, "21").unwrap_err();
        assert_eq!(
            error, "Temurin release response contains an invalid version",
            "{release}"
        );
    }
    assert!(
        RuntimeSource::for_dependency(&runtime(
            "java",
            "17",
            Ecosystem::Maven,
            "with.java-version",
            Some("zulu"),
        ))
        .is_err()
    );

    let dotnet = RuntimeSource::for_dependency(&runtime(
        "dotnet",
        "8.0.4xx",
        Ecosystem::Dotnet,
        "with.dotnet-version",
        None,
    ))
    .unwrap();
    assert!(dotnet.url().ends_with("/8.0/releases.json"));
    let body = r#"{"releases":[{"sdk":{"version":"8.0.405"},"sdks":[{"version":"8.0.309"},{"version":"8.0.406"}]}]}"#;
    assert_eq!(
        dotnet.versions(body, "8.0.4xx").unwrap(),
        ["8.0.405", "8.0.406"]
    );
}

#[test]
fn yarn_uses_the_release_source_for_its_version_family() {
    let classic = RuntimeSource::for_dependency(&yarn("1.22.22")).unwrap();
    assert!(matches!(classic, RuntimeSource::PackageManager("yarn")));
    assert_eq!(classic.url(), "https://registry.npmjs.org/yarn");

    for requirement in ["4.14.1", "stable", "^4.0.0", ">=2.0.0, <5.0.0"] {
        let modern = RuntimeSource::for_dependency(&yarn(requirement)).unwrap();
        assert!(matches!(modern, RuntimeSource::YarnModern));
        assert_eq!(modern.url(), "https://repo.yarnpkg.com/tags");
    }

    assert!(matches!(
        RuntimeSource::for_dependency(&yarn("^1.22.0")).unwrap(),
        RuntimeSource::PackageManager("yarn")
    ));
    assert!(RuntimeSource::for_dependency(&yarn(">=1.0.0")).is_err());
}

#[test]
fn modern_yarn_uses_published_tags_and_aliases() {
    let source = RuntimeSource::YarnModern;
    let body =
        r#"{"aliases":{"latest":"4.2.0","stable":"4.1.1"},"tags":["4.2.0","4.1.1","3.8.7"]}"#;
    assert_eq!(
        source.versions(body, "4.0.0").unwrap(),
        ["4.2.0", "4.1.1", "3.8.7"]
    );
    assert_eq!(
        source.preferred_release(body, "4.0.0").unwrap(),
        Some("4.2.0".to_owned())
    );
    assert_eq!(
        source.preferred_release(body, "stable").unwrap(),
        Some("4.1.1".to_owned())
    );
}

#[test]
fn modern_yarn_requires_aliases_to_reference_published_tags() {
    let source = RuntimeSource::YarnModern;
    for body in [
        r#"{"tags":["4.2.0"]}"#,
        r#"{"aliases":{"latest":"invalid"},"tags":["invalid"]}"#,
        r#"{"aliases":{"latest":"4.3.0"},"tags":["4.2.0"]}"#,
    ] {
        assert!(source.preferred_release(body, "4.0.0").is_err());
    }
}
