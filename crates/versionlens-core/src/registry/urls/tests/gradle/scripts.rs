#[test]
fn gradle_maven_repositories_are_used_before_maven_central() {
    assert_registry_urls_for_fixture(
        "file:///build.gradle",
        "groovy",
        "gradle-maven-repositories-are-used-before-maven-central.gradle",
        "com.example:demo",
        &[
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://dl.google.com/dl/android/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
        ],
    );
}

#[test]
fn gradle_plugin_portal_repository_is_used_for_regular_maven_dependencies() {
    assert_registry_urls_for_fixture(
        "file:///build.gradle",
        "groovy",
        "gradle-plugin-portal-repository-is-used-for-regular-maven-dependencies.gradle",
        "com.example:demo",
        &[
            "https://plugins.gradle.org/m2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
        ],
    );
}

#[test]
fn gradle_explicit_maven_central_preserves_repository_order() {
    assert_registry_urls_for_fixture(
        "file:///build.gradle",
        "groovy",
        "gradle-explicit-maven-central-preserves-repository-order.gradle",
        "com.example:demo",
        &[
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
        ],
    );
}

#[test]
fn gradle_build_uses_workspace_settings_repositories_before_maven_central() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-settings-repositories", settings: r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
        google()
    }
}
"#, relative_document: "build.gradle", language: "groovy", fixture: "gradle-build-uses-workspace-settings-repositories-before-maven-central.txt", dependency_name: "com.example:demo", expected_urls: &[
            "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://dl.google.com/dl/android/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
        ] });
}

#[test]
fn gradle_settings_prefer_settings_repositories_override_build_repositories() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-prefer-settings-repositories", settings: r#"dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.PREFER_SETTINGS
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
    }
}
"#, relative_document: "build.gradle", language: "groovy", fixture: "gradle-settings-prefer-settings-repositories-override-build-repositories.txt", dependency_name: "com.example:demo", expected_urls: &[
            "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
        ] });
}

#[test]
fn gradle_plugin_management_repositories_do_not_resolve_regular_dependencies() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-plugin-management-repositories", settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#, relative_document: "build.gradle", language: "groovy", fixture: "gradle-plugin-management-repositories-do-not-resolve-regular-dependencies.txt", dependency_name: "com.example:demo", expected_urls: &["https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"] });
}

#[test]
fn gradle_build_plugins_use_workspace_plugin_management_repositories() {
    assert_workspace_registry_urls(WorkspaceRegistryCase { root_name: "gradle-build-plugin-management-repositories", settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#, relative_document: "build.gradle", language: "groovy", fixture: "gradle-build-plugins-use-workspace-plugin-management-repositories.txt", dependency_name: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin", expected_urls: &[
            "https://plugins.example.test/releases/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            GRADLE_PLUGIN_MARKER_URLS[0],
            GRADLE_PLUGIN_MARKER_URLS[1],
        ] });
}
