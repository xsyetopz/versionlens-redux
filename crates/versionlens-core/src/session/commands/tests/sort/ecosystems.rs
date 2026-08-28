#[test]
fn apply_command_sorts_maven_dependency_nodes() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///pom.xml", "xml", "dependencies-unsorted.xml");

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

    let output = sort_fixture(&session, "file:///pom.xml", "xml", "dependency-management-unsorted.xml");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "      <dependency>\n        <groupId>org.alpha</groupId>\n        <type>pom</type>\n        <artifactId>alpha</artifactId>\n        <version>1</version>\n      </dependency>"
    );
    assert_eq!(
        output.edits[1].new_text,
        "      <dependency>\n        <groupId>org.zeta</groupId>\n        <type>pom</type>\n        <artifactId>zeta</artifactId>\n        <version>1</version>\n      </dependency>"
    );
}

#[test]
fn apply_command_sorts_maven_profile_dependency_nodes() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///pom.xml", "xml", "profile-dependencies-unsorted.xml");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(
        output.edits[0].new_text,
        "        <dependency>\n          <groupId>org.alpha</groupId>\n          <scope>runtime</scope>\n          <artifactId>alpha</artifactId>\n          <version>1</version>\n        </dependency>"
    );
    assert_eq!(
        output.edits[1].new_text,
        "        <dependency>\n          <groupId>org.zeta</groupId>\n          <scope>runtime</scope>\n          <artifactId>zeta</artifactId>\n          <version>1</version>\n        </dependency>"
    );
}

#[test]
fn apply_command_sorts_dotnet_package_reference_tags() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///app.csproj", "xml", "app-package-references-unsorted.csproj");

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

    let output = sort_fixture(&session, "file:///app.csproj", "xml", "app-package-references-item-groups.csproj");

    assert_sort_output(&output, &[]);
}

#[test]
fn apply_command_sorts_go_require_block_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///go.mod", "go.mod", "go-require-unsorted.mod");

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

    let output = sort_fixture(&session, "file:///go.mod", "go.mod", "go-exclude-unsorted.mod");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "\talpha.example/pkg v1.0.0");
    assert_eq!(output.edits[1].new_text, "\tzeta.example/pkg v1.0.0");
}

#[test]
fn apply_command_sorts_ruby_gemfile_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///Gemfile", "ruby", "Gemfile-unsorted");

    assert!(output.suggestions.is_empty());
    assert_eq!(output.edits.len(), 2);
    assert_eq!(output.edits[0].new_text, "gem \"alpha\", \"1.0.0\"");
    assert_eq!(output.edits[1].new_text, "gem \"zeta\", \"1.0.0\"");
}

#[test]
fn apply_command_does_not_sort_gleam_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///gleam.toml", "toml", "gleam-unsorted.toml");

    assert_sort_output(&output, &[]);
}

#[test]
fn apply_command_does_not_sort_gradle_build_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///build.gradle.kts", "kotlin", "build.gradle.kts");

    assert_sort_output(&output, &[]);
}

#[test]
fn apply_command_does_not_sort_sbt_library_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///build.sbt", "scala", "build.sbt");

    assert_sort_output(&output, &[]);
}
