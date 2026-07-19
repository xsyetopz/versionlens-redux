use super::MavenRepository;
use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document, parse_document_with_dependency_paths};
use std::fs::read_to_string;
use std::path::PathBuf;

use super::{
    extract_maven_repository_urls, parse_maven_effective_settings_https_repositories,
    parse_maven_effective_settings_https_repository_sources,
    parse_maven_effective_settings_repositories, parse_maven_effective_settings_repository_sources,
    parse_maven_metadata_versions, parse_maven_pom_repositories, parse_maven_pom_repository_urls,
    parse_maven_settings_auth_entries, parse_maven_settings_mirror_urls,
    parse_maven_settings_mirrors, parse_maven_settings_repository_urls,
};
use versionlens_model::Ecosystem::Maven;

#[test]
fn parses_maven_pom_dependencies() {
    let text = package_file_fixture("parses-maven-pom-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "project.version");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.3.6-SNAPSHOT");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.3.6-SNAPSHOT"
    );
    assert_eq!(
        dependencies[1].name,
        "org.springframework.boot:spring-boot-starter-parent"
    );
    assert_eq!(dependencies[1].requirement, "1.5.16.RELEASE");
    assert_eq!(dependencies[2].name, "org.springframework:spring-core");
    assert_eq!(dependencies[2].requirement, "5.0.7.RELEASE");
    assert_eq!(dependencies[2].range.start.line, 12);
    assert_eq!(dependencies[2].range.start.character, 4);
    assert_eq!(dependencies[2].range.end.line, 16);
    assert_eq!(dependencies[2].range.end.character, 17);
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "5.0.7.RELEASE"
    );
    assert_eq!(dependencies[3].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[3].requirement, "9.0.12");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "9.0.12"
    );
}

#[test]
fn parses_maven_plugin_dependencies_by_default() {
    let text = package_file_fixture("parses-maven-plugin-dependencies-by-default.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "project.build.plugins.plugin");
    assert_eq!(
        dependencies[0].name,
        "org.apache.maven.plugins:maven-compiler-plugin"
    );
    assert_eq!(dependencies[0].requirement, "3.13.0");
    assert_eq!(
        dependencies[1].group,
        "project.build.pluginManagement.plugins.plugin"
    );
    assert_eq!(
        dependencies[1].name,
        "org.codehaus.mojo:versions-maven-plugin"
    );
    assert_eq!(dependencies[1].requirement, "2.16.2");
}

#[test]
fn maven_property_references_trim_before_resolution_like_upstream() {
    let text =
        package_file_fixture("maven-property-references-trim-before-resolution-like-upstream.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
}

#[test]
fn resolves_maven_project_and_parent_interpolation_properties() {
    let text =
        package_file_fixture("resolves-maven-project-and-parent-interpolation-properties.xml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "org.parent:parent-pom");
    assert_eq!(dependencies[0].requirement, "3.4.5");
    assert_eq!(dependencies[1].name, "org.parent:runtime");
    assert_eq!(dependencies[1].requirement, "3.4.5");
    assert_eq!(dependencies[2].name, "org.example:child-app");
    assert_eq!(dependencies[2].requirement, "3.4.5");
}

#[test]
fn parses_smoke_maven_pom_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-maven-pom-smoke-shapes.xml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].group, "project.version");
    assert_eq!(dependencies[0].requirement, "1.3.6-SNAPSHOT");
    assert_eq!(
        dependencies[1].name,
        "org.springframework.boot:spring-boot-starter-parent"
    );
    assert_eq!(dependencies[1].requirement, "4.1.0");
    assert_eq!(dependencies[4].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[4].requirement, "11.0.23");
    assert_eq!(dependencies[5].name, "com.oracle:ojdbc6");
    assert_eq!(dependencies[5].requirement, "10.0");
    assert_eq!(dependencies[6].name, "crespo.fernando:condominio");
    assert_eq!(dependencies[6].requirement, "*");
    assert_eq!(dependencies[7].name, "crespo.fernando:other");
    assert_eq!(dependencies[7].requirement, "*");
}

#[test]
fn parses_maven_dependency_management_dependencies_by_default() {
    let text =
        package_file_fixture("parses-maven-dependency-management-dependencies-by-default.txt");
    let default_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });
    assert_eq!(default_dependencies.len(), 1);
    let dependencies = default_dependencies;

    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].group,
        "project.dependencyManagement.dependencies.dependency"
    );
    assert_eq!(dependencies[0].name, "org.example:managed");
    assert_eq!(dependencies[0].requirement, "2.3.4");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.3.4"
    );
}

#[test]
fn parses_maven_profile_dependencies_by_default() {
    let text = package_file_fixture("parses-maven-profile-dependencies-by-default.xml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(
        dependencies[0].group,
        "project.profiles.profile.dependencies.dependency"
    );
    assert_eq!(dependencies[0].name, "org.junit.jupiter:junit-jupiter");
    assert_eq!(dependencies[0].requirement, "5.11.4");
    assert_eq!(
        dependencies[1].group,
        "project.profiles.profile.dependencyManagement.dependencies.dependency"
    );
    assert_eq!(
        dependencies[1].name,
        "org.springframework.boot:spring-boot-dependencies"
    );
    assert_eq!(dependencies[1].requirement, "3.4.1");
    assert_eq!(
        dependencies[2].group,
        "project.profiles.profile.build.plugins.plugin"
    );
    assert_eq!(
        dependencies[2].name,
        "org.apache.maven.plugins:maven-surefire-plugin"
    );
    assert_eq!(dependencies[2].requirement, "3.5.2");
    assert_eq!(
        dependencies[3].group,
        "project.profiles.profile.build.pluginManagement.plugins.plugin"
    );
    assert_eq!(
        dependencies[3].name,
        "org.codehaus.mojo:versions-maven-plugin"
    );
    assert_eq!(dependencies[3].requirement, "2.18.0");
}

#[test]
fn parses_configured_maven_plugin_dependency_paths() {
    let text = package_file_fixture("parses-configured-maven-plugin-dependency-paths.txt");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project.build.plugins.plugin"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "project.build.plugins.plugin");
    assert_eq!(
        dependencies[0].name,
        "org.apache.maven.plugins:maven-compiler-plugin"
    );
    assert_eq!(dependencies[0].requirement, "3.14.0");
}

#[test]
fn configured_maven_dependency_paths_match_exact_nodes_only() {
    let text = package_file_fixture("configured-maven-dependency-paths-match-exact-nodes-only.xml");
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project.dependencies"],
    );

    assert_eq!(dependencies.len(), 0);
}

#[test]
fn maven_property_resolution_uses_first_matching_property() {
    let text = package_file_fixture("maven-property-resolution-uses-first-matching-property.xml");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[0].requirement, "9.0.12");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "9.0.12"
    );
}

#[test]
fn parses_maven_pom_repository_urls() {
    let text = package_file_fixture("parses-maven-pom-repository-urls.xml");

    assert_eq!(
        parse_maven_pom_repository_urls(text),
        vec![
            "https://packages.example.test/maven",
            "https://repo.maven.apache.org/maven2",
            "https://profile.example.test/releases",
            "https://profile-plugins.example.test/maven",
            "https://plugins.example.test/maven",
        ]
    );
}

#[test]
fn parses_maven_effective_settings_repositories() {
    let text = package_file_fixture("parses-maven-effective-settings-repositories.txt");

    assert_eq!(
        parse_maven_effective_settings_repositories(text),
        vec![
            "/Users/example/.m2/repository",
            "https://repo1.maven.org/maven2",
            "http://repo.example.test/releases",
            "https://plugins.example.test/maven",
        ]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repositories(text),
        vec![
            "https://repo1.maven.org/maven2",
            "https://plugins.example.test/maven",
        ]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repositories(""),
        vec!["https://repo.maven.apache.org/maven2/"]
    );
}

#[test]
fn parses_maven_metadata_versions() {
    let text = package_file_fixture("parses-maven-metadata-versions.txt");

    assert_eq!(
        parse_maven_metadata_versions(text),
        vec!["5.0.7.RELEASE", "5.1.0.RELEASE"]
    );
}

#[test]
fn parses_maven_repository_sources() {
    let text = package_file_fixture("parses-maven-repository-sources.txt");

    let sources = parse_maven_effective_settings_repository_sources(text);

    assert_eq!(sources.len(), 4);
    assert_eq!(sources[0].url, "/Users/example/.m2/repository");
    assert_eq!(sources[0].protocol, "file:");
    assert_eq!(sources[1].protocol, "https:");
    assert_eq!(sources[2].protocol, "http:");
    assert_eq!(sources[3].protocol, "https:");
    assert_eq!(
        extract_maven_repository_urls(&sources),
        vec![
            "/Users/example/.m2/repository",
            "https://repo1.maven.org/maven2",
            "http://repo.example.test/releases",
            "https://plugins.example.test/maven",
        ]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repository_sources(text),
        vec![sources[1].clone(), sources[3].clone()]
    );
    assert_eq!(
        parse_maven_effective_settings_repository_sources(""),
        vec![MavenRepository {
            url: "https://repo.maven.apache.org/maven2/".to_owned(),
            protocol: "https:".to_owned(),
        }]
    );
}

#[test]
fn parses_maven_settings_repositories_and_auth_entries() {
    let text = package_file_fixture("parses-maven-settings-repositories-and-auth-entries.txt");

    assert_eq!(
        parse_maven_settings_repository_urls(text),
        vec![
            "https://maven.example.test/repository/releases",
            "https://repo.maven.apache.org/maven2",
            "https://plugins.example.test/maven",
        ]
    );

    let entries = parse_maven_settings_auth_entries(text);
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0].registry,
        "https://maven.example.test/repository/releases"
    );
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
    assert_eq!(entries[1].registry, "https://plugins.example.test/maven");
    assert_eq!(
        entries[1].header_value,
        "Basic cGx1Z2luLXVzZXI6cGx1Z2luLXBhc3M="
    );
}

#[test]
fn parses_only_active_maven_settings_profile_repositories_when_active_profiles_are_declared() {
    let text = package_file_fixture(
        "parses-only-active-maven-settings-profile-repositories-when-active-profiles-are-declared.txt",
    );

    assert_eq!(
        parse_maven_settings_repository_urls(text),
        vec!["https://active.example.test/maven"]
    );

    let entries = parse_maven_settings_auth_entries(text);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].registry, "https://active.example.test/maven");
    assert_eq!(
        entries[0].header_value,
        "Basic YWN0aXZlLXVzZXI6YWN0aXZlLXBhc3M="
    );
}

include!("tests/repositories.rs");

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/maven_xml/tests")
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
