use versionlens_model::{Dependency, Ecosystem};
use versionlens_providers::{RegistryEndpoint, RegistryResponseKind};

use crate::VersionLensSession;
use crate::registry::{RegistryContext, RegistryEndpoints};
use versionlens_model::Ecosystem::{Dotnet, Go, Maven, Python};
use versionlens_model::ManifestKind::{ClojureDepsEdn, LeiningenProjectClj};

mod configured;
mod dotnet;
mod hosted;

use configured::{
    clojars_registry_endpoint, configured_registry_endpoints, default_registry_endpoint,
    extend_registry_endpoints, gradle_plugin_portal_registry_endpoint,
};
use hosted::hosted_registry_endpoints;

impl VersionLensSession {
    pub(crate) fn registry_endpoints_with_context(
        &self,
        dependency: &Dependency,
        context: &RegistryContext,
    ) -> RegistryEndpoints {
        if let Some(endpoints) = hosted_registry_endpoints(dependency) {
            return endpoints;
        }

        if context.go_proxy_disabled_for_dependency(dependency) {
            return vec![];
        }

        if dependency.ecosystem == Dotnet {
            if context.has_dotnet_registry_configuration() {
                return context
                    .dotnet_registry_urls(dependency)
                    .into_iter()
                    .map(RegistryEndpoint::ecosystem)
                    .collect();
            }

            let urls = self.dotnet_registry_urls();
            if !urls.is_empty() {
                return urls.into_iter().map(RegistryEndpoint::ecosystem).collect();
            }
        }

        let mut endpoints =
            configured_registry_endpoints(&self.config.providers.registry_urls, dependency);
        let is_clojure_maven_document = dependency.ecosystem == Maven
            && !context.maven_uses_mirror()
            && matches!(
                context.manifest_kind(),
                Some(ClojureDepsEdn | LeiningenProjectClj)
            );
        if is_clojure_maven_document {
            push_unique_endpoint(&mut endpoints, default_registry_endpoint(dependency));
            push_unique_endpoint(&mut endpoints, clojars_registry_endpoint(dependency));
        }
        if !python_dependency_has_named_source(dependency) {
            extend_registry_endpoints(&mut endpoints, &context.urls, dependency);
        }
        endpoints.extend(context.registry_endpoints(dependency));

        if dependency.ecosystem == Maven && !context.maven_uses_mirror() {
            if let Some(plugin_portal) = gradle_plugin_portal_registry_endpoint(dependency)
                && !endpoints
                    .iter()
                    .any(|endpoint| endpoint.url == plugin_portal.url)
            {
                endpoints.push(plugin_portal);
            }
            if !is_clojure_maven_document {
                push_unique_endpoint(&mut endpoints, default_registry_endpoint(dependency));
            }
        }

        let endpoints =
            if endpoints.is_empty() && context.default_registry_disabled(dependency.ecosystem) {
                endpoints
            } else if endpoints.is_empty() {
                vec![default_registry_endpoint(dependency)]
            } else {
                endpoints
            };

        go_module_proxy_endpoints_with_latest_fallback(dependency.ecosystem, endpoints)
    }
}

fn push_unique_endpoint(endpoints: &mut Vec<RegistryEndpoint>, endpoint: RegistryEndpoint) {
    if !endpoints
        .iter()
        .any(|existing| existing.url == endpoint.url)
    {
        endpoints.push(endpoint);
    }
}

fn go_module_proxy_endpoints_with_latest_fallback(
    ecosystem: Ecosystem,
    endpoints: RegistryEndpoints,
) -> RegistryEndpoints {
    if ecosystem != Go {
        return endpoints;
    }

    let mut expanded = vec![];
    for endpoint in endpoints {
        if endpoint.response_kind == RegistryResponseKind::GoModuleList
            && let Some(latest_url) = endpoint
                .url
                .strip_suffix("/@v/list")
                .map(|base| format!("{base}/@latest"))
        {
            expanded.push(endpoint);
            expanded.push(RegistryEndpoint::new(
                latest_url,
                RegistryResponseKind::GoModuleLatest,
            ));
        } else {
            expanded.push(endpoint);
        }
    }
    expanded
}

fn python_dependency_has_named_source(dependency: &Dependency) -> bool {
    dependency.ecosystem == Python
        && dependency
            .hosted_url
            .as_deref()
            .is_some_and(|url| !url.contains("://"))
}

#[cfg(test)]
mod tests {
    use crate::registry::RegistryContext;
    use std::env::temp_dir;
    use std::fs::create_dir_all;
    use std::fs::remove_dir_all;
    use std::fs::write;
    use std::process::id;
    use versionlens_model::DocumentInput;

    use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
    use versionlens_model::Ecosystem::*;

    const GRADLE_PLUGIN_MARKER_URLS: &[&str] = &[
        "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
        "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
    ];

    #[test]
    fn hosted_dependencies_use_hosted_registry_url() {
        let session = crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers: ProviderSettings {
                registry_urls: vec![RegistryUrlConfig {
                    ecosystem: Pub,
                    url: "https://pub.dev/api/packages".to_owned(),
                }],
                ..Default::default()
            },
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: false,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        });
        let input = registry_input(
            "file:///pubspec.yaml",
            "yaml",
            "hosted-dependencies-use-hosted-registry-url.yaml",
        );
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
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///compose.yaml",
            "yaml",
            "docker-compose-explicit-registry-uses-oci-registry-url.yaml",
        );
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
    fn github_actions_use_repository_identity_with_default_github_api_base() {
        let session = github_actions_session(ProviderSettings::default());
        let input = github_actions_input();
        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].ecosystem, GitHub);
        assert_eq!(dependencies[0].name, "actions/checkout");
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("actions/checkout")
        );
        assert_eq!(dependencies[0].hosted_url, None);
        assert_eq!(
            session.registry_urls(&dependencies[0]),
            vec!["https://api.github.com/repos/actions/checkout/tags"]
        );
    }

    #[test]
    fn github_actions_use_repository_identity_with_custom_github_api_base() {
        let session = github_actions_session(ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: GitHub,
                url: "https://github.example.test/api/repos/".to_owned(),
            }],
            ..Default::default()
        });
        let input = github_actions_input();
        let dependencies = session.dependencies(&input);

        assert_eq!(
            session.registry_urls(&dependencies[0]),
            vec!["https://github.example.test/api/repos/actions/checkout/tags"]
        );
    }

    fn github_actions_input() -> DocumentInput {
        registry_input(
            "file:///work/.github/workflows/ci.yml",
            "yaml",
            "github-actions-use-configured-registry-url.yaml",
        )
    }

    fn github_actions_session(providers: ProviderSettings) -> crate::VersionLensSession {
        crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![],
            providers,
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: false,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        })
    }
    #[test]
    fn gradle_plugin_markers_use_plugin_portal_before_maven_central() {
        assert_registry_urls_for_fixture(
            "file:///settings.gradle",
            "groovy",
            "gradle-plugin-markers-use-plugin-portal-before-maven-central.gradle",
            "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin",
            &[GRADLE_PLUGIN_MARKER_URLS[0], GRADLE_PLUGIN_MARKER_URLS[1]],
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
    fn gradle_version_catalog_version_referenced_by_plugin_uses_plugin_portal() {
        assert_registry_urls_for_fixture(
            "file:///gradle/libs.versions.toml",
            "toml",
            "gradle-version-catalog-version-referenced-by-plugin-uses-plugin-portal.versions.toml",
            "ksp",
            &[
                "https://plugins.gradle.org/m2/com/google/devtools/ksp/com.google.devtools.ksp.gradle.plugin/maven-metadata.xml",
                "https://repo.maven.apache.org/maven2/com/google/devtools/ksp/com.google.devtools.ksp.gradle.plugin/maven-metadata.xml",
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
            &[
                "https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml",
            ],
        );
    }

    #[test]
    fn gradle_version_catalog_libraries_use_workspace_settings_repositories() {
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-version-catalog-settings-repositories",
            settings: r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
    }
}
"#,
            relative_document: "gradle/libs.versions.toml",
            language: "toml",
            fixture: "gradle-version-catalog-libraries-use-workspace-settings-repositories.txt",
            dependency_name: "org.apache.commons:commons-lang3",
            expected_urls: &[
                "https://settings.example.test/releases/org/apache/commons/commons-lang3/maven-metadata.xml",
                "https://repo.maven.apache.org/maven2/org/apache/commons/commons-lang3/maven-metadata.xml",
            ],
        });
    }

    #[test]
    fn gradle_version_catalog_plugins_use_workspace_plugin_management_repositories() {
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-version-catalog-plugin-management-repositories",
            settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#,
            relative_document: "gradle/libs.versions.toml",
            language: "toml",
            fixture: "gradle-version-catalog-plugins-use-workspace-plugin-management-repositories.txt",
            dependency_name: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin",
            expected_urls: &[
                "https://plugins.example.test/releases/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
                "https://plugins.gradle.org/m2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
                "https://repo.maven.apache.org/maven2/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
            ],
        });
    }
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
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-settings-repositories",
            settings: r#"dependencyResolutionManagement {
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
        google()
    }
}
"#,
            relative_document: "build.gradle",
            language: "groovy",
            fixture: "gradle-build-uses-workspace-settings-repositories-before-maven-central.txt",
            dependency_name: "com.example:demo",
            expected_urls: &[
                "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
                "https://dl.google.com/dl/android/maven2/com/example/demo/maven-metadata.xml",
                "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            ],
        });
    }

    #[test]
    fn gradle_settings_prefer_settings_repositories_override_build_repositories() {
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-prefer-settings-repositories",
            settings: r#"dependencyResolutionManagement {
    repositoriesMode = RepositoriesMode.PREFER_SETTINGS
    repositories {
        maven {
            url = uri("https://settings.example.test/releases")
        }
    }
}
"#,
            relative_document: "build.gradle",
            language: "groovy",
            fixture: "gradle-settings-prefer-settings-repositories-override-build-repositories.txt",
            dependency_name: "com.example:demo",
            expected_urls: &[
                "https://settings.example.test/releases/com/example/demo/maven-metadata.xml",
                "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            ],
        });
    }

    #[test]
    fn gradle_plugin_management_repositories_do_not_resolve_regular_dependencies() {
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-plugin-management-repositories",
            settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#,
            relative_document: "build.gradle",
            language: "groovy",
            fixture: "gradle-plugin-management-repositories-do-not-resolve-regular-dependencies.txt",
            dependency_name: "com.example:demo",
            expected_urls: &[
                "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
            ],
        });
    }

    #[test]
    fn gradle_build_plugins_use_workspace_plugin_management_repositories() {
        assert_workspace_registry_urls(WorkspaceRegistryCase {
            root_name: "gradle-build-plugin-management-repositories",
            settings: r#"pluginManagement {
    repositories {
        maven {
            url = uri("https://plugins.example.test/releases")
        }
    }
}
"#,
            relative_document: "build.gradle",
            language: "groovy",
            fixture: "gradle-build-plugins-use-workspace-plugin-management-repositories.txt",
            dependency_name: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin",
            expected_urls: &[
                "https://plugins.example.test/releases/com/github/ben-manes/versions/com.github.ben-manes.versions.gradle.plugin/maven-metadata.xml",
                GRADLE_PLUGIN_MARKER_URLS[0],
                GRADLE_PLUGIN_MARKER_URLS[1],
            ],
        });
    }
    #[test]
    fn sbt_resolvers_are_used_before_maven_central() {
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///build.sbt",
            "scala",
            "sbt-resolvers-are-used-before-maven-central.sbt",
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

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
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///deps.edn",
            "clojure",
            "clojure-deps-edn-uses-maven-central-then-clojars.edn",
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

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
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///project.clj",
            "clojure",
            "leiningen-project-clj-uses-maven-central-then-clojars.clj",
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

        assert_eq!(dependencies[1].name, "metosin:malli");
        assert_eq!(
            session.registry_urls_with_context(&dependencies[1], &context),
            vec![
                "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
                "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
            ]
        );
    }
    #[test]
    fn clojure_deps_edn_mvn_repos_are_used_after_builtin_repositories() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///deps.edn".to_owned(),
            "clojure".to_owned(),
            package_file_fixture(
                "clojure-deps-edn-mvn-repos-are-used-after-builtin-repositories.edn",
            ),
            None,
        );
        assert_first_registry_urls(
            &session,
            &input,
            &[
                "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
                "https://repo.clojars.org/com/example/demo/maven-metadata.xml",
                "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
            ],
        );
    }

    #[test]
    fn leiningen_project_clj_repositories_are_used_after_builtin_repositories() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///project.clj".to_owned(),
            "clojure".to_owned(),
            package_file_fixture(
                "leiningen-project-clj-repositories-are-used-after-builtin-repositories.clj",
            ),
            None,
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

        assert_eq!(
            session.registry_urls_with_context(&dependencies[1], &context),
            vec![
                "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml",
                "https://repo.clojars.org/com/example/demo/maven-metadata.xml",
                "https://maven.example.test/releases/com/example/demo/maven-metadata.xml"
            ]
        );
    }

    #[test]
    fn mix_hex_project_api_url_overrides_default_hex_registry_url() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///mix.exs".to_owned(),
            "elixir".to_owned(),
            package_file_fixture("mix-hex-project-api-url-overrides-default-hex-registry-url.exs"),
            None,
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://hex.example.test/api/packages/plug"],
        );
    }

    #[test]
    fn mix_hex_env_api_url_takes_precedence_over_project_api_url() {
        let root = workspace_with_env("hex-env", "HEX_API_URL=https://hex.env.example.test/api\n");

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", root.join("mix.exs").display()),
            "elixir".to_owned(),
            package_file_fixture("mix-hex-env-api-url-takes-precedence-over-project-api-url.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://hex.env.example.test/api/packages/plug"],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn hex_env_api_url_configures_rebar_and_gleam_registry_urls() {
        let root = workspace_with_env(
            "hex-env-beam",
            "HEX_API_URL=https://hex.env.example.test/api\n",
        );

        let session = crate::test_support::test_session(false);
        let rebar_input = DocumentInput::new(
            format!("file://{}", root.join("rebar.config").display()),
            "erlang".to_owned(),
            package_file_fixture("hex-env-api-url-configures-rebar-and-gleam-registry-urls.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        let gleam_input = DocumentInput::new(
            format!("file://{}", root.join("gleam.toml").display()),
            "toml".to_owned(),
            package_file_fixture("hex-env-api-url-configures-rebar-and-gleam-registry-urls-2.txt"),
            Some(root.to_string_lossy().into_owned()),
        );

        let rebar_context = RegistryContext::from_document(&rebar_input);
        let gleam_context = RegistryContext::from_document(&gleam_input);
        let rebar_dependencies = session.dependencies(&rebar_input);
        let gleam_dependencies = session.dependencies(&gleam_input);

        assert_eq!(
            session.registry_urls_with_context(&rebar_dependencies[0], &rebar_context),
            vec!["https://hex.env.example.test/api/packages/cowboy"]
        );
        assert_eq!(
            session.registry_urls_with_context(&gleam_dependencies[0], &gleam_context),
            vec!["https://hex.env.example.test/api/packages/gleam_stdlib"]
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn rebar_hex_cdn_env_configures_registry_url() {
        let root = temp_dir().join(format!("versionlens-hex-cdn-env-{}", id()));
        create_dir_all(&root).unwrap();
        write(root.join(".env"), "HEX_CDN=https://repo.example.test\n").unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", root.join("rebar.config").display()),
            "erlang".to_owned(),
            package_file_fixture("rebar-hex-cdn-env-configures-registry-url.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://repo.example.test/api/packages/cowboy"],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn rebar_packages_cdn_configures_registry_url() {
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///rebar.config",
            "erlang",
            "rebar-packages-cdn-configures-registry-url.config",
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://repo.project.example.test/api/packages/cowboy"],
        );
    }

    #[test]
    fn deno_jsr_import_aliases_use_specifier_package_for_registry_urls() {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture(
                "deno-jsr-import-aliases-use-specifier-package-for-registry-urls.json",
            ),
            None,
        );
        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies[0].name, "luca");
        assert_eq!(dependencies[0].hosted_name.as_deref(), Some("@luca/cases"));
        assert_eq!(
            session.registry_urls(&dependencies[0]),
            vec!["https://jsr.io/@luca/cases/meta.json"]
        );
    }
    fn workspace_with_env(name: &str, contents: &str) -> std::path::PathBuf {
        let root = temp_dir().join(format!("versionlens-{name}-{}", id()));
        create_dir_all(&root).unwrap();
        write(root.join(".env"), contents).unwrap();
        root
    }
    use versionlens_model::Ecosystem::Dotnet;

    fn write_nuget_config(root: &std::path::Path, package_source_policy: &str) {
        create_dir_all(root).unwrap();
        write(
        root.join("NuGet.config"),
        format!(
            "<configuration>\n  <packageSources>\n    <add key=\"nuget.org\" value=\"https://api.nuget.org/v3/index.json\" />\n    <add key=\"private\" value=\"https://nuget.example.test/v3/index.json\" />\n  </packageSources>\n{package_source_policy}\n</configuration>\n"
        ),
    )
    .unwrap();
    }

    const DISABLED_NUGET_SOURCE_POLICY: &str = "  <disabledPackageSources>\n    <add key=\"nuget.org\" value=\"true\" />\n  </disabledPackageSources>";

    const MAPPED_NUGET_SOURCE_POLICY: &str = "  <packageSourceMapping>\n    <packageSource key=\"nuget.org\">\n      <package pattern=\"Newtonsoft.*\" />\n    </packageSource>\n    <packageSource key=\"private\">\n      <package pattern=\"Contoso.*\" />\n    </packageSource>\n  </packageSourceMapping>";
    #[test]
    fn dotnet_sources_are_service_indexes_not_package_urls() {
        let session = crate::test_support::session_with_provider_settings(
            ProviderSettings {
                registry_urls: vec![RegistryUrlConfig {
                    ecosystem: Dotnet,
                    url: "https://nuget.example.test/v3/index.json".to_owned(),
                }],
                ..Default::default()
            },
            false,
        );
        let input = registry_input(
            "file:///app.csproj",
            "xml",
            "dotnet-sources-are-service-indexes-not-package-urls.csproj",
        );
        let dependencies = session.dependencies(&input);

        let urls = session.registry_urls(&dependencies[0]);
        assert_eq!(urls[0], "https://nuget.example.test/v3/index.json");
        assert!(
            urls.iter()
                .all(|url| !url.contains("newtonsoft.json/index.json"))
        );
    }

    #[test]
    fn dotnet_documents_use_workspace_nuget_config_sources() {
        let root = temp_dir().join(format!("versionlens-nuget-config-{}", id()));
        write_nuget_config(&root, DISABLED_NUGET_SOURCE_POLICY);

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", root.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture("dotnet-documents-use-workspace-nuget-config-sources.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://nuget.example.test/v3/index.json"],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn paket_dependencies_use_declared_source_urls() {
        let session = crate::test_support::test_session(false);
        let input = registry_input(
            "file:///paket.dependencies",
            "plaintext",
            "paket-dependencies-use-declared-source-urls.dependencies",
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://nuget.example.test/v3/index.json"],
        );
    }

    #[test]
    fn dotnet_child_nuget_config_clear_removes_workspace_sources() {
        let root = temp_dir().join(format!("versionlens-nuget-config-clear-{}", id()));
        let app = root.join("src");
        create_dir_all(&app).unwrap();
        write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
        write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><clear /><add key="child" value="https://child.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", app.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture("dotnet-child-nuget-config-clear-removes-workspace-sources.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &["https://child.example.test/v3/index.json"],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn dotnet_child_nuget_config_remove_does_not_delete_inherited_cli_sources() {
        let root = temp_dir().join(format!("versionlens-nuget-config-remove-{}", id()));
        let app = root.join("src");
        create_dir_all(&app).unwrap();
        write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /><add key="keep" value="https://keep.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
        write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><remove key="root" /></packageSources></configuration>"#,
    )
    .unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", app.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture(
                "dotnet-child-nuget-config-remove-does-not-delete-inherited-cli-sources.txt",
            ),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &[
                "https://root.example.test/v3/index.json",
                "https://keep.example.test/v3/index.json",
            ],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn dotnet_nuget_config_reads_intermediate_ancestors_nearest_first() {
        let root = temp_dir().join(format!("versionlens-nuget-config-ancestors-{}", id()));
        let src = root.join("src");
        let app = src.join("app");
        create_dir_all(&app).unwrap();
        write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
        write(
        src.join("NuGet.config"),
        r#"<configuration><packageSources><add key="src" value="https://src.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
        write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><add key="app" value="https://app.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", app.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture(
                "dotnet-nuget-config-reads-intermediate-ancestors-nearest-first.txt",
            ),
            Some(root.to_string_lossy().into_owned()),
        );
        assert_first_registry_urls(
            &session,
            &input,
            &[
                "https://app.example.test/v3/index.json",
                "https://src.example.test/v3/index.json",
                "https://root.example.test/v3/index.json",
            ],
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn dotnet_nuget_config_ignores_local_file_sources_for_suggestions() {
        let root = temp_dir().join(format!("versionlens-nuget-config-local-source-{}", id()));
        create_dir_all(root.join("packages")).unwrap();
        write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="local" value="./packages" /></packageSources></configuration>"#,
    )
    .unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", root.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture(
                "dotnet-nuget-config-ignores-local-file-sources-for-suggestions.txt",
            ),
            Some(root.to_string_lossy().into_owned()),
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

        assert_eq!(
            session
                .registry_urls_with_context(&dependencies[0], &context)
                .len(),
            0
        );

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn dotnet_nuget_config_supplies_request_scoped_auth_headers() {
        let root = temp_dir().join(format!("versionlens-nuget-config-auth-{}", id()));
        create_dir_all(&root).unwrap();
        write(
            root.join("NuGet.config"),
            r#"
<configuration>
  <packageSources>
    <add key="private" value="https://nuget.example.test/v3/index.json" />
  </packageSources>
  <packageSourceCredentials>
    <private>
      <add key="Username" value="user" />
      <add key="ClearTextPassword" value="pass" />
    </private>
  </packageSourceCredentials>
</configuration>
"#,
        )
        .unwrap();

        let input = DocumentInput::new(
            format!("file://{}", root.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture("dotnet-nuget-config-supplies-request-scoped-auth-headers.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        let context = RegistryContext::from_document(&input);
        let service_index_headers =
            context.auth_headers_for_url(Dotnet, "https://nuget.example.test/v3/index.json");
        let package_headers = context.auth_headers_for_url(
            Dotnet,
            "https://nuget.example.test/v3-flatcontainer/newtonsoft.json/index.json",
        );
        let other_headers =
            context.auth_headers_for_url(Dotnet, "https://other.example.test/v3/index.json");

        assert_eq!(service_index_headers.len(), 1);
        assert_eq!(service_index_headers[0].value, "Basic dXNlcjpwYXNz");
        assert_eq!(package_headers.len(), 1);
        assert_eq!(package_headers[0].value, "Basic dXNlcjpwYXNz");
        assert_eq!(other_headers.len(), 0);

        remove_dir_all(root).unwrap();
    }

    #[test]
    fn dotnet_nuget_config_package_source_mapping_filters_sources() {
        let root = temp_dir().join(format!("versionlens-nuget-config-mapping-{}", id()));
        write_nuget_config(&root, MAPPED_NUGET_SOURCE_POLICY);

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", root.join("app.csproj").display()),
            "xml".to_owned(),
            package_file_fixture("dotnet-nuget-config-package-source-mapping-filters-sources.txt"),
            Some(root.to_string_lossy().into_owned()),
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);
        for (index, expected) in [
            (0, "https://api.nuget.org/v3/index.json"),
            (1, "https://nuget.example.test/v3/index.json"),
        ] {
            assert_eq!(
                session.registry_urls_with_context(&dependencies[index], &context),
                vec![expected]
            );
        }

        remove_dir_all(root).unwrap();
    }
    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/core-scenarios/registry/urls/tests", name)
    }

    fn registry_input(uri: &str, language: &str, fixture: &str) -> DocumentInput {
        DocumentInput::new(
            uri.to_owned(),
            language.to_owned(),
            package_file_fixture(fixture),
            None,
        )
    }

    fn registry_context_and_dependencies(
        session: &crate::VersionLensSession,
        input: &DocumentInput,
    ) -> (RegistryContext, Vec<versionlens_model::Dependency>) {
        crate::test_support::registry_context_and_dependencies(session, input)
    }

    fn assert_first_registry_urls(
        session: &crate::VersionLensSession,
        input: &DocumentInput,
        expected: &[&str],
    ) {
        let (context, dependencies) = registry_context_and_dependencies(session, input);
        crate::test_support::assert_registry_urls(session, &context, &dependencies[0], expected);
    }

    fn assert_registry_urls_for_fixture(
        uri: &str,
        language: &str,
        fixture: &str,
        dependency_name: &str,
        expected_urls: &[&str],
    ) {
        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            uri.to_owned(),
            language.to_owned(),
            package_file_fixture(fixture),
            None,
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

        assert_eq!(dependencies[0].name, dependency_name);
        crate::test_support::assert_registry_urls(
            &session,
            &context,
            &dependencies[0],
            expected_urls,
        );
    }

    #[derive(Clone, Copy)]
    struct WorkspaceRegistryCase<'a> {
        root_name: &'a str,
        settings: &'a str,
        relative_document: &'a str,
        language: &'a str,
        fixture: &'a str,
        dependency_name: &'a str,
        expected_urls: &'a [&'a str],
    }

    fn assert_workspace_registry_urls(case: WorkspaceRegistryCase<'_>) {
        let root = temp_dir().join(format!("versionlens-{}-{}", case.root_name, id()));
        let document = root.join(case.relative_document);
        if let Some(parent) = document.parent() {
            create_dir_all(parent).unwrap();
        }
        write(root.join("settings.gradle"), case.settings).unwrap();

        let session = crate::test_support::test_session(false);
        let input = DocumentInput::new(
            format!("file://{}", document.display()),
            case.language.to_owned(),
            package_file_fixture(case.fixture),
            Some(root.to_string_lossy().into_owned()),
        );
        let (context, dependencies) = registry_context_and_dependencies(&session, &input);

        assert_eq!(dependencies[0].name, case.dependency_name);
        assert_eq!(
            session.registry_urls_with_context(&dependencies[0], &context),
            case.expected_urls
                .iter()
                .map(|url| (*url).to_owned())
                .collect::<Vec<_>>()
        );

        remove_dir_all(root).unwrap();
    }

    impl crate::VersionLensSession {
        pub(crate) fn registry_urls(
            &self,
            dependency: &versionlens_model::Dependency,
        ) -> Vec<String> {
            self.registry_urls_with_context(dependency, &RegistryContext::default())
        }

        pub(crate) fn registry_urls_with_context(
            &self,
            dependency: &versionlens_model::Dependency,
            context: &RegistryContext,
        ) -> Vec<String> {
            self.registry_endpoints_with_context(dependency, context)
                .into_iter()
                .map(|endpoint| endpoint.url)
                .collect()
        }
    }
}
