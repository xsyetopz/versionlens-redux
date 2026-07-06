use versionlens_parsers::Ecosystem::{Cargo, Composer, Deno, Dotnet, Go, Maven, Python};
#[test]
fn apply_command_preserves_semver_requirement_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-preserves-semver-requirement-prefix.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("left-pad"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "^2.0.0");
}

#[test]
fn apply_command_preserves_composer_stability_flag_suffix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///composer.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-preserves-composer-stability-flag-suffix.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("monolog/monolog"),
        &[RegistryResponseInput {
            package: "monolog/monolog".to_owned(),
            ecosystem: Composer,
            body: r#"{"packages":{"monolog/monolog":[{"version":"1.1.0"}]}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0@beta");
}

#[test]
fn apply_command_updates_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-project-version-by-requested-level.json"),
            workspace_root: None,
        },
        Some("updateMajor"),
        Some("1.2.3"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_updates_jsr_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///jsr.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-jsr-project-version-by-requested-level.json"),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("@scope/pkg"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_deno_json_jsr_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("apply-command-updates-deno-json-jsr-project-version-by-requested-level.json"),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("@scope/pkg"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_prerelease_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-prerelease-project-version-by-requested-level.json"),
            workspace_root: None,
        },
        Some("updateRelease"),
        Some("1.2.3-beta.4"),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3");
}

#[test]
fn apply_command_updates_only_project_versions_for_prerelease_command() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-only-project-versions-for-prerelease-command.json"),
            workspace_root: None,
        },
        Some("updatePrerelease"),
        None,
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.3-beta.5");
}

#[test]
fn apply_command_updates_cargo_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-cargo-project-version-by-requested-level.toml"),
            workspace_root: None,
        },
        Some("updatePatch"),
        Some("version"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "package");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.2.4");
}

#[test]
fn apply_command_updates_cargo_renamed_package_version_preserving_alias() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Cargo.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-cargo-renamed-package-version-preserving-alias.toml"),
            workspace_root: None,
        },
        None,
        Some("local_name"),
        &[RegistryResponseInput {
            package: "registry-name".to_owned(),
            ecosystem: Cargo,
            body: r#"{"versions":[{"num":"1.1.0"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.name, "local_name");
    assert_eq!(
        output.suggestions[0].dependency.hosted_name.as_deref(),
        Some("registry-name")
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn apply_command_updates_go_hyphenated_prerelease_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("apply-command-updates-go-hyphenated-prerelease-version.mod"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("example.test/prerelease"),
        selected_version: Some("v1.0.0"),
        responses: &[RegistryResponseInput {
            package: "example.test/prerelease".to_owned(),
            ecosystem: Go,
            body: "v1.0.0-alpha-beta\nv1.0.0\n".to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(
        output.suggestions[0].dependency.requirement,
        "v1.0.0-alpha-beta"
    );
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "v1.0.0");
}

#[test]
fn apply_command_updates_bare_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///requirements.txt".to_owned(),
            language_id: "pip-requirements".to_owned(),
            text: package_file_fixture("apply-command-updates-bare-requirements-with-equals-prefix.txt"),
            workspace_root: None,
        },
        None,
        Some("importlib-metadata"),
        &[RegistryResponseInput {
            package: "importlib-metadata".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"8.7.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==8.7.0");
}

#[test]
fn apply_command_updates_empty_pipfile_requirements_with_equals_prefix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Pipfile".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-empty-pipfile-requirements-with-equals-prefix.Pipfile"),
            workspace_root: None,
        },
        None,
        Some("magic"),
        &[RegistryResponseInput {
            package: "magic".to_owned(),
            ecosystem: Python,
            body: r#"{"info":{"version":"1.2.3"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "==1.2.3");
}

#[test]
fn apply_command_updates_dotnet_package_reference_child_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///app.csproj".to_owned(),
            language_id: "xml".to_owned(),
            text: package_file_fixture("apply-command-updates-dotnet-package-reference-child-version.csproj"),
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
            text: package_file_fixture("apply-command-updates-packages-config-version-attribute.config"),
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

#[test]
fn apply_command_inserts_missing_deno_import_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("apply-command-inserts-missing-deno-import-versions.json"),
            workspace_root: None,
        },
        None,
        Some("@std/assert"),
        &[RegistryResponseInput {
            package: "@std/assert".to_owned(),
            ecosystem: Deno,
            body: r#"{"versions":{"1.0.1":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@std/assert@1.0.1");
}
