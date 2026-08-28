use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use versionlens_model::Ecosystem::Go;

#[test]
fn parses_go_mod_dependencies() {
    let text = package_file_fixture("parses-go-mod-dependencies.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 7);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Go,
            "require",
            "example.test/one",
            "v1.2.3",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.2.3"
    );
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.test/local");
    assert_eq!(dependencies[1].requirement, "../local");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "../local"
    );
    assert_eq!(dependencies[2].group, "replace");
    assert_eq!(dependencies[2].name, "example.test/old");
    assert_eq!(dependencies[2].requirement, "./vendor/old");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "./vendor/old"
    );
    assert_eq!(dependencies[3].name, "github.com/docker/cli");
    assert_eq!(dependencies[3].requirement, "v26.1.3+incompatible");
    assert_eq!(dependencies[3].requirement_suffix, "+incompatible");
    assert_eq!(dependencies[4].name, "k8s.io/utils");
    assert_eq!(
        dependencies[4].requirement,
        "v0.0.0-20230726121419-3b25d923346b"
    );
    assert_eq!(dependencies[5].name, "example.test/prerelease");
    assert_eq!(dependencies[5].requirement, "v1.0.0-alpha-beta");
    assert_eq!(dependencies[6].group, "exclude");
    assert_eq!(dependencies[6].name, "example.test/bad");
}

#[test]
fn parses_go_work_replace_dependencies() {
    let text = package_file_fixture("parses-go-work-replace-dependencies.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Go, "use", "./app", "./app"),
    );
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.com/old");
    assert_eq!(dependencies[1].requirement, "./local");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "./local"
    );
}

#[test]
fn parses_quoted_go_module_paths_and_versions() {
    let text = package_file_fixture("parses-quoted-go-module-paths-and-versions.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/quoted");
    assert_eq!(dependencies[0].requirement, "v1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.2.3"
    );
    assert_eq!(dependencies[1].group, "exclude");
    assert_eq!(dependencies[1].name, "example.test/bad");
    assert_eq!(dependencies[1].requirement, "v0.5.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "v0.5.0"
    );
    assert_eq!(dependencies[2].group, "replace");
    assert_eq!(dependencies[2].name, "example.test/new");
    assert_eq!(dependencies[2].requirement, "v1.1.0");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "v1.1.0"
    );
}

#[test]
fn parses_interpreted_go_string_escape_sequences() {
    let text = package_file_fixture("parses-interpreted-go-string-escape-sequences.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/escaped");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "example.test\\/escaped"
    );
    assert_eq!(dependencies[0].requirement, "v1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.2.3"
    );
}

#[test]
fn parses_raw_string_go_module_paths_and_versions() {
    let text = package_file_fixture("parses-raw-string-go-module-paths-and-versionsgo.mod");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/raw");
    assert_eq!(dependencies[0].requirement, "v1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.2.3"
    );
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.test/new");
    assert_eq!(dependencies[1].requirement, "v1.1.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "v1.1.0"
    );
}

#[test]
fn parses_go_work_use_directories_as_local_dependencies() {
    let text = package_file_fixture("parses-go-work-use-directories-as-local-dependencies.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 3);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Go, "use", "./app", "./app"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "./app"
    );
    assert_eq!(dependencies[1].group, "use");
    assert_eq!(dependencies[1].name, "./lib");
    assert_eq!(dependencies[1].requirement, "./lib");
    assert_eq!(dependencies[2].group, "use");
    assert_eq!(dependencies[2].name, "../shared");
    assert_eq!(dependencies[2].requirement, "../shared");
}

#[test]
fn parses_smoke_go_mod_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-go-mod-smoke-shapes.txt");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 15);
    assert_eq!(dependencies[0].name, "github.com/docker/buildx");
    assert_eq!(dependencies[1].name, "github.com/docker/cli");
    assert_eq!(dependencies[1].requirement_suffix, "+incompatible");
    assert_eq!(dependencies[6].name, "golang.org/x/term");
    assert_eq!(dependencies[7].name, "k8s.io/api");
    assert_eq!(dependencies[11].name, "k8s.io/klog/v2");
    assert_eq!(dependencies[12].name, "k8s.io/kube-openapi");
    assert_eq!(
        dependencies[12].requirement,
        "v0.0.0-20231010175941-2dd684a91f00"
    );
    assert_eq!(dependencies[13].name, "k8s.io/utils");
    assert_eq!(
        dependencies[13].requirement,
        "v0.0.0-20230726121419-3b25d923346b"
    );
    assert_eq!(dependencies[14].group, "exclude");
    assert_eq!(dependencies[14].name, "github.com/docker/go-units");
}

#[test]
fn go_mod_dependency_without_version_parses_blank_requirement_like_upstream() {
    let text = package_file_fixture(
        "go-mod-dependency-without-version-parses-blank-requirement-like-upstream.txt",
    );
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/blank");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "example.test/blank"
    );
    crate::support::tests::assert_requirement_range_ends_at_dependency(&dependencies, 0);
}

#[test]
fn go_mod_versions_with_hyphenated_prerelease_identifiers_are_parsed() {
    let text = package_file_fixture(
        "go-mod-versions-with-hyphenated-prerelease-identifiers-are-parsed.txt",
    );
    let dependencies = parse_go_fixture(text);

    crate::support::tests::assert_dependency_group(&dependencies, 1, 0, Go, "require");
    crate::support::tests::assert_dependency_metadata(
        &dependencies,
        0,
        "require",
        "example.test/prerelease",
        "v1.0.0-alpha-beta",
    );
}

#[test]
fn parses_go_mod_hyphenated_prerelease_versions() {
    let text = package_file_fixture("parses-go-mod-hyphenated-prerelease-versionsgo.mod");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/prerelease");
    assert_eq!(dependencies[0].requirement, "v1.0.0-alpha-beta");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.0.0-alpha-beta"
    );
}

#[test]
fn go_mod_single_line_directives_require_literal_space_like_upstream() {
    let text = package_file_fixture(
        "go-mod-single-line-directives-require-literal-space-like-upstreamgo.mod",
    );
    let dependencies = parse_go_fixture(text);

    assert!(dependencies.is_empty());
}

#[test]
fn go_mod_replace_dependencies_use_replacement_source_ranges() {
    let text =
        package_file_fixture("go-mod-replace-dependencies-use-replacement-source-rangesgo.mod");
    let dependencies = parse_go_fixture(text);

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "replace");
    assert_eq!(dependencies[0].name, "example.test/local");
    assert_eq!(dependencies[0].requirement, "../local");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "../local"
    );
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.test/old");
    assert_eq!(dependencies[1].requirement, "./vendor/old");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "./vendor/old"
    );
}

fn parse_go_fixture(text: impl AsRef<str>) -> Vec<versionlens_model::Dependency> {
    parse_document(&DocumentInput::new(
        "file:///work/go.mod".to_owned(),
        "go.mod".to_owned(),
        text.as_ref().to_owned(),
        None,
    ))
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/go_mod/tests", name)
}
