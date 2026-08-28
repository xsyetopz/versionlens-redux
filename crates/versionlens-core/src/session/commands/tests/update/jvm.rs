#[test]
fn apply_command_updates_dotnet_package_reference_child_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///app.csproj".to_owned(), "xml".to_owned(), package_file_fixture(
                "command-updates-dotnet-package-reference-child-version.csproj",
            ), None),
        command: Some("update"),
        dependency_name: Some("Microsoft.NET.Test.Sdk"),
        selected_version: Some("18.8.0"),
        responses: &[RegistryResponseInput::new("Microsoft.NET.Test.Sdk".to_owned(), Dotnet, r#"{"versions":["18.7.0","18.8.0"]}"#.to_owned())],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.requirement, "18.7.0");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "18.8.0");
}

#[test]
fn apply_command_updates_gradle_groovy_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.gradle".to_owned(), "groovy".to_owned(), package_file_fixture("command-updates-gradle-groovy-dependency-version.gradle"), None),
        command: Some("update"),
        dependency_name: Some("org.springframework:spring-core"),
        selected_version: Some("6.2.9"),
        responses: &[RegistryResponseInput::new("org.springframework:spring-core".to_owned(), Maven, r#"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "implementation", "6.2.8", "6.2.9");
}

#[test]
fn apply_command_updates_gradle_kotlin_named_argument_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.gradle.kts".to_owned(), "kotlin".to_owned(), package_file_fixture("command-updates-gradle-kotlin-named-argument-dependency-version.gradle.kts"), None),
        command: Some("update"),
        dependency_name: Some("org.slf4j:slf4j-api"),
        selected_version: Some("2.0.18"),
        responses: &[RegistryResponseInput::new("org.slf4j:slf4j-api".to_owned(), Maven, r#"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "implementation", "2.0.17", "2.0.18");
}

#[test]
fn apply_command_updates_gradle_plugin_dsl_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///settings.gradle.kts".to_owned(), "kotlin".to_owned(), package_file_fixture("command-updates-gradle-plugin-dsl-version.gradle.kts"), None),
        command: Some("update"),
        dependency_name: Some("com.android.application:com.android.application.gradle.plugin"),
        selected_version: Some("8.12.1"),
        responses: &[RegistryResponseInput::new("com.android.application:com.android.application.gradle.plugin".to_owned(), Maven, r#"<metadata><versioning><versions><version>8.12.0</version><version>8.12.1</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "plugins", "8.12.0", "8.12.1");
}

#[test]
fn apply_command_updates_gradle_version_catalog_library_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///gradle/libs.versions.toml".to_owned(), "toml".to_owned(), package_file_fixture("command-updates-gradle-version-catalog-library-version.versions.toml"), None),
        command: Some("update"),
        dependency_name: Some("org.apache.commons:commons-lang3"),
        selected_version: Some("3.18.0"),
        responses: &[RegistryResponseInput::new("org.apache.commons:commons-lang3".to_owned(), Maven, r#"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "libraries", "3.17.0", "3.18.0");
}

#[test]
fn apply_command_updates_gradle_version_catalog_plugin_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///gradle/libs.versions.toml".to_owned(), "toml".to_owned(), package_file_fixture("command-updates-gradle-version-catalog-plugin-version.versions.toml"), None),
        command: Some("update"),
        dependency_name: Some("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"),
        selected_version: Some("0.52.0"),
        responses: &[RegistryResponseInput::new("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(), Maven, r#"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "plugins", "0.51.0", "0.52.0");
}

#[test]
fn apply_command_updates_sbt_library_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-updates-sbt-library-dependency-version.sbt"), None),
        command: Some("update"),
        dependency_name: Some("org.scala-stm:scala-stm_2.13"),
        selected_version: Some("0.9.2"),
        responses: &[RegistryResponseInput::new("org.scala-stm:scala-stm_2.13".to_owned(), Maven, r#"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].dependency.group,
        "libraryDependencies"
    );
    assert_eq!(output.suggestions[0].dependency.requirement, "0.9.1");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.9.2");
}

#[test]
fn apply_command_updates_sbt_dependency_override_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-updates-sbt-dependency-override-version.sbt"), None),
        command: Some("update"),
        dependency_name: Some("log4j:log4j"),
        selected_version: Some("1.2.17"),
        responses: &[RegistryResponseInput::new("log4j:log4j".to_owned(), Maven, r#"<metadata><versioning><versions><version>1.2.16</version><version>1.2.17</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].dependency.group,
        "dependencyOverrides"
    );
    assert_eq!(output.suggestions[0].dependency.requirement, "1.2.16");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.17");
}

#[test]
fn apply_command_does_not_update_sbt_url_artifact_dependency() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///build.sbt".to_owned(), "scala".to_owned(), package_file_fixture("command-does-not-update-sbt-url-artifact-dependency.sbt"), None),
        command: Some("update"),
        dependency_name: Some("slinky:slinky"),
        selected_version: Some("2.2"),
        responses: &[RegistryResponseInput::new("slinky:slinky".to_owned(), Maven, r#"<metadata><versioning><versions><version>2.1</version><version>2.2</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_eq!(output.suggestions.len(), 1);
    crate::support::tests::assert_suggestion_without_edits(&output, 0, "fixed", Some("package URL"));
}

#[test]
fn apply_command_updates_clojure_deps_edn_maven_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///deps.edn".to_owned(), "clojure".to_owned(), package_file_fixture("command-updates-clojure-deps-edn-maven-version.edn"), None),
        command: Some("update"),
        dependency_name: Some("org.clojure:tools.reader"),
        selected_version: Some("1.2.0"),
        responses: &[RegistryResponseInput::new("org.clojure:tools.reader".to_owned(), Maven, r#"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "deps", "1.1.1", "1.2.0");
}

#[test]
fn apply_command_updates_leiningen_project_clj_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///project.clj".to_owned(), "clojure".to_owned(), package_file_fixture("command-updates-leiningen-project-clj-dependency-version.clj"), None),
        command: Some("update"),
        dependency_name: Some("org.clojure:clojure"),
        selected_version: Some("1.12.0"),
        responses: &[RegistryResponseInput::new("org.clojure:clojure".to_owned(), Maven, r#"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>"#.to_owned())],
    });

    assert_single_dependency_update(&output, "dependencies", "1.11.3", "1.12.0");
}

#[test]
fn apply_command_updates_packages_config_version_attribute() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///packages.config".to_owned(), "xml".to_owned(), package_file_fixture(
                "command-updates-packages-config-version-attribute.config",
            ), None),
        command: Some("update"),
        dependency_name: Some("jQuery"),
        selected_version: Some("3.7.1"),
        responses: &[RegistryResponseInput::new("jQuery".to_owned(), Dotnet, r#"{"versions":["3.1.1","3.7.1"]}"#.to_owned())],
    });

    assert_single_dependency_update(&output, "packages.package", "3.1.1", "3.7.1");
}
