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
