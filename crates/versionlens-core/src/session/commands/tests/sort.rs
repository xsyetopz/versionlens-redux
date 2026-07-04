use super::{DocumentInput, Ecosystem, session_with_dependency_properties, standard_session};
use versionlens_vscode_model::TextEdit;

#[test]
fn apply_command_sorts_requirements_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: "zeta==1\n# keep\nalpha==1\n".to_owned(),
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

    let text = "# Requirements for smoke testing\nrequests==2.25.1\nflask>=2.0\ndjango<=3.2\npytest>3.0\nnumpy<1.22 # this should not cause issues\npandas~=1.2\nurllib3===1.26.5\nsix==1.17.0\npython-dateutil\nnot_found_package==1.17.0\n";
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(
        apply_line_edits(text, &output.edits),
        "# Requirements for smoke testing\ndjango<=3.2\nflask>=2.0\nnot_found_package==1.17.0\nnumpy<1.22 # this should not cause issues\npandas~=1.2\npytest>3.0\npython-dateutil\nrequests==2.25.1\nsix==1.17.0\nurllib3===1.26.5"
    );
}

#[test]
fn apply_command_sorts_pyproject_project_dependencies() {
    let session = standard_session();

    let text = "[project]\ndependencies = [\n  \"zeta==1\",\n  \"alpha==1\"\n]\n";
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(text, &output.edits),
        "[project]\ndependencies = [\n  \"alpha==1\",\n  \"zeta==1\"\n]"
    );
}

#[test]
fn apply_command_sorts_pyproject_poetry_dependencies() {
    let session = standard_session();

    let text = "[tool.poetry.dependencies]\nzeta = \"1\"\nalpha = \"1\"\n";
    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pyproject.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        apply_line_edits(text, &output.edits),
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
            text:
                "dependencies:\n  zeta: 1\n  alpha: 1\ndev_dependencies:\n  z-dev: 1\n  a-dev: 1\n"
                    .to_owned(),
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
            text: "dependencies:\n  flutter_bloc: 0.10.1\n  equatable:\n".to_owned(),
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
            text: "dependencies:\n  sqflite:\n    git:\n      url: https://github.com/tekartik/sqflite\n      path: sqflite\n  equatable: ^0.2.0\n"
                .to_owned(),
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
            text: "{\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  },\n  \"devDependencies\": {\n    \"z-dev\": \"1\",\n    \"a-dev\": \"1\"\n  }\n}"
                .to_owned(),
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
            text: "{\n  \"version\": \"1.0.0\",\n  \"packageManager\": \"pnpm@9.0.0\",\n  \"dependencies\": {\n    \"zeta\": \"1\",\n    \"alpha\": \"1\"\n  }\n}"
                .to_owned(),
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
fn apply_command_sorts_composer_require_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: "{\n  \"version\": \"1.0.0\",\n  \"require\": {\n    \"symfony/console\": \"8.1.*\",\n    \"allocine/twigcs\": \"^3.1.3\"\n  }\n}"
                .to_owned(),
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
    let session = session_with_dependency_properties(Ecosystem::Deno, &["scopes"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: "{\n  \"scopes\": {\n    \"https://deno.land/x/app/\": {\n      \"zeta\": \"npm:zeta@1.0.0\",\n      \"chalk\": \"npm:chalk@5.3.0\"\n    },\n    \"https://deno.land/x/other/\": {\n      \"bravo\": \"jsr:@scope/bravo@1.0.0\",\n      \"alpha\": \"jsr:@scope/alpha@1.0.0\"\n    }\n  }\n}"
                .to_owned(),
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
            text: "catalogs:\n  react18:\n    react-dom: ^19.2.7\n    react: ^18.3.1\n".to_owned(),
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
    let session = session_with_dependency_properties(Ecosystem::Npm, &["workspaces.catalogs.*"]);

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: "{\n  \"workspaces\": {\n    \"catalogs\": {\n      \"react18\": {\n        \"react-dom\": \"^19.2.7\",\n        \"react\": \"^18.3.1\"\n      }\n    }\n  }\n}"
                .to_owned(),
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
            text: "<project>\n  <dependencies>\n    <dependency>\n      <groupId>org.zeta</groupId>\n      <artifactId>zeta</artifactId>\n      <version>1</version>\n    </dependency>\n    <dependency>\n      <groupId>org.alpha</groupId>\n      <artifactId>alpha</artifactId>\n      <version>1</version>\n    </dependency>\n  </dependencies>\n</project>"
                .to_owned(),
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
        Ecosystem::Maven,
        &["project.dependencyManagement.dependencies.dependency"],
    );

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: "<project>\n  <dependencyManagement>\n    <dependencies>\n      <dependency>\n        <groupId>org.zeta</groupId>\n        <artifactId>zeta</artifactId>\n        <version>1</version>\n      </dependency>\n      <dependency>\n        <groupId>org.alpha</groupId>\n        <artifactId>alpha</artifactId>\n        <version>1</version>\n      </dependency>\n    </dependencies>\n  </dependencyManagement>\n</project>"
                .to_owned(),
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
fn apply_command_sorts_dotnet_package_reference_tags() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: "<Project>\n  <ItemGroup>\n    <PackageReference Include=\"Zeta.Package\" Version=\"1\" />\n    <PackageReference Include=\"Alpha.Package\" Version=\"1\" />\n  </ItemGroup>\n</Project>"
                .to_owned(),
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
fn apply_command_sorts_go_require_block_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: "module example.test/app\n\nrequire (\n\tzeta.example/pkg v1.0.0\n\talpha.example/pkg v1.0.0 // indirect\n)\n"
                .to_owned(),
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
fn apply_command_sorts_ruby_gemfile_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Gemfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: "gem \"zeta\", \"1.0.0\"\ngem \"alpha\", \"1.0.0\"\n".to_owned(),
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

fn apply_line_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| std::cmp::Reverse(edit.range.start.line));
    for edit in ordered {
        let start = usize::try_from(edit.range.start.line).unwrap();
        let end = usize::try_from(edit.range.end.line).unwrap();
        lines.splice(start..=end, edit.new_text.lines().map(str::to_owned));
    }
    lines.join("\n")
}
