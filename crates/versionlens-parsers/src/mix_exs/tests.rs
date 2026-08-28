use crate::document::test_support::{extract_range, parse_fixture};
use versionlens_model::Ecosystem::Hex;

#[test]
fn parses_mix_exs_dependency_tuple_forms() {
    let text = package_file_fixture("parses-mix-exs-dependency-tuple-forms.txt");

    let dependencies = parse_fixture(text, "file:///work/mix.exs", "elixir");

    assert_eq!(dependencies.len(), 5);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Hex, "deps", "plug", ">= 1.15.0"),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">= 1.15.0"
    );
    assert_eq!(dependencies[1].name, "phoenix");
    assert_eq!(dependencies[1].group, "deps.dev,test");
    assert_eq!(dependencies[2].name, "plug");
    assert_eq!(dependencies[2].hosted_name.as_deref(), Some("plug_alias"));
    assert_eq!(dependencies[2].requirement, "~> 1.20");
    assert_eq!(dependencies[3].name, "gettext");
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[4].name, "local_dependency");
    assert_eq!(dependencies[4].hosted_url, Some("path".to_owned()));
}

#[test]
fn parses_mix_exs_umbrella_dependency_tuple() {
    let text = package_file_fixture("parses-mix-exs-umbrella-dependency-tuple.txt");

    let dependencies = parse_fixture(text, "file:///work/mix.exs", "elixir");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "shared_app");
    assert_eq!(dependencies[0].requirement, "in_umbrella:true");
    assert_eq!(dependencies[0].hosted_url.as_deref(), Some("umbrella"));
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "true"
    );
}

#[test]
fn parses_mix_exs_multiline_dependency_tuple() {
    let text = package_file_fixture("parses-mix-exs-multiline-dependency-tuplemix.exs");

    let dependencies = parse_fixture(text, "file:///work/mix.exs", "elixir");

    crate::support::tests::assert_single_git_dependency(&dependencies, "gettext");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "https://github.com/elixir-lang/gettext.git"
    );
}

#[test]
fn parses_mix_exs_dependencies_from_compact_list() {
    let text = package_file_fixture("parses-mix-exs-dependencies-from-compact-listmix.exs");

    let dependencies = parse_fixture(text, "file:///work/mix.exs", "elixir");

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_two_dependency_requirements(
        &dependencies,
        "plug",
        ">= 1.15.0",
        "phoenix",
        "~> 1.7",
    );
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/mix_exs/tests", name)
}
