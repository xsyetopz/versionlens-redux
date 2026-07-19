#[test]
fn gradle_maven_repositories_are_used_before_maven_central() {
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
        uri: "file:///build.gradle".to_owned(),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-maven-repositories-are-used-before-maven-central.gradle",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://dl.google.com/dl/android/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_plugin_portal_repository_is_used_for_regular_maven_dependencies() {
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
        uri: "file:///build.gradle".to_owned(),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-plugin-portal-repository-is-used-for-regular-maven-dependencies.gradle",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://plugins.gradle.org/m2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_explicit_maven_central_preserves_repository_order() {
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
        uri: "file:///build.gradle".to_owned(),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-explicit-maven-central-preserves-repository-order.gradle",
        ),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn gradle_build_uses_workspace_settings_repositories_before_maven_central() {
    let root = temp_dir().join(format!("versionlens-gradle-settings-repositories-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.gradle"),
        r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
        google()
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
        uri: format!("file://{}", root.join("build.gradle").display()),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-build-uses-workspace-settings-repositories-before-maven-central.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://dl.google.com/dl/android/maven2/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gradle_settings_prefer_settings_repositories_override_build_repositories() {
    let root = temp_dir().join(format!(
        "versionlens-gradle-prefer-settings-repositories-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.gradle"),
        r#"dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.PREFER_SETTINGS
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
        uri: format!("file://{}", root.join("build.gradle").display()),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-settings-prefer-settings-repositories-override-build-repositories.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gradle_plugin_management_repositories_do_not_resolve_regular_dependencies() {
    let root = temp_dir().join(format!(
        "versionlens-gradle-plugin-management-repositories-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
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
        uri: format!("file://{}", root.join("build.gradle").display()),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-plugin-management-repositories-do-not-resolve-regular-dependencies.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gradle_build_plugins_use_workspace_plugin_management_repositories() {
    let root = temp_dir().join(format!(
        "versionlens-gradle-build-plugin-management-repositories-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
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
        uri: format!("file://{}", root.join("build.gradle").display()),
        language_id: "groovy".to_owned(),
        text: package_file_fixture(
            "gradle-build-plugins-use-workspace-plugin-management-repositories.txt",
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
