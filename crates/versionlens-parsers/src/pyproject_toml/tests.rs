use crate::document::test_support::{assert_parenthesized_pep508, extract_range, parse_fixture};
use crate::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use versionlens_model::Ecosystem::Python;

#[test]
fn parses_pipfile_dependencies() {
    let text = package_file_fixture("parses-pipfile-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/Pipfile", "toml");

    assert_eq!(dependencies.len(), 3);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0, Python, "packages", "requests", "==2.32",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "==2.32"
    );
    assert_eq!(dependencies[1].name, "local");
    assert_eq!(dependencies[1].requirement, "../local");
    assert_eq!(dependencies[2].group, "dev-packages");
    assert_eq!(dependencies[2].name, "pytest");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "table-driven manifest coverage stays readable as one scenario"
)]
fn parses_pyproject_toml_dependencies() {
    let text = package_file_fixture("parses-pyproject-toml-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/pyproject.toml", "toml");

    assert_eq!(dependencies.len(), 15);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Python, "project", "version", "1.2.3"),
    );
    assert_eq!(dependencies[1].group, "project.dependencies");
    assert_eq!(dependencies[1].name, "httpx");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, "==");
    assert_eq!(dependencies[2].name, "django");
    assert_eq!(dependencies[2].requirement, ">2.1");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        ">2.1"
    );
    assert_eq!(dependencies[3].group, "project.optional-dependencies.test");
    assert_eq!(dependencies[3].name, "pytest");
    assert_eq!(dependencies[4].group, "project.optional-dependencies.test");
    assert_eq!(dependencies[4].name, "pytest-cov");
    assert_eq!(dependencies[4].requirement, "<=7.1.0");
    assert_eq!(
        extract_range(text, dependencies[4].requirement_range),
        "<=7.1.0"
    );
    assert_eq!(dependencies[5].group, "dependency-groups.dev");
    assert_eq!(dependencies[5].name, "mypy");
    assert_eq!(dependencies[5].requirement, ">=1.16");
    assert_eq!(dependencies[6].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[6].name, "python");
    assert_eq!(dependencies[6].requirement, "^3.12");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "^3.12"
    );
    assert_eq!(dependencies[7].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[7].name, "requests");
    assert_eq!(dependencies[7].requirement, "^2.32");
    assert_eq!(
        extract_range(text, dependencies[7].requirement_range),
        "^2.32"
    );
    assert_eq!(dependencies[8].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[8].name, "local");
    assert_eq!(dependencies[8].requirement, "../local");
    assert_eq!(dependencies[9].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[9].name, "private");
    assert_eq!(dependencies[9].hosted_url, Some("private".to_owned()));
    assert_eq!(dependencies[10].group, "tool.poetry.dependencies.httpx");
    assert_eq!(dependencies[10].name, "httpx");
    assert_eq!(dependencies[10].requirement, "^0.28");
    assert_eq!(dependencies[11].group, "tool.poetry.group.dev.dependencies");
    assert_eq!(dependencies[11].name, "ruff");
    assert_eq!(
        dependencies[12].group,
        "tool.poetry.group.dev.dependencies.pytest"
    );
    assert_eq!(dependencies[12].name, "pytest");
    assert_eq!(dependencies[12].requirement, "^8");
    assert_eq!(dependencies[13].group, "tool.uv.sources");
    assert_eq!(dependencies[13].name, "local");
    assert_eq!(dependencies[13].requirement, "../local");
    assert_eq!(dependencies[14].group, "tool.uv.sources");
    assert_eq!(dependencies[14].name, "remote");
    assert_eq!(
        dependencies[14].requirement,
        "https://example.test/repo.git"
    );
}

#[test]
fn configured_project_table_does_not_match_optional_dependencies_table() {
    let text = package_file_fixture(
        "configured-project-table-does-not-match-optional-dependencies-table.txt",
    );
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput::new(
            "file:///work/pyproject.toml".to_owned(),
            "toml".to_owned(),
            text.to_owned(),
            None,
        ),
        &["project"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "project.dependencies");
    assert_eq!(dependencies[0].name, "httpx");
    assert_eq!(dependencies[0].requirement, "==0.28.1");
}

#[test]
fn parses_poetry_python_version_dependency() {
    let text = package_file_fixture("parses-poetry-python-version-dependency.toml");
    let dependencies = parse_fixture(text, "file:///work/pyproject.toml", "toml");

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Python,
            "tool.poetry.dependencies",
            "python",
            "^3.12",
        ),
    );
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "^3.12"
    );
    assert_eq!(
        dependencies[1].group,
        "tool.poetry.group.dev.dependencies.python"
    );
    assert_eq!(dependencies[1].name, "python");
    assert_eq!(dependencies[1].requirement, "^3.13");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "^3.13"
    );
}

#[test]
fn parses_dependency_groups_and_uv_sources_by_default() {
    let text = package_file_fixture("parses-dependency-groups-and-uv-sources-by-default.txt");
    let dependencies = parse_fixture(text, "file:///work/pyproject.toml", "toml");
    assert_dependency_groups_and_sources(&dependencies);
}

#[test]
fn parses_configured_dependency_groups_and_uv_sources() {
    let text = package_file_fixture("parses-configured-dependency-groups-and-uv-sources.txt");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput::new(
            "file:///work/pyproject.toml".to_owned(),
            "toml".to_owned(),
            text.to_owned(),
            None,
        ),
        &["dependency-groups", "tool.uv.sources"],
    );

    assert_dependency_groups_and_sources(&dependencies);
}

fn assert_dependency_groups_and_sources(dependencies: &[versionlens_model::Dependency]) {
    assert_eq!(dependencies.len(), 3);
    for (dependency, (group, name, requirement)) in dependencies.iter().zip([
        ("dependency-groups.dev", "mypy", ">=1.16"),
        ("tool.uv.sources", "local", "../local"),
        ("tool.uv.sources", "remote", "https://example.test/repo.git"),
    ]) {
        assert_eq!(dependency.group, group);
        assert_eq!(dependency.name, name);
        assert_eq!(dependency.requirement, requirement);
    }
}

#[test]
fn parses_legacy_poetry_dev_dependencies_by_default() {
    let text = package_file_fixture("parses-legacy-poetry-dev-dependencies-by-default.txt");
    let dependencies = parse_fixture(text, "file:///work/pyproject.toml", "toml");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "tool.poetry.dev-dependencies");
    assert_eq!(dependencies[0].name, "pytest");
    assert_eq!(dependencies[0].requirement, "^8");
    assert_eq!(dependencies[1].group, "tool.poetry.dev-dependencies.mypy");
    assert_eq!(dependencies[1].name, "mypy");
    assert_eq!(dependencies[1].requirement, "^1.16");
}

#[test]
fn parses_configured_poetry_dev_dependencies() {
    let text = package_file_fixture("parses-configured-poetry-dev-dependencies.toml");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput::new(
            "file:///work/pyproject.toml".to_owned(),
            "toml".to_owned(),
            text.to_owned(),
            None,
        ),
        &["tool.poetry.dev-dependencies".to_owned()],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "tool.poetry.dev-dependencies");
    assert_eq!(dependencies[0].name, "pytest");
    assert_eq!(dependencies[0].requirement, "^8");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "^8");
    assert_eq!(dependencies[1].name, "ruff");
    assert_eq!(dependencies[1].requirement, "0.9");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "0.9"
    );
    assert_eq!(dependencies[2].group, "tool.poetry.dev-dependencies.mypy");
    assert_eq!(dependencies[2].name, "mypy");
    assert_eq!(dependencies[2].requirement, "^1.16");
    assert_eq!(dependencies[2].hosted_url, None);
}

#[test]
fn parses_parenthesized_pep_508_requirements_without_polluting_package_names() {
    let text = r#"[project]
dependencies = ["unfat (>=0.0.13)", "httpx[http2] (>=0.28,<1); python_version >= '3.11'"]
"#;
    let dependencies = parse_fixture(text, "file:///work/pyproject.toml", "toml");

    assert_parenthesized_pep508(text, &dependencies);
}

#[test]
fn parses_smoke_python_smoke_shapes() {
    let pyproject = r#"
[project]
name = "test"
dependencies = [
  "httpx==0.28.1",
  "gidgethub[httpx]>4.0.0",
  "django>=6.0.6; os_name != 'nt'",
  "django>=6.0.6; os_name == 'nt'",
  "uvicorn[standard] >=0.49.0",
  "magic",
]

[project.optional-dependencies]
test = ["pytest==9.1.1", "pytest-cov[all]==7.1.0", "pytest-cov[all]<=7.1.0"]

[tool.poetry.dependencies]
mysqlclient = "2.2.8"

[tool.poetry.group.dev.dependencies]
pip = { version = "26.1.2", source = "private" }
my-package = { path = ".." }
"#;
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/pyproject.toml".to_owned(),
        "toml".to_owned(),
        pyproject.to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].name, "httpx");
    assert_eq!(dependencies[1].name, "gidgethub");
    assert_eq!(dependencies[1].requirement, ">4.0.0");
    assert_eq!(dependencies[2].name, "django");
    assert_eq!(dependencies[3].name, "django");
    assert_eq!(dependencies[4].name, "uvicorn");
    assert_eq!(dependencies[4].requirement, ">=0.49.0");
    assert_eq!(
        extract_range(pyproject, dependencies[4].requirement_range),
        ">=0.49.0"
    );
    assert_eq!(dependencies[5].name, "magic");
    assert_eq!(dependencies[5].requirement_prefix, "==");
    assert_eq!(dependencies[8].name, "pytest-cov");
    assert_eq!(dependencies[8].requirement, "<=7.1.0");
    assert_eq!(dependencies[11].name, "my-package");
    assert_eq!(dependencies[11].requirement, "..");

    let pipfile = r#"
[project]
version = "1.2.3"
description = "smoke test"

[packages]
Sphinx = "7.3.0"

[dev-packages]
pip = { version = "24.0", source = "private" }
my_script = "0.1.0"
magic = ""
"#;
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/Pipfile".to_owned(),
        "toml".to_owned(),
        pipfile.to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[1].name, "Sphinx");
    assert_eq!(dependencies[2].name, "pip");
    assert_eq!(dependencies[2].requirement, "24.0");
    assert_eq!(dependencies[3].name, "my_script");
    assert_eq!(dependencies[4].name, "magic");
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, "==");
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture(
        "tests/fixtures/versionlens-parsers/src/pyproject_toml/tests",
        name,
    )
}
