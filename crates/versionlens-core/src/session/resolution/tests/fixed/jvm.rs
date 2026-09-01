#[test]
fn gradle_version_catalog_references_are_fixed_without_registry_updates() {
    let output = resolve_fixture!(
        "file:///repo/gradle/libs.versions.toml",
        "toml",
        "version-catalog-references-are-fixed-without-registry-updates.versions.toml",
        &[RegistryResponseInput::new("org.codehaus.groovy:groovy".to_owned(), Maven, r#"<metadata><versioning><versions><version>4.0.0</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_suggestion(&output, 0, "fixed", Some("version catalog alias"));
    assert_suggestion(&output, 1, "fixed", Some("version catalog reference"));
    assert_no_edits(&output);
}

#[test]
fn gradle_version_catalog_direct_library_versions_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/gradle/libs.versions.toml",
        "toml",
        "version-catalog-direct-library-versions-use-maven-lookup.versions.toml",
        &[RegistryResponseInput::new("org.apache.commons:commons-lang3".to_owned(), Maven, r#"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("3.17.0"));
    assert_no_edits(&output);
}

#[test]
fn sbt_scala_cross_dependencies_without_scala_version_are_fixed() {
    let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-scala-cross-dependencies-without-scala-version-are-fixed.sbt",
        &[RegistryResponseInput::new("org.typelevel:cats-core".to_owned(), Maven, r#"<metadata><versioning><versions><version>2.13.0</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("Scala binary version"));
    assert_no_edits(&output);
}

#[test]
fn sbt_maven_dependencies_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-maven-dependencies-use-maven-lookup.sbt",
        &[RegistryResponseInput::new("org.scala-stm:scala-stm_2.13".to_owned(), Maven, r#"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("0.9.1"));
    assert_no_edits(&output);
}

#[test]
fn sbt_url_artifact_dependencies_are_fixed_without_registry_updates() {
    let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-url-artifact-dependencies-are-fixed-without-registry-updates.sbt",
        &[RegistryResponseInput::new("jquery:jquery".to_owned(), Maven, r#"<metadata><versioning><versions><version>3.2.2</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("package URL"));
    assert_no_edits(&output);
}

#[test]
fn gradle_build_dependencies_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/build.gradle.kts",
        "kotlin",
        "build-dependencies-use-maven-lookup.gradle.kts",
        &[RegistryResponseInput::new("org.springframework:spring-core".to_owned(), Maven, r#"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("6.2.8"));
    assert_no_edits(&output);
}

#[test]
fn gradle_plugin_markers_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/settings.gradle",
        "groovy",
        "plugin-markers-use-maven-lookup.gradle",
        &[RegistryResponseInput::new("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(), Maven, r#"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("0.51.0"));
    assert_no_edits(&output);
}

#[test]
fn gradle_kotlin_shorthand_plugin_routes_to_maven_marker_and_updates() {
    let session = standard_session();
    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///repo/build.gradle.kts".to_owned(),
            "kotlin".to_owned(),
            "plugins {\n    kotlin(\"jvm\") version \"2.0.0\"\n}".to_owned(),
            None,
        ),
        &[RegistryResponseInput::new(
            "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin".to_owned(),
            Maven,
            r#"{"versions":["2.0.0","2.1.0"]}"#.to_owned(),
        )],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "updateAvailable", Some("2.1.0"));
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.1.0");
}

#[test]
fn gradle_project_and_file_dependencies_are_fixed() {
    let output = resolve_fixture!(
        "file:///repo/build.gradle",
        "groovy",
        "project-and-file-dependencies-are-fixed.gradle",
        &[RegistryResponseInput::new(":shared".to_owned(), Maven, r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_suggestion(&output, 0, "fixed", Some("local package"));
    assert_suggestion(&output, 1, "fixed", Some("local package"));
    assert_no_edits(&output);
}

#[test]
fn gradle_kotlin_named_argument_dependencies_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/build.gradle.kts",
        "kotlin",
        "kotlin-named-argument-dependencies-use-maven-lookup.gradle.kts",
        &[RegistryResponseInput::new("org.slf4j:slf4j-api".to_owned(), Maven, r#"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("2.0.17"));
    assert_no_edits(&output);
}

#[test]
fn clojure_deps_edn_git_and_local_dependencies_are_fixed() {
    let output = resolve_fixture!(
        "file:///repo/deps.edn",
        "clojure",
        "clojure-deps-edn-git-and-local-dependencies-are-fixed.edn",
        &[
            RegistryResponseInput::new("io.github.sally:awesome".to_owned(), Maven, r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned()),
            RegistryResponseInput::new("my.dev:project".to_owned(), Maven, r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned()),
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_suggestion(&output, 0, "fixed", Some("git repository"));
    assert_suggestion(&output, 1, "fixed", Some("local package"));
    assert_no_edits(&output);
}

#[test]
fn clojure_deps_edn_maven_dependencies_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/deps.edn",
        "clojure",
        "clojure-deps-edn-maven-dependencies-use-maven-lookup.edn",
        &[RegistryResponseInput::new("org.clojure:tools.reader".to_owned(), Maven, r#"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_suggestion(&output, 0, "fixed", Some("1.1.1"));
    assert_no_edits(&output);
}

#[test]
fn leiningen_project_clj_dependencies_use_maven_lookup() {
    let output = resolve_fixture!(
        "file:///repo/project.clj",
        "clojure",
        "leiningen-project-clj-dependencies-use-maven-lookup.clj",
        &[
            RegistryResponseInput::new("demo".to_owned(), Maven, r#"<metadata><versioning><versions><version>0.1.0-SNAPSHOT</version></versions></versioning></metadata>"#.to_owned()),
            RegistryResponseInput::new("org.clojure:clojure".to_owned(), Maven, r#"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>"#.to_owned()),
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "current");
    assert_suggestion(&output, 1, "fixed", Some("1.11.3"));
    assert_no_edits(&output);
}
