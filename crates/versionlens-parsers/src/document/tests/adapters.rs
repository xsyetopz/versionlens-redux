fn adapter_input(uri: &str, text: &str) -> DocumentInput {
    DocumentInput::new(uri.to_owned(), "plaintext".to_owned(), text.to_owned(), None)
}

fn with_paths(uri: &str, text: &str, paths: &[&str]) -> Vec<versionlens_model::Dependency> {
    crate::parse_document_with_dependency_paths(&adapter_input(uri, text), paths)
}

const NPM: &str = "{\"name\":\"demo\",\"version\":\"0.4.4\",\"dependencies\":{\"a\":\"^1\"}}";
const CARGO: &str = "[package]\nname = \"demo\"\nversion = \"1.2.3\"\nrust-version = \"1.70\"\n\n[workspace.package]\nversion = \"2.0.0\"\n\n[dependencies]\nserde = \"1\"\n";

#[test]
fn npm_version_as_a_user_path_is_not_a_dependency() {
    let uri = "file:///work/package.json";

    assert_eq!(with_paths(uri, NPM, &["version"]), vec![]);
    assert!(
        with_paths(uri, NPM, &["version", "dependencies"])
            .iter()
            .all(|dependency| dependency.group != "version")
    );
    assert_eq!(parse_project_versions(&adapter_input(uri, NPM)).len(), 1);
}

#[test]
fn npm_without_user_paths_uses_the_defaults() {
    let dependencies = parse_document(&adapter_input("file:///work/package.json", NPM));

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "a");
}

#[test]
fn cargo_version_fields_are_not_dependencies_under_any_user_path() {
    let uri = "file:///work/Cargo.toml";

    for paths in [
        &["package.version"][..],
        &["package.*"],
        &["*.*"],
        &["workspace.package"],
        &["workspace.*"],
        &["workspace.package.version"],
    ] {
        let dependencies = with_paths(uri, CARGO, paths);
        assert!(
            dependencies
                .iter()
                .all(|dependency| dependency.name != "version"),
            "{paths:?} emitted {dependencies:?}"
        );
    }
    let versions = parse_project_versions(&adapter_input(uri, CARGO));
    assert_eq!(
        versions
            .iter()
            .map(|dependency| dependency.requirement.as_str())
            .collect::<Vec<_>>(),
        ["1.2.3", "2.0.0"]
    );
}

#[test]
fn gleam_ranges_are_exact_with_crlf_line_breaks() {
    let text = "name = \"app\"\r\nversion = \"1.2.3\"\r\n\r\n[dependencies]\r\ngleam_stdlib = \">= 0.34.0 and < 2.0.0\" # c\r\nlocal = { path = \"../local\" }\r\n";
    let dependencies = with_paths("file:///work/gleam.toml", text, &[]);

    assert_eq!(dependencies.len(), 3);
    let expected = [
        ("version = \"1.2.3\"", "1.2.3"),
        (
            "gleam_stdlib = \">= 0.34.0 and < 2.0.0\" # c",
            ">= 0.34.0 and < 2.0.0",
        ),
        ("local = { path = \"../local\" }", "../local"),
    ];
    for (dependency, (line, requirement)) in dependencies.iter().zip(expected) {
        assert_eq!(extract_range(text, dependency.range), line);
        assert_eq!(
            extract_range(text, dependency.requirement_range),
            requirement
        );
    }
}

#[test]
fn julia_ranges_are_exact_with_crlf_line_breaks() {
    let project = "name = \"App\"\r\nversion = \"0.1.0\"\r\n\r\n[deps]\r\nJSON = \"682c06a0\"\r\n\r\n[compat]\r\nJSON = \"0.21\"\r\n";
    let dependencies = with_paths("file:///work/Project.toml", project, &[]);

    assert_eq!(dependencies.len(), 2);
    assert_eq!(
        extract_range(project, dependencies[0].range),
        "version = \"0.1.0\""
    );
    assert_eq!(
        extract_range(project, dependencies[1].range),
        "JSON = \"0.21\""
    );
    assert_eq!(
        extract_range(project, dependencies[1].requirement_range),
        "0.21"
    );

    let manifest = "[[deps.JSON]]\r\nuuid = \"x\"\r\nversion = \"0.21.4\"\r\n\r\n[[deps.Local]]\r\npath = \"/home/Local\"\r\n";
    let dependencies = with_paths("file:///work/Manifest.toml", manifest, &[]);

    assert_eq!(dependencies.len(), 2);
    let block = |dependency: &versionlens_model::Dependency| {
        (
            dependency.range.start.line,
            dependency.range.start.character,
            dependency.range.end.line,
            dependency.range.end.character,
        )
    };
    assert_eq!(block(&dependencies[0]), (0, 0, 4, 0));
    assert_eq!(
        extract_range(manifest, dependencies[0].requirement_range),
        "0.21.4"
    );
    assert_eq!(block(&dependencies[1]), (4, 0, 6, 0));
    assert_eq!(
        extract_range(manifest, dependencies[1].requirement_range),
        "/home/Local"
    );
}

#[test]
fn stack_yaml_reads_git_github_quoted_and_commented_entries() {
    let text = "resolver: lts-21.25\nextra-deps:\n  - acme-1.2.3 # pinned\n  - \"quoted-2.0\"\n  - 'single-3.0'\n  - github: owner/repo\n    commit: abc\n  - git: https://example.test/x.git\n    commit: def\n  - ./local\n";
    let dependencies = with_paths("file:///work/stack.yaml", text, &[]);

    let summary = dependencies
        .iter()
        .map(|dependency| {
            (
                dependency.name.as_str(),
                dependency.requirement.as_str(),
                dependency.hosted_url.as_deref(),
                extract_range(text, dependency.requirement_range),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [
            ("stackage-lts", "21.25", Some("stackage"), "21.25"),
            ("acme", "1.2.3", None, "1.2.3"),
            ("quoted", "2.0", None, "2.0"),
            ("single", "3.0", None, "3.0"),
            ("owner/repo", "owner/repo", Some("git"), "owner/repo"),
            (
                "https://example.test/x.git",
                "https://example.test/x.git",
                Some("git"),
                "https://example.test/x.git"
            ),
            ("./local", "./local", Some("path"), "./local"),
        ]
    );
}
