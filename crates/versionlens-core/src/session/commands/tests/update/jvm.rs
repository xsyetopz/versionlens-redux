#[test]
fn apply_command_updates_dotnet_package_reference_child_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-dotnet-package-reference-child-version.csproj",
            ),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("Microsoft.NET.Test.Sdk"),
        selected_version: Some("18.8.0"),
        responses: &[RegistryResponseInput {
            package: "Microsoft.NET.Test.Sdk".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["18.7.0","18.8.0"]}"#.to_owned(),
        }],
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
        input: DocumentInput {
            uri: "file:///build.gradle".to_owned(),
            language_id: "groovy".to_owned(),
            text: package_file_fixture("apply-command-updates-gradle-groovy-dependency-version.gradle"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.springframework:spring-core"),
        selected_version: Some("6.2.9"),
        responses: &[RegistryResponseInput {
            package: "org.springframework:spring-core".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "implementation");
    assert_eq!(output.suggestions[0].dependency.requirement, "6.2.8");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "6.2.9");
}

#[test]
fn apply_command_updates_gradle_kotlin_named_argument_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("apply-command-updates-gradle-kotlin-named-argument-dependency-version.gradle.kts"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.slf4j:slf4j-api"),
        selected_version: Some("2.0.18"),
        responses: &[RegistryResponseInput {
            package: "org.slf4j:slf4j-api".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "implementation");
    assert_eq!(output.suggestions[0].dependency.requirement, "2.0.17");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.18");
}

#[test]
fn apply_command_updates_gradle_plugin_dsl_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///settings.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("apply-command-updates-gradle-plugin-dsl-version.gradle.kts"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("com.android.application:com.android.application.gradle.plugin"),
        selected_version: Some("8.12.1"),
        responses: &[RegistryResponseInput {
            package: "com.android.application:com.android.application.gradle.plugin".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>8.12.0</version><version>8.12.1</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "plugins");
    assert_eq!(output.suggestions[0].dependency.requirement, "8.12.0");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "8.12.1");
}

#[test]
fn apply_command_updates_gradle_version_catalog_library_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-gradle-version-catalog-library-version.versions.toml"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.apache.commons:commons-lang3"),
        selected_version: Some("3.18.0"),
        responses: &[RegistryResponseInput {
            package: "org.apache.commons:commons-lang3".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "libraries");
    assert_eq!(output.suggestions[0].dependency.requirement, "3.17.0");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "3.18.0");
}

#[test]
fn apply_command_updates_gradle_version_catalog_plugin_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-gradle-version-catalog-plugin-version.versions.toml"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"),
        selected_version: Some("0.52.0"),
        responses: &[RegistryResponseInput {
            package: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "plugins");
    assert_eq!(output.suggestions[0].dependency.requirement, "0.51.0");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.52.0");
}

#[test]
fn apply_command_updates_sbt_library_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("apply-command-updates-sbt-library-dependency-version.sbt"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.scala-stm:scala-stm_2.13"),
        selected_version: Some("0.9.2"),
        responses: &[RegistryResponseInput {
            package: "org.scala-stm:scala-stm_2.13".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
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
        input: DocumentInput {
            uri: "file:///build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("apply-command-updates-sbt-dependency-override-version.sbt"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("log4j:log4j"),
        selected_version: Some("1.2.17"),
        responses: &[RegistryResponseInput {
            package: "log4j:log4j".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>1.2.16</version><version>1.2.17</version></versions></versioning></metadata>"#.to_owned(),
        }],
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
        input: DocumentInput {
            uri: "file:///build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-sbt-url-artifact-dependency.sbt"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("slinky:slinky"),
        selected_version: Some("2.2"),
        responses: &[RegistryResponseInput {
            package: "slinky:slinky".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.1</version><version>2.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("package URL"));
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_clojure_deps_edn_maven_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("apply-command-updates-clojure-deps-edn-maven-version.edn"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.clojure:tools.reader"),
        selected_version: Some("1.2.0"),
        responses: &[RegistryResponseInput {
            package: "org.clojure:tools.reader".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "deps");
    assert_eq!(output.suggestions[0].dependency.requirement, "1.1.1");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.0");
}

#[test]
fn apply_command_updates_leiningen_project_clj_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///project.clj".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("apply-command-updates-leiningen-project-clj-dependency-version.clj"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("org.clojure:clojure"),
        selected_version: Some("1.12.0"),
        responses: &[RegistryResponseInput {
            package: "org.clojure:clojure".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "dependencies");
    assert_eq!(output.suggestions[0].dependency.requirement, "1.11.3");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.12.0");
}

#[test]
fn apply_command_updates_packages_config_version_attribute() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///packages.config".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-packages-config-version-attribute.config",
            ),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("jQuery"),
        selected_version: Some("3.7.1"),
        responses: &[RegistryResponseInput {
            package: "jQuery".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["3.1.1","3.7.1"]}"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "packages.package");
    assert_eq!(output.suggestions[0].dependency.requirement, "3.1.1");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "3.7.1");
}
