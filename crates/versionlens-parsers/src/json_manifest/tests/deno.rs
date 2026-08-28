use super::{DocumentInput, parse_document_with_dependency_paths};
use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::*;

#[test]
fn parses_deno_json_imports() {
    let text = package_file_fixture("parses-deno-json-imports.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].ecosystem, Deno);
    assert_deno_jsr_dependency(text, &dependencies, "@std/assert");
    assert_eq!(dependencies[1].name, "luca");
    assert_eq!(dependencies[1].requirement, "1.0.0");
    assert_eq!(dependencies[1].requirement_prefix, "jsr:@luca/cases@");
    assert_eq!(dependencies[1].hosted_name.as_deref(), Some("@luca/cases"));
    assert_eq!(dependencies[2].ecosystem, Npm);
    assert_eq!(dependencies[2].name, "chalk");
    assert_eq!(dependencies[2].requirement, "5.3.0");
    assert_eq!(dependencies[2].requirement_prefix, "npm:chalk@");
    assert_eq!(dependencies[2].hosted_name.as_deref(), Some("chalk"));
    assert_eq!(dependencies[3].name, "emptyJsr");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, "jsr:@std/assert@");
    assert_eq!(dependencies[4].ecosystem, Npm);
    assert_eq!(dependencies[4].name, "emptyNpm");
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, "npm:chalk@");
    assert_eq!(dependencies[5].ecosystem, Deno);
    assert_eq!(dependencies[5].name, "url");
    assert_eq!(dependencies[5].requirement, "https://deno.land/std/mod.ts");
    assert_eq!(dependencies[6].ecosystem, Deno);
    assert_eq!(dependencies[6].group, "scopes.https://deno.land/x/app/");
    assert_eq!(dependencies[6].name, "@scope/pkg");
    assert_eq!(dependencies[6].requirement, "0.2.0");
    assert_eq!(dependencies[6].requirement_prefix, "jsr:@scope/pkg@");
    assert_eq!(dependencies[7].ecosystem, Npm);
    assert_eq!(dependencies[7].group, "scopes.https://deno.land/x/other/");
    assert_eq!(dependencies[7].name, "chalk");
    assert_eq!(dependencies[7].requirement, "5.4.0");
    assert_eq!(dependencies[7].requirement_prefix, "npm:chalk@");
}

#[test]
fn deno_imports_preserve_import_specifiers_like_upstream_npm_parser() {
    let text = package_file_fixture(
        "deno-imports-preserve-import-specifiers-like-upstream-npm-parser.txt",
    );
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Deno);
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "^1.0.0");
    assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert@^1.0.0"
    );
    assert_eq!(dependencies[1].ecosystem, Npm);
    assert_eq!(dependencies[1].name, "chalk");
    assert_eq!(dependencies[1].requirement, "5.3.0");
    assert_eq!(dependencies[1].requirement_prefix, "npm:chalk@");
    assert_eq!(dependencies[2].ecosystem, Deno);
    assert_eq!(dependencies[2].name, "url");
    assert_eq!(dependencies[2].requirement, "https://deno.land/std/mod.ts");
}

#[test]
fn parses_configured_deno_scopes() {
    let text = package_file_fixture("parses-configured-deno-scopes.txt");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput::new(
            "file:///work/deno.json".to_owned(),
            "jsonc".to_owned(),
            text.to_owned(),
            None,
        ),
        &["imports", "scopes"],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "^1.0.0");
    assert_eq!(dependencies[1].ecosystem, Deno);
    assert_eq!(dependencies[1].group, "scopes.https://deno.land/x/app/");
    assert_eq!(dependencies[1].name, "@scope/pkg");
    assert_eq!(dependencies[1].requirement, "0.2.0");
    assert_eq!(dependencies[2].ecosystem, Npm);
    assert_eq!(dependencies[2].group, "scopes.https://deno.land/x/other/");
    assert_eq!(dependencies[2].name, "chalk");
    assert_eq!(dependencies[2].requirement, "5.4.0");
}

#[test]
fn parses_smoke_deno_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-deno-smoke-shapes.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "json");

    assert_eq!(dependencies.len(), 4);
    crate::support::tests::assert_dependency_group(&dependencies, 4, 0, Deno, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "1.0.19");
    assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/assert@1.0.19"
    );
    assert_eq!(dependencies[1].name, "luca");
    assert_eq!(dependencies[1].requirement, "1.0.0");
    assert_eq!(dependencies[2].ecosystem, Npm);
    assert_eq!(dependencies[2].name, "cowsay");
    assert_eq!(dependencies[2].requirement, "1.6.0");
    assert_eq!(dependencies[3].ecosystem, Deno);
    assert_eq!(dependencies[3].name, "cases");
    assert_eq!(
        dependencies[3].requirement,
        "https://deno.land/x/case/mod.ts"
    );
}

#[test]
fn parses_unversioned_deno_imports_as_empty_requirements_with_specifier_prefixes() {
    let text = package_file_fixture(
        "parses-unversioned-deno-imports-as-empty-requirements-with-specifier-prefixes.txt",
    );
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Deno);
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[1].ecosystem, Npm);
    assert_eq!(dependencies[1].name, "chalk");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, "npm:chalk@");
}

#[test]
fn parses_deno_scopes_by_default() {
    let text = package_file_fixture("parses-deno-scopes-by-default.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, "@std/assert");
    assert_eq!(dependencies[1].group, "scopes.https://deno.land/x/app/");
    assert_eq!(dependencies[1].name, "@scope/pkg");
    assert_eq!(dependencies[1].requirement, "0.2.0");
}

#[test]
fn parses_deno_json_jsr_project_version() {
    let text = package_file_fixture("parses-deno-json-jsr-project-version.json");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

    assert_eq!(dependencies.len(), 2);
    assert_deno_project_version(text, &dependencies[0]);
    assert_eq!(dependencies[1].group, "imports");
    assert_eq!(dependencies[1].name, "@std/assert");
}

#[test]
fn parses_import_map_json_dependencies() {
    let text = package_file_fixture("parses-import-map-json-dependencies.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/import_map.json", "json");

    crate::support::tests::assert_dependency_group(&dependencies, 4, 0, Deno, "imports");
    assert_eq!(dependencies[0].name, "@std/async");
    assert_eq!(dependencies[0].requirement, "^1.0.0");
    assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/async@");
    assert_eq!(dependencies[0].hosted_name.as_deref(), Some("@std/async"));
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "jsr:@std/async@^1.0.0"
    );
    assert_eq!(dependencies[1].name, "@std/async/");
    assert_eq!(dependencies[1].requirement, "^1.0.0");
    assert_eq!(dependencies[1].requirement_prefix, "jsr:/@std/async@");
    assert_eq!(dependencies[1].requirement_suffix, "/");
    assert_eq!(dependencies[1].hosted_name.as_deref(), Some("@std/async"));
    assert_eq!(dependencies[2].ecosystem, Npm);
    assert_eq!(dependencies[2].name, "chalk");
    assert_eq!(dependencies[2].requirement, "5.4.0");
    assert_eq!(dependencies[2].requirement_prefix, "npm:chalk@");
    assert_eq!(dependencies[2].hosted_name.as_deref(), Some("chalk"));
    assert_eq!(dependencies[3].group, "scopes.https://deno.land/x/app/");
    assert_eq!(dependencies[3].name, "cases");
    assert_eq!(dependencies[3].requirement, "1.0.0");
    assert_eq!(dependencies[3].requirement_prefix, "jsr:@luca/cases@");
    assert_eq!(dependencies[3].hosted_name.as_deref(), Some("@luca/cases"));
}

#[test]
fn parses_jsr_json_project_version() {
    let text = package_file_fixture("parses-jsr-json-project-version.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/jsr.json", "json");

    assert_eq!(dependencies.len(), 1);
    assert_deno_project_version(text, &dependencies[0]);
}

fn assert_deno_project_version(text: &str, dependency: &versionlens_model::Dependency) {
    assert_eq!(dependency.ecosystem, Deno);
    assert_eq!(dependency.group, "version");
    assert_eq!(dependency.name, "@scope/pkg");
    assert_eq!(dependency.requirement, "1.2.3");
    assert_eq!(extract_range(text, dependency.range), "@scope/pkg");
    assert_eq!(extract_range(text, dependency.requirement_range), "1.2.3");
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/json_manifest/tests/deno",
        name,
    )
}

fn assert_deno_jsr_dependency(
    text: &str,
    dependencies: &[versionlens_model::Dependency],
    name: &str,
) {
    assert_eq!(dependencies[0].group, "imports");
    assert_eq!(dependencies[0].name, name);
    assert_eq!(dependencies[0].requirement, "^1.0.0");
    assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
    crate::support::tests::assert_dependency_requirement_range(
        text,
        dependencies,
        0,
        "jsr:@std/assert@^1.0.0",
    );
}
