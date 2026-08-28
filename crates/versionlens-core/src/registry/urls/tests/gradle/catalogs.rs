#[test]
fn gradle_plugin_markers_use_plugin_portal_before_maven_central() {
    assert_registry_urls_for_fixture(
        "file:///settings.gradle",
        "groovy",
        "gradle-plugin-markers-use-plugin-portal-before-maven-central.gradle",
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin",
        &[
            GRADLE_PLUGIN_MARKER_URLS[0],
            GRADLE_PLUGIN_MARKER_URLS[1],
        ],
    );
}

#[test]
fn gradle_version_catalog_plugin_aliases_use_plugin_marker_lookup() {
    assert_registry_urls_for_fixture(
        "file:///gradle/libs.versions.toml",
        "toml",
        "gradle-version-catalog-plugin-aliases-use-plugin-marker-lookup.versions.toml",
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin",
        &[
            "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
        ],
    );
}

#[test]
fn gradle_version_catalog_library_aliases_use_module_lookup() {
    assert_registry_urls_for_fixture(
        "file:///gradle/libs.versions.toml",
        "toml",
        "gradle-version-catalog-library-aliases-use-module-lookup.versions.toml",
        "org.apache.commons:commons-lang3",
        &["https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml"],
    );
}

#[test]
fn gradle_version_catalog_libraries_use_workspace_settings_repositories() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-version-catalog-settings-repositories", settings: r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
    }
}
"#, relative_document: "gradle/libs.versions.toml", language: "toml", fixture: "gradle-version-catalog-libraries-use-workspace-settings-repositories.txt", dependency_name: "org.apache.commons:commons-lang3", expected_urls: &[
            "https://settings.example.test/releases/org/apache/commons/commons-lang3/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml",
        ] });
}

#[test]
fn gradle_version_catalog_plugins_use_workspace_plugin_management_repositories() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-version-catalog-plugin-management-repositories", settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#, relative_document: "gradle/libs.versions.toml", language: "toml", fixture: "gradle-version-catalog-plugins-use-workspace-plugin-management-repositories.txt", dependency_name: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin", expected_urls: &[
            "https://plugins.example.test/releases/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
        ] });
}
