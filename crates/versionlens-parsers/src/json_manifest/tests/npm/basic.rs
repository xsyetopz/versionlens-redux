use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Npm;

#[test]
fn parses_package_json_dependency_groups() {
    let (text, dependencies) = parse_package_fixture("package-json-dependency-groups.txt");

    assert_eq!(dependencies.len(), 9);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "@types/node");
    assert_eq!(dependencies[0].requirement, "26.0.1");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "26.0.1"
    );
    assert_eq!(dependencies[1].name, "pacote");
    assert_eq!(dependencies[1].requirement, "11.1.9");
    assert_eq!(dependencies[1].requirement_prefix, "npm:pacote@");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "npm:pacote@11.1.9"
    );
    assert_eq!(dependencies[2].name, "@types/react");
    assert_eq!(dependencies[2].requirement, "^19.0.0");
    assert_eq!(dependencies[2].requirement_prefix, "npm:@types/react@");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "npm:@types/react@^19.0.0"
    );
    assert_eq!(dependencies[3].name, "types-react");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, "npm:types-react@");
    assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
    assert_eq!(dependencies[4].name, "@types/react");
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, "npm:@types/react@");
    assert_eq!(extract_range(text, dependencies[4].requirement_range), "");
    assert_eq!(dependencies[5].name, "local-file");
    assert_eq!(dependencies[5].requirement, "file:../local");
    assert_eq!(
        extract_range(text, dependencies[5].requirement_range),
        "file:../local"
    );
    assert_eq!(dependencies[6].name, "local");
    assert_eq!(dependencies[6].requirement, "file:../local/package.json");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "link:../local"
    );
    assert_eq!(dependencies[7].group, "devDependencies");
    assert_eq!(dependencies[8].group, "peerDependencies");
}

#[test]
fn parses_package_json5_dependency_groups() {
    let text = package_file_fixture("package-json5-dependency-groups.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/package.json5", "json5");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(dependencies[0].requirement, "1.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.0.0"
    );
}

#[test]
fn package_json_ranges_count_utf16_code_units_before_dependencies() {
    let text = package_file_fixture(
        "package-json-ranges-count-utf16-code-units-before-dependencies.json5",
    );
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/package.json", "json");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
    assert_eq!(dependencies[0].range.start.character, 31);
}

#[test]
fn parses_npm_bundle_dependency_name_arrays_by_default() {
    let (text, dependencies) =
        parse_package_fixture("bundle-dependency-name-arrays-by-default.json");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "bundledDependencies");
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
    assert_eq!(dependencies[1].group, "bundleDependencies");
    assert_eq!(dependencies[1].name, "right-pad");
    assert_eq!(dependencies[1].requirement, "");
}

#[test]
fn parses_bun_trusted_dependencies_by_default() {
    let (text, dependencies) = parse_package_fixture("bun-trusted-dependencies-by-default.txt");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "trustedDependencies");
    assert_eq!(dependencies[0].name, "my-trusted-package");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "my-trusted-package"
    );
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
}

#[test]
fn parses_yarn_resolutions_by_default() {
    let (text, dependencies) = parse_package_fixture("yarn-resolutions-by-default.json");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "resolutions");
    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(dependencies[0].requirement, "1.1.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.1.0"
    );
}

#[test]
fn parses_yarn_resolution_selectors_by_terminal_package() {
    let (_, dependencies) =
        parse_package_fixture("yarn-resolution-selectors-by-terminal-package.json");

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].group, "resolutions");
    assert_eq!(dependencies[0].name, "memory-fs");
    assert_eq!(dependencies[0].requirement, "0.4.1");
    assert_eq!(dependencies[1].name, "json5");
    assert_eq!(dependencies[1].requirement, "2.1.0");
    assert_eq!(dependencies[2].name, "@babel/generator");
    assert_eq!(dependencies[2].requirement, "7.3.4");
    assert_eq!(dependencies[3].name, "@babel/generator");
    assert_eq!(dependencies[3].requirement, "7.3.4");
}

#[test]
fn parses_package_manager_prerelease_build_metadata() {
    let (text, dependencies) =
        parse_package_fixture("package-manager-prerelease-build-metadata.txt");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "packageManager");
    assert_eq!(dependencies[0].name, "pnpm");
    assert_eq!(
        dependencies[0].requirement,
        "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
    );
}

#[test]
fn ignores_invalid_package_manager_values() {
    let (_, dependencies) = parse_package_fixture("ignores-invalid-package-manager-values.json");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "left-pad");
}

#[test]
fn parses_dev_engines_package_manager_version() {
    let (text, dependencies) = parse_package_fixture("dev-engines-package-manager-version.json");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "devEngines.packageManager");
    assert_eq!(dependencies[0].name, "npm");
    assert_eq!(dependencies[0].requirement, "^10.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^10.0.0"
    );
}

#[test]
fn parses_npm_overrides_dot_self_override() {
    let (text, dependencies) = parse_package_fixture("overrides-dot-self-override.json");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(dependencies[0].group, "overrides");
    assert_eq!(dependencies[0].name, "foo");
    assert_eq!(dependencies[0].requirement, "1.0.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.0.0"
    );
    assert_eq!(dependencies[1].group, "overrides");
    assert_eq!(dependencies[1].name, "bar");
    assert_eq!(dependencies[1].requirement, "2.0.0");
}

#[test]
fn parses_scoped_npm_override_version_selector_by_package_name() {
    assert_scoped_override_fixture("scoped-npm-override-version-selector-by-package-name.txt");
}

#[test]
fn parses_scoped_npm_override_dot_self_selector_by_package_name() {
    assert_scoped_override_fixture("scoped-npm-override-dot-self-selector-by-package-name.json");
}

fn assert_scoped_override_fixture(name: &str) {
    let text = package_file_fixture(name);
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/package.json", "json");
    let dependency = dependencies.first().expect("scoped override dependency");
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependency.ecosystem, Npm);
    assert_eq!(dependency.group, "overrides");
    assert_eq!(dependency.name, "@scope/pkg");
    assert_eq!(dependency.requirement, "1.2.4");
    assert_eq!(extract_range(text, dependency.requirement_range), "1.2.4");
}

fn parse_package_fixture(name: &str) -> (&'static str, Vec<versionlens_model::Dependency>) {
    let text = package_file_fixture(name);
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/package.json", "json");
    (text, dependencies)
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_basic",
        name,
    )
}
