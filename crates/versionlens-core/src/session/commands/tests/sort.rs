use std::fs::read_to_string;
use std::path::PathBuf;

use super::{DocumentInput, session_with_dependency_properties, standard_session};
use versionlens_parsers::Ecosystem::{Deno, Maven, Npm};
use versionlens_vscode_model::TextEdit;

#[test]
fn apply_command_sorts_requirements_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture("requirements-unsorted-with-comment.txt"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "alpha==1");
    assert_eq!(output.edits[1].new_text, "zeta==1");
}

#[test]
fn apply_command_sorts_smoke_requirements_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("requirements-smoke.txt");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "# Requirements for smoke testing\ndjango<=3.2\nflask>=2.0\nnot_found_package==1.17.0\nnumpy<1.22 # this should not cause issues\npandas~=1.2\npytest>3.0\npython-dateutil\nrequests==2.25.1\nsix==1.17.0\nurllib3===1.26.5"
    );
}

#[test]
fn apply_command_sorts_pyproject_project_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("pyproject-project-unsorted.toml");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "[project]\ndependencies = [\n  \"alpha==1\",\n  \"zeta==1\"\n]"
    );
}

#[test]
fn apply_command_sorts_pyproject_poetry_dependencies() {
    let session = standard_session();

    let text = package_file_fixture("pyproject-poetry-unsorted.toml");
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.clone(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(&text, &output.edits),
        "[tool.poetry.dependencies]\nalpha = \"1\"\nzeta = \"1\""
    );
}

#[test]
fn apply_command_sorts_pub_dependencies_by_group() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-groups-unsorted.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(output.edits[0].new_text, "  alpha: 1");
    assert_eq!(output.edits[1].new_text, "  zeta: 1");
    assert_eq!(output.edits[2].new_text, "  a-dev: 1");
    assert_eq!(output.edits[3].new_text, "  z-dev: 1");
}

#[test]
fn apply_command_sorts_pub_dependencies_with_blank_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-blank-version.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "  equatable:");
    assert_eq!(output.edits[1].new_text, "  flutter_bloc: 0.10.1");
}

#[test]
fn apply_command_sorts_complex_pub_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pubspec-complex.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "  equatable: ^0.2.0");
    assert_eq!(
        output.edits[1].new_text,
        "  sqflite:\n    git:\n      url: https://github.com/tekartik/sqflite\n      path: sqflite"
    );
}

#[test]
fn apply_command_sorts_package_json_dependencies_by_group() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-groups-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
    assert_eq!(output.edits[2].new_text, "    \"a-dev\": \"1\",");
    assert_eq!(output.edits[3].new_text, "    \"z-dev\": \"1\"");
}

#[test]
fn apply_command_sorts_package_json_dependencies_with_metadata_entries() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-metadata-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    \"alpha\": \"1\",");
    assert_eq!(output.edits[1].new_text, "    \"zeta\": \"1\"");
}

#[test]
fn apply_command_does_not_sort_docker_compose_images() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///docker-compose.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("docker-compose-images.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_sorts_composer_require_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("composer-require-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "    \"allocine/twigcs\": \"^3.1.3\","
    );
    assert_eq!(
        output.edits[1].new_text,
        "    \"symfony/console\": \"8.1.*\""
    );
}

#[test]
fn apply_command_sorts_deno_scoped_imports_within_each_scope() {
    let session = session_with_dependency_properties(Deno, &["scopes"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("deno-scopes-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 4);
    assert_eq!(
        output.edits[0].new_text,
        "      \"chalk\": \"npm:chalk@5.3.0\","
    );
    assert_eq!(
        output.edits[1].new_text,
        "      \"zeta\": \"npm:zeta@1.0.0\""
    );
    assert_eq!(
        output.edits[2].new_text,
        "      \"alpha\": \"jsr:@scope/alpha@1.0.0\","
    );
    assert_eq!(
        output.edits[3].new_text,
        "      \"bravo\": \"jsr:@scope/bravo@1.0.0\""
    );
}

#[test]
fn apply_command_sorts_pnpm_named_catalog_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pnpm-workspace.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("pnpm-workspace-named-catalog-unsorted.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "    react: ^18.3.1");
    assert_eq!(output.edits[1].new_text, "    react-dom: ^19.2.7");
}

#[test]
fn apply_command_sorts_package_json_named_workspace_catalog_dependencies() {
    let session = session_with_dependency_properties(Npm, &["workspaces.catalogs.*"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("package-workspace-catalog-unsorted.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "        \"react\": \"^18.3.1\",");
    assert_eq!(
        output.edits[1].new_text,
        "        \"react-dom\": \"^19.2.7\""
    );
}

#[test]
fn apply_command_sorts_maven_dependency_nodes() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("pom-dependencies-unsorted.xml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>"
    );
    assert_eq!(
        output.edits[1].new_text,
        "    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>"
    );
}

#[test]
fn apply_command_sorts_configured_maven_dependency_management_nodes() {
    let session = session_with_dependency_properties(
        Maven,
        &["project.dependencyManagement.dependencies.dependency"],
    );

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("pom-dependency-management-unsorted.xml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "      <dependency>\n        <groupId>org.alpha</groupId>\n        <artifactId>alpha</artifactId>\n        <version>1</version>\n      </dependency>"
    );
    assert_eq!(
        output.edits[1].new_text,
        "      <dependency>\n        <groupId>org.zeta</groupId>\n        <artifactId>zeta</artifactId>\n        <version>1</version>\n      </dependency>"
    );
}

#[test]
fn apply_command_sorts_maven_profile_dependency_nodes() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("pom-profile-dependencies-unsorted.xml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "        <dependency>\n          <groupId>org.alpha</groupId>\n          <artifactId>alpha</artifactId>\n          <version>1</version>\n        </dependency>"
    );
    assert_eq!(
        output.edits[1].new_text,
        "        <dependency>\n          <groupId>org.zeta</groupId>\n          <artifactId>zeta</artifactId>\n          <version>1</version>\n        </dependency>"
    );
}

#[test]
fn apply_command_sorts_dotnet_package_reference_tags() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("app-package-references-unsorted.csproj"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "    <PackageReference Include=\"Alpha.Package\" Version=\"1\" />"
    );
    assert_eq!(
        output.edits[1].new_text,
        "    <PackageReference Include=\"Zeta.Package\" Version=\"1\" />"
    );
}

#[test]
fn apply_command_does_not_sort_dotnet_package_references_across_item_groups() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("app-package-references-item-groups.csproj"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_sorts_go_require_block_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-require-unsorted.mod"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "\talpha.example/pkg v1.0.0 // indirect"
    );
    assert_eq!(output.edits[1].new_text, "\tzeta.example/pkg v1.0.0");
}

#[test]
fn apply_command_sorts_go_exclude_block_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-exclude-unsorted.mod"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "\talpha.example/pkg v1.0.0");
    assert_eq!(output.edits[1].new_text, "\tzeta.example/pkg v1.0.0");
}

#[test]
fn apply_command_sorts_ruby_gemfile_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("Gemfile-unsorted"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
    assert_eq!(output.edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
}

#[test]
fn apply_command_does_not_sort_gleam_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///gleam.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gleam-unsorted.toml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_gradle_build_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("build.gradle.kts"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_sbt_library_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("build.sbt"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/commands/sort")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session command sort fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

include!("sort_more.rs");
