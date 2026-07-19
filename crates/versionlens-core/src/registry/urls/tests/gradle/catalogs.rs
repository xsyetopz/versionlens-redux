#[test]
fn gradle_plugin_markers_use_plugin_portal_before_maven_central() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///settings.gradle".to_owned(),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-plugin-markers-use-plugin-portal-before-maven-central.gradle",
        ),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);

    assert_eq!(
        dependencies[0].name,
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
    );
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec![
            "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_version_catalog_plugin_aliases_use_plugin_marker_lookup() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///gradle/libs.versions.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "gradle-version-catalog-plugin-aliases-use-plugin-marker-lookup.versions.toml",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        dependencies[0].name,
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_version_catalog_library_aliases_use_module_lookup() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///gradle/libs.versions.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "gradle-version-catalog-library-aliases-use-module-lookup.versions.toml",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "org.apache.commons:commons-lang3");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_version_catalog_libraries_use_workspace_settings_repositories() {
    let root = temp_dir().join(format!(
        "versionlens-gradle-version-catalog-settings-repositories-{}",
        id()
    ));
    let gradle_dir = root.join("gradle");
    create_dir_all(&gradle_dir).unwrap();
    write(
        root.join("settings.gradle"),
        r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
    }
}
"#,
    )
    .unwrap();

    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", gradle_dir.join("libs.versions.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "gradle-version-catalog-libraries-use-workspace-settings-repositories.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "org.apache.commons:commons-lang3");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://settings.example.test/releases/org/apache/commons/commons-lang3/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml"
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gradle_version_catalog_plugins_use_workspace_plugin_management_repositories() {
    let root = temp_dir().join(format!(
        "versionlens-gradle-version-catalog-plugin-management-repositories-{}",
        id()
    ));
    let gradle_dir = root.join("gradle");
    create_dir_all(&gradle_dir).unwrap();
    write(
        root.join("settings.gradle"),
        r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#,
    )
    .unwrap();

    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: format!("file://{}", gradle_dir.join("libs.versions.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "gradle-version-catalog-plugins-use-workspace-plugin-management-repositories.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(
        dependencies[0].name,
        "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://plugins.example.test/releases/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml"
        ]
    );

    remove_dir_all(root).unwrap();
}
