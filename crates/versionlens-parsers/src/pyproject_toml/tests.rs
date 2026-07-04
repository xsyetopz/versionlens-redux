use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document, parse_document_with_dependency_paths};

#[test]
fn parses_pipfile_dependencies() {
    let text = r#"
[packages]
requests = "==2.32"
local = { path = "../local" }

[dev-packages]
pytest = ">=8"
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Pipfile".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Python);
    assert_eq!(dependencies[0].group, "packages");
    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(dependencies[0].requirement, "==2.32");
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
fn parses_pyproject_toml_dependencies() {
    let text = r#"
[project]
version = "1.2.3"
dependencies = [
  "httpx",
  "django>2.1"
]

[project.optional-dependencies]
test = ["pytest>=8", "pytest-cov[all]<=7.1.0"]

[dependency-groups]
dev = ["mypy>=1.16"]

[tool.poetry.dependencies]
python = "^3.12"
requests = "^2.32"
local = { path = "../local" }
private = { version = "^1.0", source = "private" }

[[tool.poetry.source]]
name = "private"
url = "https://poetry.example.test/simple/"

[tool.poetry.dependencies.httpx]
version = "^0.28"

[tool.poetry.group.dev.dependencies]
ruff = "0.9"

[tool.poetry.group.dev.dependencies.pytest]
version = "^8"

[tool.uv.sources]
local = { path = "../local" }
remote = { git = "https://example.test/repo.git" }
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Python);
    assert_eq!(dependencies[0].group, "project");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.2.3");
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
    assert_eq!(dependencies[5].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[5].name, "python");
    assert_eq!(dependencies[5].requirement, "^3.12");
    assert_eq!(
        extract_range(text, dependencies[5].requirement_range),
        "^3.12"
    );
    assert_eq!(dependencies[6].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[6].name, "requests");
    assert_eq!(dependencies[6].requirement, "^2.32");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "^2.32"
    );
    assert_eq!(dependencies[7].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[7].name, "local");
    assert_eq!(dependencies[7].requirement, "../local");
    assert_eq!(dependencies[8].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[8].name, "private");
    assert_eq!(dependencies[8].hosted_url, None);
    assert_eq!(dependencies[9].group, "tool.poetry.dependencies.httpx");
    assert_eq!(dependencies[9].name, "httpx");
    assert_eq!(dependencies[9].requirement, "^0.28");
    assert_eq!(dependencies[10].group, "tool.poetry.group.dev.dependencies");
    assert_eq!(dependencies[10].name, "ruff");
    assert_eq!(
        dependencies[11].group,
        "tool.poetry.group.dev.dependencies.pytest"
    );
    assert_eq!(dependencies[11].name, "pytest");
    assert_eq!(dependencies[11].requirement, "^8");
}

#[test]
fn configured_project_table_does_not_match_optional_dependencies_table() {
    let text = r#"
[project]
dependencies = ["httpx==0.28.1"]

[project.optional-dependencies]
test = ["pytest>=8"]
"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "project.dependencies");
    assert_eq!(dependencies[0].name, "httpx");
    assert_eq!(dependencies[0].requirement, "==0.28.1");
}

#[test]
fn parses_poetry_python_version_dependency() {
    let text = r#"
[tool.poetry.dependencies]
python = "^3.12"

[tool.poetry.group.dev.dependencies.python]
version = "^3.13"
"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Python);
    assert_eq!(dependencies[0].group, "tool.poetry.dependencies");
    assert_eq!(dependencies[0].name, "python");
    assert_eq!(dependencies[0].requirement, "^3.12");
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
fn parses_configured_dependency_groups_and_uv_sources() {
    let text = r#"
[dependency-groups]
dev = ["mypy>=1.16"]

[tool.uv.sources]
local = { path = "../local" }
remote = { git = "https://example.test/repo.git" }
"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["dependency-groups", "tool.uv.sources"],
    );

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].group, "dependency-groups.dev");
    assert_eq!(dependencies[0].name, "mypy");
    assert_eq!(dependencies[0].requirement, ">=1.16");
    assert_eq!(dependencies[1].group, "tool.uv.sources");
    assert_eq!(dependencies[1].name, "local");
    assert_eq!(dependencies[1].requirement, "../local");
    assert_eq!(dependencies[2].name, "remote");
    assert_eq!(dependencies[2].requirement, "https://example.test/repo.git");
}

#[test]
fn parses_configured_poetry_dev_dependencies() {
    let text = r#"
[tool.poetry.dev-dependencies]
pytest = "^8"
ruff = { version = "0.9", source = "private" }

[tool.poetry.dev-dependencies.mypy]
version = "^1.16"
source = "private"

[[tool.poetry.source]]
name = "private"
url = "https://poetry.example.test/simple/"
"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: pyproject.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Pipfile".to_owned(),
        language_id: "toml".to_owned(),
        text: pipfile.to_owned(),
        workspace_root: None,
    });

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
