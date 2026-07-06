use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use versionlens_parsers::DocumentInput;

use std::env;

use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
use versionlens_parsers::Ecosystem::Pub;

#[test]
fn hosted_dependencies_use_hosted_registry_url() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Pub,
                url: "https://pub.dev/api/packages".to_owned(),
            }],
            ..crate::default()
        },
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///pubspec.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("hosted-dependencies-use-hosted-registry-url.yaml"),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);
    let output = session.analyze_document(input);

    assert_eq!(
        output.dependencies[0].hosted_url.as_deref(),
        Some("https://pub.example.test/")
    );
    assert_eq!(
        output.dependencies[0].hosted_name.as_deref(),
        Some("hosted_alias")
    );
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://pub.example.test/api/packages/hosted_alias"]
    );
}

#[test]
fn docker_compose_explicit_registry_uses_oci_registry_url() {
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
        uri: "file:///compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("docker-compose-explicit-registry-uses-oci-registry-url.yaml"),
        workspace_root: None,
    };
    let dependencies = session.dependencies(&input);
    let output = session.analyze_document(input);

    assert_eq!(output.dependencies[0].name, "team/app");
    assert_eq!(
        output.dependencies[0].hosted_url.as_deref(),
        Some("registry.example.test")
    );
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://registry.example.test/v2/team/app/tags/list"]
    );
}

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

#[test]
fn sbt_resolvers_are_used_before_maven_central() {
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
        uri: "file:///build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: package_file_fixture("sbt-resolvers-are-used-before-maven-central.sbt"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn clojure_deps_edn_uses_maven_central_then_clojars() {
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
        uri: "file:///deps.edn".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture("clojure-deps-edn-uses-maven-central-then-clojars.edn"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "metosin:malli");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
            "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
        ]
    );
}

#[test]
fn leiningen_project_clj_uses_maven_central_then_clojars() {
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
        uri: "file:///project.clj".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture("leiningen-project-clj-uses-maven-central-then-clojars.clj"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[1].name, "metosin:malli");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec![
            "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
            "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
        ]
    );
}

include!("tests/more.rs");

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/registry/urls/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
