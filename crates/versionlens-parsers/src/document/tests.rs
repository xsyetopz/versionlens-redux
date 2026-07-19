use super::parse_document;
use super::test_support::extract_range;
use crate::DocumentInput;
use crate::parse_clojure_maven_repositories;
use crate::parse_gradle_dependency_maven_repositories;
use crate::parse_gradle_maven_repositories;
use crate::parse_gradle_plugin_maven_repositories;
use crate::parse_leiningen_maven_repositories;
use crate::parse_sbt_maven_repositories;
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::Ecosystem::{CocoaPods, Docker, Hex, Maven, Unity};
#[test]
fn parses_unity_project_manifest_dependencies() {
    let text = package_file_fixture("parses-unity-project-manifest-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Packages/manifest.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Unity);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "com.unity.timeline");
    assert_eq!(dependencies[0].requirement, "1.8.7");
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(dependencies[1].name, "com.example.tools.physics");
    assert_eq!(
        dependencies[1].hosted_url,
        Some("https://registry.example.com".to_owned())
    );
    assert_eq!(dependencies[2].name, "com.acme.local");
    assert_eq!(dependencies[2].hosted_url, Some("file".to_owned()));
    assert_eq!(dependencies[3].name, "com.acme.git");
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
}

#[test]
fn parses_cocoapods_podfile_dependencies() {
    let text = package_file_fixture("parses-cocoapods-podfile-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Podfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, CocoaPods);
    assert_eq!(dependencies[0].group, "target App");
    assert_eq!(dependencies[0].name, "AFNetworking");
    assert_eq!(dependencies[0].requirement, "~> 4.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "~> 4.0"
    );
    assert_eq!(dependencies[0].requirement_prefix, "~> ");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("https://cdn.cocoapods.org")
    );
    assert_eq!(dependencies[1].name, "SSZipArchive");
    assert_eq!(dependencies[1].requirement, "latest");
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("latest"));
    assert_eq!(dependencies[2].name, "LocalPod");
    assert_eq!(dependencies[2].requirement, "../LocalPod");
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("path"));
    assert_eq!(dependencies[3].name, "GitPod");
    assert_eq!(dependencies[3].requirement, "1.2.3");
    assert_eq!(dependencies[3].hosted_url.as_deref(), Some("git"));
    assert_eq!(dependencies[4].name, "JSONKit");
    assert_eq!(
        dependencies[4].requirement,
        "https://example.com/JSONKit.podspec"
    );
    assert_eq!(dependencies[4].hosted_url.as_deref(), Some("podspec"));
    assert_eq!(dependencies[5].name, "PonyDebugger");
    assert_eq!(dependencies[5].requirement, "latest");
    assert_eq!(
        dependencies[5].hosted_url.as_deref(),
        Some("https://private.example.com/specs")
    );
}

#[test]
fn parses_kustomization_yaml_images() {
    let text = package_file_fixture("parses-kustomization-yaml-images.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/kustomization.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Docker);
    assert_eq!(dependencies[0].group, "images");
    assert_eq!(dependencies[0].name, "platform/nginx");
    assert_eq!(dependencies[0].requirement, "1.25.3");
    assert_eq!(
        dependencies[0].hosted_url,
        Some("registry.example.com".to_owned())
    );
    assert_eq!(dependencies[0].hosted_name, Some("nginx".to_owned()));
    assert_eq!(dependencies[1].name, "redis");
    assert_eq!(dependencies[1].requirement, "sha256:abcdef");
    assert_eq!(dependencies[1].requirement_prefix, "digest: ");
}
#[test]
fn parses_rebar_config_dependencies() {
    let text = package_file_fixture("parses-rebar-config-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/rebar.config".to_owned(),
        language_id: "erlang".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Hex);
    assert_eq!(dependencies[0].group, "deps");
    assert_eq!(dependencies[0].name, "rebar");
    assert_eq!(dependencies[0].requirement, "latest");
    assert_eq!(dependencies[1].name, "cowboy");
    assert_eq!(dependencies[1].requirement, "2.12.0");
    assert_eq!(dependencies[2].name, "lager_fork");
    assert_eq!(dependencies[2].requirement, "latest");
    assert_eq!(dependencies[2].hosted_name, Some("lager".to_owned()));
    assert_eq!(dependencies[3].name, "jsx_fork");
    assert_eq!(dependencies[3].requirement, "3.1.0");
    assert_eq!(dependencies[3].hosted_name, Some("jsx".to_owned()));
    assert_eq!(dependencies[4].name, "gettext");
    assert_eq!(
        dependencies[4].requirement,
        "https://github.com/elixir-lang/gettext.git"
    );
    assert_eq!(dependencies[4].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[5].name, "local_dep");
    assert_eq!(
        dependencies[5].requirement,
        "https://github.com/example/local.git"
    );
    assert_eq!(dependencies[5].hosted_url, Some("git".to_owned()));
}

#[test]
fn parses_gleam_toml_dependencies() {
    let text = package_file_fixture("parses-gleam-toml-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/gleam.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Hex);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "1.0.0");
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "gleam_stdlib");
    assert_eq!(dependencies[1].requirement, ">= 0.44.0 and < 2.0.0");
    assert_eq!(dependencies[2].name, "my_other_project");
    assert_eq!(dependencies[2].requirement, "../my_other_project");
    assert_eq!(dependencies[2].hosted_url, Some("path".to_owned()));
    assert_eq!(dependencies[3].name, "my_library");
    assert_eq!(
        dependencies[3].requirement,
        "https://github.com/my-project/my-library"
    );
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[4].group, "dev_dependencies");
    assert_eq!(dependencies[4].name, "gleeunit");
}

#[test]
fn parses_gradle_version_catalog_dependencies() {
    let text = package_file_fixture("parses-gradle-version-catalog-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/gradle/libs.versions.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "versions");
    assert_eq!(dependencies[0].name, "groovy");
    assert_eq!(dependencies[0].requirement, "3.0.5");
    assert_eq!(dependencies[1].name, "kotlin");
    assert_eq!(dependencies[1].requirement, "2.0.20");
    assert_eq!(dependencies[1].requirement_prefix, "prefer = \"");
    assert_eq!(dependencies[2].group, "libraries");
    assert_eq!(dependencies[2].name, "org.codehaus.groovy:groovy");
    assert_eq!(dependencies[2].requirement, "groovy");
    assert_eq!(dependencies[2].hosted_url, Some("version.ref".to_owned()));
    assert_eq!(dependencies[3].name, "org.apache.commons:commons-lang3");
    assert_eq!(dependencies[3].requirement, "3.17.0");
    assert_eq!(dependencies[4].group, "plugins");
    assert_eq!(
        dependencies[4].name,
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
    );
    assert_eq!(dependencies[4].requirement, "0.45.0");
    assert_eq!(dependencies[5].group, "plugins");
    assert_eq!(
        dependencies[5].name,
        "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
    );
    assert_eq!(dependencies[5].requirement, "kotlin");
    assert_eq!(dependencies[5].hosted_url, Some("version.ref".to_owned()));
}

#[test]
fn parses_sbt_build_dependencies() {
    let text = package_file_fixture("parses-sbt-build-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "libraryDependencies");
    assert_eq!(dependencies[0].name, "org.apache.derby:derby");
    assert_eq!(dependencies[0].requirement, "10.4.1.3");
    assert_eq!(dependencies[1].name, "org.scala-stm:scala-stm_2.13");
    assert_eq!(dependencies[1].requirement, "0.9.1");
    assert_eq!(dependencies[1].hosted_name, Some("scala-stm".to_owned()));
    assert_eq!(dependencies[2].name, "org.typelevel:cats-core_2.13");
    assert_eq!(dependencies[2].requirement, "2.12.0");
    assert_eq!(dependencies[3].name, "com.lihaoyi:os-lib_2.13");
    assert_eq!(dependencies[3].requirement, "0.11.3");
}

#[test]
fn parses_sbt_multiple_dependencies_on_one_line() {
    let text = package_file_fixture("parses-sbt-multiple-dependencies-on-one-line.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "org.zeta:zeta");
    assert_eq!(dependencies[0].requirement, "1.0.0");
    assert_eq!(dependencies[1].name, "org.alpha:alpha");
    assert_eq!(dependencies[1].requirement, "2.0.0");
    assert_eq!(
        dependencies[1].hosted_url,
        Some("scala-binary-version".to_owned())
    );
}

#[test]
fn parses_sbt_dependency_overrides() {
    let text = package_file_fixture("parses-sbt-dependency-overridesbuild.sbt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "dependencyOverrides");
    assert_eq!(dependencies[0].name, "log4j:log4j");
    assert_eq!(dependencies[0].requirement, "1.2.16");
}

#[test]
fn parses_sbt_double_percent_without_scala_version_as_fixed_source() {
    let text = package_file_fixture(
        "parses-sbt-double-percent-without-scala-version-as-fixed-sourcebuild.sbt",
    );

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "org.typelevel:cats-core");
    assert_eq!(dependencies[0].requirement, "2.12.0");
    assert_eq!(
        dependencies[0].hosted_url,
        Some("scala-binary-version".to_owned())
    );
}

#[test]
fn parses_sbt_scoped_scala_version_for_cross_dependencies() {
    let text =
        package_file_fixture("parses-sbt-scoped-scala-version-for-cross-dependenciesbuild.sbt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "org.typelevel:cats-core_2.13");
    assert_eq!(dependencies[0].requirement, "2.12.0");
    assert_eq!(dependencies[0].hosted_name, Some("cats-core".to_owned()));
}

#[test]
fn parses_sbt_url_artifact_dependency_as_non_registry_source() {
    let text =
        package_file_fixture("parses-sbt-url-artifact-dependency-as-non-registry-sourcebuild.sbt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "jquery:jquery");
    assert_eq!(dependencies[0].requirement, "3.2.1");
    assert_eq!(dependencies[0].hosted_url, Some("url".to_owned()));
}

#[test]
fn parses_sbt_maven_resolvers() {
    let repositories = parse_sbt_maven_repositories(
        r#"resolvers += "Private Maven" at "https://maven.example.test/releases"
resolvers ++= Seq(
  "Snapshots" at "https://maven.example.test/snapshots",
  Resolver.mavenLocal
)
"#,
    );

    assert_eq!(repositories.len(), 2);
    assert_eq!(repositories[0].id, "Private Maven");
    assert_eq!(repositories[0].url, "https://maven.example.test/releases");
    assert_eq!(repositories[1].id, "Snapshots");
    assert_eq!(repositories[1].url, "https://maven.example.test/snapshots");
}

#[test]
fn parses_gradle_build_dependencies_and_plugins() {
    let text = package_file_fixture("parses-gradle-build-dependencies-and-plugins.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.gradle".to_owned(),
        language_id: "groovy".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "plugins");
    assert_eq!(
        dependencies[0].name,
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
    );
    assert_eq!(dependencies[0].requirement, "0.51.0");
    assert_eq!(dependencies[1].group, "plugins");
    assert_eq!(
        dependencies[1].name,
        "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
    );
    assert_eq!(dependencies[1].requirement, "2.1.20");
    assert_eq!(dependencies[2].group, "implementation");
    assert_eq!(dependencies[2].name, "org.springframework:spring-core");
    assert_eq!(dependencies[2].requirement, "6.2.8");
    assert_eq!(dependencies[3].group, "testImplementation");
    assert_eq!(dependencies[3].name, "junit:junit");
    assert_eq!(dependencies[3].requirement, "4.13.2");
    assert_eq!(dependencies[4].group, "runtimeOnly");
    assert_eq!(dependencies[4].name, "org.postgresql:postgresql");
    assert_eq!(dependencies[4].requirement, "42.7.3");
    assert_eq!(dependencies[5].group, "compileOnly");
    assert_eq!(dependencies[5].name, ":local");
    assert_eq!(dependencies[5].requirement, ":local");
    assert_eq!(dependencies[5].hosted_url, Some("local".to_owned()));
    assert_eq!(dependencies[6].group, "annotationProcessor");
    assert_eq!(dependencies[6].name, "libs/processor.jar");
    assert_eq!(dependencies[6].requirement, "libs/processor.jar");
    assert_eq!(dependencies[6].hosted_url, Some("local".to_owned()));
}

#[test]
fn parses_gradle_settings_plugins() {
    let text = package_file_fixture("parses-gradle-settings-plugins.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/settings.gradle.kts".to_owned(),
        language_id: "kotlin".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "plugins");
    assert_eq!(
        dependencies[0].name,
        "com.android.application:com.android.application.gradle.plugin"
    );
    assert_eq!(dependencies[0].requirement, "8.12.0");
}

#[test]
fn parses_gradle_kotlin_named_argument_dependencies() {
    let text =
        package_file_fixture("parses-gradle-kotlin-named-argument-dependenciessettings.gradle.kts");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.gradle.kts".to_owned(),
        language_id: "kotlin".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "implementation");
    assert_eq!(dependencies[0].name, "org.slf4j:slf4j-api");
    assert_eq!(dependencies[0].requirement, "2.0.17");
    assert_eq!(dependencies[1].group, "testImplementation");
    assert_eq!(dependencies[1].name, "org.assertj:assertj-core");
    assert_eq!(dependencies[1].requirement, "3.27.3");
}

#[test]
fn parses_gradle_maven_repositories() {
    let repositories = parse_gradle_maven_repositories(
        r#"repositories {
    maven {
        url = uri("https://maven.example.test/releases")
    }
    google()
    gradlePluginPortal()
    maven {
        url 'https://maven.example.test/snapshots'
    }
    mavenCentral()
    mavenLocal()
}
"#,
    );

    assert_eq!(repositories.len(), 5);
    assert_eq!(repositories[0].id, "maven");
    assert_eq!(repositories[0].url, "https://maven.example.test/releases");
    assert_eq!(repositories[1].id, "google");
    assert_eq!(
        repositories[1].url,
        "https://dl.google.com/dl/android/maven2/"
    );
    assert_eq!(repositories[2].id, "gradlePluginPortal");
    assert_eq!(repositories[2].url, "https://plugins.gradle.org/m2/");
    assert_eq!(repositories[3].id, "maven");
    assert_eq!(repositories[3].url, "https://maven.example.test/snapshots");
    assert_eq!(repositories[4].id, "mavenCentral");
    assert_eq!(repositories[4].url, "https://repo.maven.apache.org/maven2");
}

#[test]
fn parses_gradle_plugin_maven_repositories_separately() {
    let text = package_file_fixture("parses-gradle-plugin-maven-repositories-separately.txt");

    let plugin_repositories = parse_gradle_plugin_maven_repositories(text);
    let dependency_repositories = parse_gradle_dependency_maven_repositories(text);

    assert_eq!(plugin_repositories.len(), 2);
    assert_eq!(plugin_repositories[0].id, "maven");
    assert_eq!(
        plugin_repositories[0].url,
        "https://plugins.example.test/releases"
    );
    assert_eq!(plugin_repositories[1].id, "gradlePluginPortal");
    assert_eq!(plugin_repositories[1].url, "https://plugins.gradle.org/m2/");
    assert_eq!(dependency_repositories.len(), 1);
    assert_eq!(dependency_repositories[0].id, "mavenCentral");
    assert_eq!(
        dependency_repositories[0].url,
        "https://repo.maven.apache.org/maven2"
    );
}

include!("tests/cpp.rs");
include!("tests/jvm.rs");
include!("tests/native_infra.rs");

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/document/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
