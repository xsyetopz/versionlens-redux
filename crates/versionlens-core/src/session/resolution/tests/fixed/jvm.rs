#[test]
fn gradle_version_catalog_references_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gradle-version-catalog-references-are-fixed-without-registry-updates.versions.toml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.codehaus.groovy:groovy".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>4.0.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("version catalog alias")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("version catalog reference")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_version_catalog_direct_library_versions_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gradle-version-catalog-direct-library-versions-use-maven-lookup.versions.toml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.apache.commons:commons-lang3".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("3.17.0"));
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_scala_cross_dependencies_without_scala_version_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-scala-cross-dependencies-without-scala-version-are-fixed.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.typelevel:cats-core".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.13.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("Scala binary version")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_maven_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-maven-dependencies-use-maven-lookup.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.scala-stm:scala-stm_2.13".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("0.9.1"));
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_url_artifact_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-url-artifact-dependencies-are-fixed-without-registry-updates.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "jquery:jquery".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>3.2.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("package URL"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_build_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("gradle-build-dependencies-use-maven-lookup.gradle.kts"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.springframework:spring-core".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("6.2.8"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_plugin_markers_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/settings.gradle".to_owned(),
            language_id: "groovy".to_owned(),
            text: package_file_fixture("gradle-plugin-markers-use-maven-lookup.gradle"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("0.51.0"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_project_and_file_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle".to_owned(),
            language_id: "groovy".to_owned(),
            text: package_file_fixture("gradle-project-and-file-dependencies-are-fixed.gradle"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: ":shared".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("local package")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_kotlin_named_argument_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("gradle-kotlin-named-argument-dependencies-use-maven-lookup.gradle.kts"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.slf4j:slf4j-api".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("2.0.17"));
    assert!(output.edits.is_empty());
}

#[test]
fn clojure_deps_edn_git_and_local_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("clojure-deps-edn-git-and-local-dependencies-are-fixed.edn"),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "io.github.sally:awesome".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
            },
            RegistryResponseInput {
                package: "my.dev:project".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn clojure_deps_edn_maven_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("clojure-deps-edn-maven-dependencies-use-maven-lookup.edn"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.clojure:tools.reader".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
}

#[test]
fn leiningen_project_clj_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project.clj".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("leiningen-project-clj-dependencies-use-maven-lookup.clj"),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "demo".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>0.1.0-SNAPSHOT</version></versions></versioning></metadata>"#.to_owned(),
            },
            RegistryResponseInput {
                package: "org.clojure:clojure".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("1.11.3"));
    assert!(output.edits.is_empty());
}
