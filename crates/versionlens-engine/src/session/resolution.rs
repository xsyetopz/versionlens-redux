use versionlens_model::{Dependency, DocumentInput, ProjectVersionBump};

use super::operation::OperationContext;
use crate::RegistryResponseInput;
use crate::registry::RegistryContext;

pub(crate) struct ResolutionRequest<'a> {
    pub(super) input: &'a DocumentInput,
    pub(super) dependencies: Vec<Dependency>,
    pub(super) document_uri: &'a str,
    pub(super) responses: &'a [RegistryResponseInput],
    pub(super) project_bump: Option<ProjectVersionBump>,
    pub(super) context: &'a RegistryContext,
    pub(super) operation: &'a OperationContext,
}

mod dependency;
mod latest;
mod parallel;
mod runtime;

#[cfg(test)]
mod tests {
    use versionlens_model::Ecosystem;
    use versionlens_model::Ecosystem::*;
    use versionlens_model::{Dependency, DocumentInput};
    use versionlens_parsers::parse_document;

    use crate::RegistryResponseInput;
    use crate::contract::ResolveDocumentOutput;
    use crate::registry::RegistryContext;

    use crate::{ProviderSettings, RegistryUrlConfig, VersionLensSession};
    use std::env::temp_dir;
    use std::fs::{create_dir_all, remove_dir_all, write};
    use std::process::id;

    fn standard_session() -> VersionLensSession {
        crate::test_support::test_session(true)
    }

    fn session_without_vulnerabilities() -> VersionLensSession {
        crate::test_support::test_session(false)
    }

    mod dotnet {
        use super::*;

        #[tokio::test]
        async fn dotnet_local_nuget_source_does_not_resolve_versions_from_package_folder() {
            let root = temp_dir().join(format!("versionlens-dotnet-local-{}", id()));
            let source = root.join("packages");
            let package_dir = source.join("newtonsoft.json");
            create_dir_all(package_dir.join("13.0.1")).unwrap();
            create_dir_all(package_dir.join("13.0.3")).unwrap();
            write(
                root.join("NuGet.config"),
                r#"<configuration>
  <packageSources>
    <clear />
    <add key="local" value="./packages" />
  </packageSources>
</configuration>"#,
            )
            .unwrap();

            let output = resolve_local_source(
                &root,
                "dotnet-local-nuget-source-does-not-resolve-versions-from-package-folder.txt",
            )
            .await;

            crate::test_support::assert_suggestion_without_edits(&output, 0, "unresolved", None);

            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn dotnet_local_nuget_source_does_not_resolve_flat_nupkg_files() {
            let root = temp_dir().join(format!("versionlens-dotnet-flat-local-{}", id()));
            let source = root.join("packages");
            create_dir_all(&source).unwrap();
            write(source.join("Newtonsoft.Json.13.0.3.nupkg"), "").unwrap();
            write(source.join("Newtonsoft.Json.13.0.1.nupkg"), "").unwrap();
            write(
        root.join("NuGet.config"),
        format!(
            r#"<configuration><packageSources><add key="local" value="file://{}" /></packageSources></configuration>"#,
            source.display()
        ),
    )
    .unwrap();

            let output = resolve_local_source(
                &root,
                "dotnet-local-nuget-source-does-not-resolve-flat-nupkg-files.txt",
            )
            .await;

            crate::test_support::assert_suggestion_without_edits(&output, 0, "unresolved", None);

            remove_dir_all(root).unwrap();
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/dotnet", name)
        }

        async fn resolve_local_source(
            root: &std::path::Path,
            fixture: &str,
        ) -> ResolveDocumentOutput {
            standard_session()
                .resolve_document(DocumentInput::new(
                    format!("file://{}", root.join("app.csproj").display()),
                    "xml".to_owned(),
                    package_file_fixture(fixture),
                    Some(root.to_string_lossy().into_owned()),
                ))
                .await
        }
    }
    mod error {
        use super::*;

        #[tokio::test]
        async fn registry_response_without_latest_creates_no_match() {
            let session = session_without_vulnerabilities();
            let input = DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("registry-response-without-latest-creates-no-match.json"),
                None,
            );
            let response = RegistryResponseInput::new(
                "left-pad".to_owned(),
                Npm,
                r#"{"versions":{}}"#.to_owned(),
            );

            let output = session
                .resolve_document_with_responses(input, &[response])
                .await;

            assert_eq!(output.suggestions[0].status, "noMatch");
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn npm_error_registry_response_creates_error_suggestion() {
            let session = session_without_vulnerabilities();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "npm-error-registry-response-creates-error-suggestion.json",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        r#"{"status":"E404"}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "error",
                Some("not found"),
            );
        }

        #[tokio::test]
        async fn hex_error_registry_response_creates_error_suggestion() {
            let session = session_without_vulnerabilities();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///mix.exs".to_owned(),
                        "elixir".to_owned(),
                        package_file_fixture(
                            "hex-error-registry-response-creates-error-suggestion.exs",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "plug".to_owned(),
                        Hex,
                        r#"{"status":404}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "error",
                Some("not found"),
            );
        }

        #[tokio::test]
        async fn hex_rate_limited_registry_response_creates_rate_limited_suggestion() {
            let session = session_without_vulnerabilities();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///mix.exs".to_owned(),
                        "elixir".to_owned(),
                        package_file_fixture(
                            "hex-rate-limited-registry-response-creates-error-suggestion.exs",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "plug".to_owned(),
                        Hex,
                        r#"{"status":429}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "rateLimited");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("rate limited")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn registry_responses_try_next_matching_body_when_first_has_no_latest() {
            let session = session_without_vulnerabilities();

            let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "registry-responses-try-next-matching-body-when-first-has-no-latest.json",
                ),
                None,
            ),
            &[
                RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{"versions":{}}"#.to_owned(),
                ),
                RegistryResponseInput::new(
                    "left-pad".to_owned(),
                    Npm,
                    r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
                ),
            ],
        )
        .await;

            assert_update(&output, "1.1.0");
        }

        #[tokio::test]
        async fn unauthorized_registry_response_reports_auth_request_urls() {
            let (output, base_url) =
                crate::test_support::with_unauthorized_server(|base_url| async move {
                    let session = crate::test_support::session_with_provider_settings(
                        ProviderSettings {
                            registry_urls: vec![RegistryUrlConfig {
                                ecosystem: Npm,
                                url: base_url.clone(),
                            }],
                            ..Default::default()
                        },
                        false,
                    );
                    (
                        session
                            .resolve_document(DocumentInput::new(
                                "file:///package.json".to_owned(),
                                "json".to_owned(),
                                package_file_fixture(
                                    "unauthorized-registry-response-reports-auth-request-urls.json",
                                ),
                                None,
                            ))
                            .await,
                        base_url,
                    )
                })
                .await;

            assert_eq!(output.authorization_required_count, 1);
            assert_eq!(output.authorization_required_requests.len(), 1);
            assert_eq!(output.authorization_required_requests[0].auth_url, base_url);
            assert_eq!(
                output.authorization_required_requests[0].request_url,
                format!("{base_url}/left-pad")
            );
        }

        #[tokio::test]
        async fn registry_http_status_error_names_the_status_and_host() {
            let (base_url, server) = github::github_api_server(vec![(404, "{}")]);
            let host = base_url
                .trim_start_matches("http://")
                .split('/')
                .next()
                .unwrap()
                .to_owned();
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings {
                    registry_urls: vec![RegistryUrlConfig {
                        ecosystem: Npm,
                        url: base_url.clone(),
                    }],
                    ..Default::default()
                },
                false,
            );

            let output = session
                .resolve_document(DocumentInput::new(
                    "file:///package.json".to_owned(),
                    "json".to_owned(),
                    package_file_fixture(
                        "unauthorized-registry-response-reports-auth-request-urls.json",
                    ),
                    None,
                ))
                .await;

            assert_eq!(server.join().unwrap(), ["/repos/left-pad"]);
            let expected = format!("not found (404 {host})");
            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "error",
                Some(expected.as_str()),
            );
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/error", name)
        }
        use super::assert_update;
    }
    mod fixed {
        use super::{DocumentInput, standard_session};
        use crate::RegistryResponseInput;
        use crate::test_support::{assert_no_edits, assert_suggestion};
        use crate::test_support::{file_uri, local_test_root};
        use std::env::temp_dir;
        use std::fs::create_dir_all;
        use std::fs::remove_dir_all;
        use std::fs::write;
        use std::process::id;

        use versionlens_model::Ecosystem::*;

        mod dotnet {
            use super::{DocumentInput, standard_session};
            use crate::RegistryResponseInput;
            use versionlens_model::Ecosystem::Dotnet;

            #[tokio::test]
            async fn dotnet_nuget_versions_return_registry_suggestions() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/app.csproj".to_owned(),
                            "xml".to_owned(),
                            package_file_fixture(
                                "dotnet-nuget-versions-return-registry-suggestions.csproj",
                            ),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "Microsoft.Extensions.Logging".to_owned(),
                            Dotnet,
                            r#"{"versions":["5.0.0","5.0.1"]}"#.to_owned(),
                        )],
                    )
                    .await;

                assert_eq!(
                    output.suggestions[0].dependency.name,
                    "Microsoft.Extensions.Logging"
                );
                assert_eq!(output.suggestions[0].dependency.requirement, "5.0.0");
                crate::test_support::assert_suggestion(&output, 0, "fixed", Some("5.0.0"));
            }

            #[tokio::test]
            async fn dotnet_four_segment_versions_return_empty_suggestions_from_registry() {
                let session = standard_session();

                let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///repo/app.csproj".to_owned(),
                "xml".to_owned(),
                package_file_fixture(
                    "dotnet-four-segment-versions-return-empty-suggestions-from-registry.csproj",
                ),
                None,
            ),
            &[RegistryResponseInput::new(
                "Test.Package".to_owned(),
                Dotnet,
                r#"{"versions":["1.2.4"]}"#.to_owned(),
            )],
        )
        .await;

                assert_eq!(output.suggestions.len(), 0);
                assert_eq!(output.edits.len(), 0);
            }

            #[tokio::test]
            async fn dotnet_invalid_versions_return_no_match_from_registry() {
                let session = standard_session();
                let input = DocumentInput::new(
                    "file:///repo/app.csproj".to_owned(),
                    "xml".to_owned(),
                    package_file_fixture(
                        "dotnet-invalid-versions-return-no-match-from-registry.csproj",
                    ),
                    None,
                );

                let output = session
                    .resolve_document_with_responses(
                        input.clone(),
                        &[RegistryResponseInput::new(
                            "Test.Package".to_owned(),
                            Dotnet,
                            r#"{"versions":["1.2.4","1.3.0","2.0.0","2.1.0-beta.1"]}"#.to_owned(),
                        )],
                    )
                    .await;
                let analysis = session.analyze_document(input);
                let (titles, arguments) =
                    crate::session::resolution::tests::lens_titles_and_arguments(&analysis);

                assert_eq!(output.suggestions[0].status, "noMatch");
                assert_eq!(output.suggestions[0].dependency.name, "Test.Package");
                assert_eq!(output.suggestions[0].latest, None);
                assert_eq!(output.edits.len(), 0);
                assert_eq!(titles, ["⚪ no match", "↑  latest 2.0.0"]);
                assert_eq!(arguments, [vec!["update", "2.0.0"]]);
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/fixed/dotnet",
                    name,
                )
            }
        }
        mod npm {
            use super::{DocumentInput, standard_session};
            use crate::RegistryResponseInput;
            use versionlens_model::Ecosystem::Npm;

            #[tokio::test]
            async fn npm_bare_relative_paths_are_invalid_versions() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/project/package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture("bare-relative-paths-are-invalid-versions.json"),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "filepackage".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                        )],
                    )
                    .await;

                assert_eq!(output.suggestions[0].status, "error");
                assert_eq!(
                    output.suggestions[0].latest.as_deref(),
                    Some("invalid version")
                );
                assert_eq!(output.edits.len(), 0);
            }

            #[tokio::test]
            async fn npm_github_url_without_ref_is_fixed_git_dependency() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/project/package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture(
                                "github-url-without-ref-is-fixed-git-dependency.json",
                            ),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "owner/url".to_owned(),
                            Npm,
                            r#"[{"sha":"abcdef1234567890"}]"#.to_owned(),
                        )],
                    )
                    .await;

                crate::test_support::assert_fixed_git_repository(&output);
            }

            #[tokio::test]
            async fn npm_override_reference_dependencies_ignore_registry_updates() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/project/package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture(
                                "override-reference-dependencies-ignore-registry-updates.json",
                            ),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "bar".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                        )],
                    )
                    .await;

                assert_eq!(output.suggestions[0].status, "fixed");
                assert_eq!(
                    output.suggestions[0].latest.as_deref(),
                    Some("override reference")
                );
                assert_eq!(output.edits.len(), 0);
            }

            #[tokio::test]
            async fn npm_portal_dependencies_resolve_as_local_paths_without_registry_updates() {
                let session = standard_session();

                let output = session
        .resolve_document_with_responses(
            DocumentInput::new(
                "file:///repo/project/package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "portal-dependencies-resolve-as-local-paths-without-registry-updates.json",
                ),
                None,
            ),
            &[RegistryResponseInput::new(
                "local".to_owned(),
                Npm,
                r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
            )],
        )
        .await;

                crate::test_support::assert_suggestion_without_edits(
                    &output,
                    0,
                    "directoryNotFound",
                    Some("../local"),
                );
            }

            #[tokio::test]
            async fn npm_exec_dependencies_are_not_registry_updates() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/project/package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture("exec-dependencies-are-not-registry-updates.json"),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "generated".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                        )],
                    )
                    .await;

                crate::test_support::assert_suggestion_without_edits(
                    &output,
                    0,
                    "notSupported",
                    None,
                );
            }

            #[tokio::test]
            async fn npm_patch_dependencies_are_not_registry_updates() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///repo/project/package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture(
                                "patch-dependencies-are-not-registry-updates.json",
                            ),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "@types/react".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"19.0.0"}}"#.to_owned(),
                        )],
                    )
                    .await;

                crate::test_support::assert_suggestion_without_edits(
                    &output,
                    0,
                    "notSupported",
                    None,
                );
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/fixed/npm",
                    name,
                )
            }
        }
        mod registry_sources {
            use super::{DocumentInput, standard_session};
            use std::env::temp_dir;
            use std::fs::create_dir_all;
            use std::fs::remove_dir_all;
            use std::fs::write;
            use std::process::id;

            #[test]
            fn requirements_index_urls_override_python_registry_urls() {
                let input = DocumentInput::new(
                    "file:///repo/requirements.txt".to_owned(),
                    "pip-requirements".to_owned(),
                    package_file_fixture(
                        "requirements-index-urls-override-python-registry-urls.txt",
                    ),
                    None,
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &[
                        "https://primary.example.test/simple/requests/",
                        "https://extra.example.test/simple/requests/",
                    ],
                );
            }

            #[test]
            fn pipfile_sources_override_python_registry_urls() {
                let input = DocumentInput::new(
                    "file:///repo/Pipfile".to_owned(),
                    "toml".to_owned(),
                    package_file_fixture("pipfile-sources-override-python-registry-urls.Pipfile"),
                    None,
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &["https://pypi.example.test/simple/requests/"],
                );
            }

            #[test]
            fn python_documents_use_workspace_pip_conf_registry_urls() {
                let root = temp_dir().join(format!("versionlens-pip-conf-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join("pip.conf"),
        "[global]\nindex-url = https://primary.example.test/simple/\nextra-index-url = https://extra.example.test/simple\n",
    )
    .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("requirements.txt").display()),
                    "pip-requirements".to_owned(),
                    package_file_fixture(
                        "python-documents-use-workspace-pip-conf-registry-urls.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &[
                        "https://primary.example.test/simple/requests/",
                        "https://extra.example.test/simple/requests/",
                    ],
                );

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn python_documents_use_pip_environment_registry_urls() {
                let root = temp_dir().join(format!("versionlens-pip-env-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join(".env"),
        "PIP_INDEX_URL=https://env-primary.example.test/simple/\nPIP_EXTRA_INDEX_URL=https://env-extra-one.example.test/simple https://env-extra-two.example.test/simple/\n",
    )
    .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("requirements.txt").display()),
                    "pip-requirements".to_owned(),
                    package_file_fixture("python-documents-use-pip-environment-registry-urls.txt"),
                    Some(root.to_string_lossy().into_owned()),
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &[
                        "https://env-primary.example.test/simple/requests/",
                        "https://env-extra-one.example.test/simple/requests/",
                        "https://env-extra-two.example.test/simple/requests/",
                    ],
                );

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn gemfile_source_urls_override_ruby_registry_urls() {
                let input = DocumentInput::new(
                    "file:///repo/Gemfile".to_owned(),
                    "ruby".to_owned(),
                    package_file_fixture("gemfile-source-urls-override-ruby-registry-urlsGemfile"),
                    None,
                );
                assert_registry_urls(
                    &input,
                    "rails",
                    &["https://gems.example.test/api/v1/versions/rails.json"],
                );
            }

            #[test]
            fn gemfile_source_blocks_override_dependency_registry_url() {
                let input = DocumentInput::new(
                    "file:///repo/Gemfile".to_owned(),
                    "ruby".to_owned(),
                    package_file_fixture(
                        "gemfile-source-blocks-override-dependency-registry-urlGemfile",
                    ),
                    None,
                );
                assert_private_gem_registry_url(&input);
            }

            #[test]
            fn gemfile_source_option_overrides_dependency_registry_url() {
                let input = DocumentInput::new(
                    "file:///repo/Gemfile".to_owned(),
                    "ruby".to_owned(),
                    package_file_fixture(
                        "gemfile-source-option-overrides-dependency-registry-urlGemfile",
                    ),
                    None,
                );
                assert_private_gem_registry_url(&input);
            }

            fn assert_private_gem_registry_url(input: &DocumentInput) {
                let session = standard_session();
                let dependencies = session.dependencies(input);
                assert_eq!(dependencies[0].name, "private_gem");
                assert_eq!(
                    session.registry_urls(&dependencies[0]),
                    vec!["https://private.gems.example.test/api/v1/versions/private_gem.json"]
                );
            }

            #[test]
            fn pyproject_uv_indexes_override_python_registry_urls() {
                let input = DocumentInput::new(
                    "file:///repo/pyproject.toml".to_owned(),
                    "toml".to_owned(),
                    package_file_fixture("uv-indexes-override-python-registry-urls.toml"),
                    None,
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &[
                        "https://primary.example.test/simple/requests/",
                        "https://extra.example.test/simple/requests/",
                        "https://private.example.test/simple/requests/",
                    ],
                );
            }

            #[test]
            fn pyproject_poetry_sources_override_python_registry_urls() {
                let input = DocumentInput::new(
                    "file:///repo/pyproject.toml".to_owned(),
                    "toml".to_owned(),
                    package_file_fixture("poetry-sources-override-python-registry-urls.toml"),
                    None,
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &["https://poetry.example.test/simple/requests/"],
                );
            }

            #[test]
            fn pyproject_poetry_dependency_source_overrides_python_registry_url() {
                let input = DocumentInput::new(
                    "file:///repo/pyproject.toml".to_owned(),
                    "toml".to_owned(),
                    package_file_fixture(
                        "poetry-dependency-source-overrides-python-registry-url.toml",
                    ),
                    None,
                );
                let dependencies = standard_session().dependencies(&input);
                let session = standard_session();

                assert_eq!(dependencies[1].name, "private");
                assert_eq!(dependencies[1].hosted_url, Some("private".to_owned()));
                let context = crate::registry::RegistryContext::from_document(&input);
                assert_eq!(
                    session.registry_urls_with_context(&dependencies[1], &context),
                    vec!["https://private.example.test/simple/private/"]
                );
            }

            #[test]
            fn python_documents_use_workspace_uv_toml_registry_urls() {
                let root = temp_dir().join(format!("versionlens-uv-toml-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join("uv.toml"),
        "index-url = 'https://primary.example.test/simple/'\nextra-index-url = ['https://extra.example.test/simple']\n[[index]]\nname = 'private'\nurl = 'https://private.example.test/simple/'\n",
    )
    .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("pyproject.toml").display()),
                    "toml".to_owned(),
                    package_file_fixture(
                        "python-documents-use-workspace-uv-toml-registry-urls.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                assert_registry_urls(
                    &input,
                    "requests",
                    &[
                        "https://primary.example.test/simple/requests/",
                        "https://extra.example.test/simple/requests/",
                        "https://private.example.test/simple/requests/",
                    ],
                );

                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn podfile_external_and_latest_dependencies_resolve_as_fixed_without_registry_updates()
             {
                let input = DocumentInput::new(
                    "file:///repo/Podfile".to_owned(),
                    "ruby".to_owned(),
                    package_file_fixture(
                        "podfile-external-and-latest-dependencies-resolve-as-fixed-without-registry-updatesPodfile",
                    ),
                    None,
                );
                let output = standard_session().resolve_document(input).await;

                assert_eq!(output.suggestions.len(), 4);
                assert_eq!(output.edits.len(), 0);
                assert_eq!(output.suggestions[0].status, "fixed");
                assert_eq!(
                    output.suggestions[0].latest.as_deref(),
                    Some("latest version")
                );
                assert_eq!(output.suggestions[1].latest.as_deref(), Some("local pod"));
                assert_eq!(
                    output.suggestions[2].latest.as_deref(),
                    Some("git repository")
                );
                assert_eq!(
                    output.suggestions[3].latest.as_deref(),
                    Some("podspec source")
                );
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/fixed/registry_sources",
                    name,
                )
            }

            fn assert_registry_urls(
                input: &DocumentInput,
                dependency_name: &str,
                expected: &[&str],
            ) {
                let (session, context, dependencies) =
                    crate::session::resolution::tests::registry_case(input);
                let dependency = dependencies
                    .iter()
                    .find(|dependency| dependency.name == dependency_name)
                    .unwrap_or_else(|| panic!("missing dependency {dependency_name}"));
                assert_eq!(
                    session.registry_urls_with_context(dependency, &context),
                    expected
                        .iter()
                        .map(|url| (*url).to_owned())
                        .collect::<Vec<_>>()
                );
            }
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/fixed", name)
        }

        async fn assert_ruby_path_dependency_fixture(
            fixture: &str,
            package: &str,
            root_name: &str,
        ) {
            let root = local_test_root(root_name);
            let app = root.join("app");
            create_dir_all(app.join("vendor/local")).unwrap();
            let output = standard_session()
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&app.join("Gemfile")),
                        "ruby".to_owned(),
                        package_file_fixture(fixture),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        package.to_owned(),
                        Ruby,
                        r#"[{"number":"9.9.9"}]"#.to_owned(),
                    )],
                )
                .await;
            assert_eq!(output.suggestions[0].status, "directory");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("vendor/local")
            );
            assert_eq!(output.edits.len(), 0);
            remove_dir_all(root).unwrap();
        }

        macro_rules! resolve_fixture {
            ($uri:expr, $language:expr, $fixture:expr, $responses:expr $(,)?) => {{
                standard_session()
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            $uri.to_owned(),
                            $language.to_owned(),
                            package_file_fixture($fixture),
                            None,
                        ),
                        $responses,
                    )
                    .await
            }};
        }

        #[tokio::test]
        async fn cargo_path_dependencies_resolve_existing_relative_directories() {
            let session = standard_session();
            let root = local_test_root("cargo-path-directory");
            let cache = root.join("crates/versionlens-cache");
            create_dir_all(&cache).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&root.join("Cargo.toml")),
                        "toml".to_owned(),
                        r#"
[dependencies]
versionlens-cache = { path = "crates/versionlens-cache" }
versionlens-engine = { version = "0.1.0", path = "crates/versionlens-cache" }
"#
                        .to_owned(),
                        Some(root.to_string_lossy().into_owned()),
                    ),
                    &[RegistryResponseInput::new(
                        "versionlens-cache".to_owned(),
                        Cargo,
                        r#"{"versions":[{"num":"9.9.9","yanked":false}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 2);
            assert_eq!(output.suggestions[0].status, "directory");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("crates/versionlens-cache")
            );
            assert_eq!(output.suggestions[1].status, "directory");
            assert_eq!(
                output.suggestions[1].latest.as_deref(),
                Some("crates/versionlens-cache")
            );
            assert_eq!(output.edits.len(), 0);
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn missing_local_dependencies_return_directory_not_found_without_registry_lookup() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/project/package.json".to_owned(), "json".to_owned(), package_file_fixture("missing-local-dependencies-return-directory-not-found-without-registry-lookup.json"), None),
        &[RegistryResponseInput::new("local".to_owned(), Npm, r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned())],
    ).await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "directoryNotFound",
                Some("../local"),
            );
        }

        #[tokio::test]
        async fn ruby_path_dependencies_resolve_as_directories() {
            assert_ruby_path_dependency_fixture(
                "ruby-path-dependencies-resolve-as-directories.txt",
                "local",
                "ruby-directory",
            )
            .await;
        }

        #[tokio::test]
        async fn stack_custom_resolver_resolves_as_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///work/stack.yaml".to_owned(),
                        "yaml".to_owned(),
                        package_file_fixture(
                            "stack-custom-resolver-resolves-as-fixed-without-registry-updates.yaml",
                        ),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(
                output.suggestions[0].latest,
                Some("stack resolver".to_owned())
            );
        }

        #[tokio::test]
        async fn terraform_builtin_provider_resolves_as_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///main.tf".to_owned(), "terraform".to_owned(), package_file_fixture(
                "terraform-builtin-provider-resolves-as-fixed-without-registry-updates.tf",
            ), None),
        &[RegistryResponseInput::new("terraform.io/builtin/terraform".to_owned(), Terraform, r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned())],
    ).await;

            crate::test_support::assert_fixed_suggestion(&output, "built-in provider");
        }

        #[tokio::test]
        async fn helm_local_and_repository_alias_dependencies_resolve_as_fixed_without_registry_updates()
         {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///Chart.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("helm-local-and-repository-alias-dependencies-resolve-as-fixed-without-registry-updates.yaml"), None),
        &[RegistryResponseInput::new("local".to_owned(), Helm, "apiVersion: v1\nentries:\n  local:\n    - version: 9.9.9\n".to_owned())],
    ).await;

            assert_eq!(output.suggestions.len(), 2);
            assert_suggestion(&output, 0, "fixed", Some("local chart"));
            assert_eq!(output.suggestions[1].status, "fixed");
            assert_eq!(
                output.suggestions[1].latest.as_deref(),
                Some("repository alias")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn ansible_git_role_dependencies_resolve_as_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/requirements.yml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "ansible-git-role-dependencies-resolve-as-fixed-without-registry-updates.yml",
            ), None),
        &[],
    ).await;

            assert_eq!(output.suggestions.len(), 1);
            crate::test_support::assert_fixed_git_repository(&output);
        }

        #[tokio::test]
        async fn bazel_non_registry_overrides_resolve_as_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/MODULE.bazel".to_owned(), "starlark".to_owned(), package_file_fixture("bazel-non-registry-overrides-resolve-as-fixed-without-registry-updatesMODULE.bazel"), None),
        &[],
    ).await;

            assert_eq!(output.suggestions.len(), 2);
            crate::session::resolution::tests::assert_fixed_git_repository_suggestion(&output);
            assert_eq!(output.suggestions[1].status, "fixed");
            assert_eq!(
                output.suggestions[1].latest.as_deref(),
                Some("local module")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn nix_local_inputs_resolve_as_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///work/flake.nix".to_owned(),
                        "nix".to_owned(),
                        package_file_fixture(
                            "nix-local-inputs-resolve-as-fixed-without-registry-updates.nix",
                        ),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "fixed",
                Some("local flake"),
            );
        }

        #[tokio::test]
        async fn renv_non_repository_packages_resolve_as_fixed_sources() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/app/renv.lock".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "renv-non-repository-packages-resolve-as-fixed-sources.lock",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "localpkg".to_owned(),
                        Cran,
                        "Package: localpkg\nVersion: 9.9.9\n".to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("local package")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn cran_fixed_requirements_ignore_versions_from_other_packages() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/DESCRIPTION".to_owned(),
                        "r".to_owned(),
                        "Package: example\nVersion: 0.1.0\nImports: dplyr (1.1.3)\n".to_owned(),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "dplyr".to_owned(),
                        Cran,
                        "Package: dplyr\nVersion: 1.1.4\n\nPackage: unrelated\nVersion: 1.1.3\n"
                            .to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 2);
            assert_eq!(output.suggestions[1].dependency.name, "dplyr");
            assert_eq!(output.suggestions[1].status, "noMatch");
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn go_replace_local_dependencies_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("go-directory");
            let app = root.join("app");
            let local = root.join("local");
            create_dir_all(&app).unwrap();
            create_dir_all(&local).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&app.join("go.mod")),
                        "go.mod".to_owned(),
                        package_file_fixture(
                            "go-replace-local-dependencies-resolve-as-directories.txt",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "example.test/local".to_owned(),
                        Go,
                        "v9.9.9\n".to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "directory",
                Some("../local"),
            );
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn go_work_use_directories_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("go-work-use-directory");
            let app = root.join("app");
            let lib = root.join("lib");
            create_dir_all(&app).unwrap();
            create_dir_all(&lib).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&root.join("go.work")),
                        "go.mod".to_owned(),
                        package_file_fixture("go-work-use-directories-resolve-as-directories.txt"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "./app".to_owned(),
                        Go,
                        "v9.9.9\n".to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 2);
            assert!(
                output
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.status == "directory")
            );
            assert_eq!(output.suggestions[0].latest.as_deref(), Some("./app"));
            assert_eq!(output.suggestions[1].latest.as_deref(), Some("./lib"));
            assert_eq!(output.edits.len(), 0);
            remove_dir_all(root).unwrap();
        }
        #[tokio::test]
        async fn gradle_version_catalog_version_resolves_through_referencing_library() {
            let output = resolve_fixture!(
        "file:///repo/gradle/libs.versions.toml",
        "toml",
        "version-catalog-references-are-fixed-without-registry-updates.versions.toml",
        &[RegistryResponseInput::new("org.codehaus.groovy:groovy".to_owned(), Maven, r"<metadata><versioning><versions><version>3.0.5</version><version>4.0.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 2);
            assert_suggestion(&output, 0, "updateAvailable", Some("4.0.0"));
            assert_suggestion(&output, 1, "fixed", Some("version catalog reference"));
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "4.0.0");
            assert_eq!(output.edits[0].range.start.line, 1);
        }

        #[tokio::test]
        async fn gradle_version_catalog_direct_library_versions_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/gradle/libs.versions.toml",
        "toml",
        "version-catalog-direct-library-versions-use-maven-lookup.versions.toml",
        &[RegistryResponseInput::new("org.apache.commons:commons-lang3".to_owned(), Maven, r"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "updateAvailable", Some("3.18.0"));
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "3.18.0");
        }

        #[tokio::test]
        async fn gradle_version_catalog_unreferenced_versions_are_fixed_without_registry_lookup() {
            let output = resolve_fixture!(
                "file:///repo/gradle/libs.versions.toml",
                "toml",
                "version-catalog-unreferenced-versions-are-fixed-without-registry-lookup.versions.toml",
                &[],
            );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("version catalog alias"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn sbt_scala_cross_dependencies_without_scala_version_are_fixed() {
            let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-scala-cross-dependencies-without-scala-version-are-fixed.sbt",
        &[RegistryResponseInput::new("org.typelevel:cats-core".to_owned(), Maven, r"<metadata><versioning><versions><version>2.13.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("Scala binary version"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn sbt_maven_dependencies_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-maven-dependencies-use-maven-lookup.sbt",
        &[RegistryResponseInput::new("org.scala-stm:scala-stm_2.13".to_owned(), Maven, r"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("0.9.1"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn sbt_url_artifact_dependencies_are_fixed_without_registry_updates() {
            let output = resolve_fixture!(
        "file:///repo/build.sbt",
        "scala",
        "sbt-url-artifact-dependencies-are-fixed-without-registry-updates.sbt",
        &[RegistryResponseInput::new("jquery:jquery".to_owned(), Maven, r"<metadata><versioning><versions><version>3.2.2</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("package URL"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn gradle_build_dependencies_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/build.gradle.kts",
        "kotlin",
        "build-dependencies-use-maven-lookup.gradle.kts",
        &[RegistryResponseInput::new("org.springframework:spring-core".to_owned(), Maven, r"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("6.2.8"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn gradle_plugin_markers_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/settings.gradle",
        "groovy",
        "plugin-markers-use-maven-lookup.gradle",
        &[RegistryResponseInput::new("com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(), Maven, r"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("0.51.0"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn gradle_kotlin_shorthand_plugin_routes_to_maven_marker_and_updates() {
            let session = standard_session();
            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/build.gradle.kts".to_owned(),
                        "kotlin".to_owned(),
                        "plugins {\n    kotlin(\"jvm\") version \"2.0.0\"\n}".to_owned(),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
                            .to_owned(),
                        Maven,
                        r#"{"versions":["2.0.0","2.1.0"]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "updateAvailable", Some("2.1.0"));
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "2.1.0");
        }

        #[tokio::test]
        async fn gradle_project_and_file_dependencies_are_fixed() {
            let output = resolve_fixture!(
        "file:///repo/build.gradle",
        "groovy",
        "project-and-file-dependencies-are-fixed.gradle",
        &[RegistryResponseInput::new(":shared".to_owned(), Maven, r"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 2);
            assert_suggestion(&output, 0, "fixed", Some("local package"));
            assert_suggestion(&output, 1, "fixed", Some("local package"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn gradle_kotlin_named_argument_dependencies_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/build.gradle.kts",
        "kotlin",
        "kotlin-named-argument-dependencies-use-maven-lookup.gradle.kts",
        &[RegistryResponseInput::new("org.slf4j:slf4j-api".to_owned(), Maven, r"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("2.0.17"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn clojure_deps_edn_git_and_local_dependencies_are_fixed() {
            let output = resolve_fixture!(
        "file:///repo/deps.edn",
        "clojure",
        "clojure-deps-edn-git-and-local-dependencies-are-fixed.edn",
        &[
            RegistryResponseInput::new("io.github.sally:awesome".to_owned(), Maven, r"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>".to_owned()),
            RegistryResponseInput::new("my.dev:project".to_owned(), Maven, r"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>".to_owned()),
        ],
    );

            assert_eq!(output.suggestions.len(), 2);
            assert_suggestion(&output, 0, "fixed", Some("git repository"));
            assert_suggestion(&output, 1, "fixed", Some("local package"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn clojure_deps_edn_maven_dependencies_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/deps.edn",
        "clojure",
        "clojure-deps-edn-maven-dependencies-use-maven-lookup.edn",
        &[RegistryResponseInput::new("org.clojure:tools.reader".to_owned(), Maven, r"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "fixed", Some("1.1.1"));
            assert_no_edits(&output);
        }

        #[tokio::test]
        async fn leiningen_project_version_is_local_and_dependencies_use_maven_lookup() {
            let output = resolve_fixture!(
        "file:///repo/project.clj",
        "clojure",
        "leiningen-project-clj-dependencies-use-maven-lookup.clj",
        &[RegistryResponseInput::new("org.clojure:clojure".to_owned(), Maven, r"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>".to_owned())],
    );

            assert_eq!(output.suggestions.len(), 2);
            assert_suggestion(&output, 0, "updateAvailable", Some("0.1.0"));
            assert_suggestion(&output, 1, "fixed", Some("1.11.3"));
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "0.1.0");
        }
        #[tokio::test]
        async fn pub_path_dependencies_resolve_as_directories() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/app/pubspec.yaml".to_owned(),
                        "yaml".to_owned(),
                        package_file_fixture("pub-path-dependencies-resolve-as-directories.yaml"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "http_parser".to_owned(),
                        Pub,
                        r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "directory",
                Some("../../"),
            );
        }

        #[tokio::test]
        async fn pub_sdk_dependencies_resolve_as_fixed_without_registry_lookup() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/app/pubspec.yaml".to_owned(),
                        "yaml".to_owned(),
                        package_file_fixture(
                            "pub-sdk-dependencies-resolve-as-fixed-without-registry-lookup.yaml",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "flutter".to_owned(),
                        Pub,
                        r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "fixed",
                Some("sdk:flutter"),
            );
        }

        #[tokio::test]
        async fn pub_workspace_paths_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("pub-workspace-directory");
            let packages = root.join("packages");
            let shared = packages.join("shared");
            create_dir_all(&shared).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&root.join("pubspec.yaml")),
                        "yaml".to_owned(),
                        package_file_fixture("pub-workspace-paths-resolve-as-directories.txt"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "packages/shared".to_owned(),
                        Pub,
                        r#"{"latest":{"version":"9.9.9"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "directory");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("packages/shared")
            );
            assert_eq!(output.edits.len(), 0);
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn dub_sdl_path_dependencies_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("dub-directory");
            create_two_local_dependencies(&root);
            let output = resolve_local_fixture(LocalFixtureCase {
                session: &session,
                root: &root,
                manifest: "dub.sdl",
                language_id: "plaintext",
                fixture_name: "dub-sdl-path-dependencies-resolve-as-directories.txt",
                package: "localdep",
                ecosystem: Dub,
                response: r#"{"versions":[{"version":"9.9.9"}]}"#,
            })
            .await;

            assert_two_directory_suggestions(&output, "./localdep", "vendor/localdep");
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn dub_json_path_dependencies_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("dub-json-directory");
            let local = root.join("localdep");
            create_dir_all(&local).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&root.join("dub.json")),
                        "json".to_owned(),
                        package_file_fixture(
                            "dub-json-path-dependencies-resolve-as-directories.txt",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "localdep".to_owned(),
                        Dub,
                        r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "directory",
                Some("./localdep"),
            );
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn gleam_path_dependencies_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("gleam-directory");
            create_two_local_dependencies(&root);
            let output = resolve_local_fixture(LocalFixtureCase {
                session: &session,
                root: &root,
                manifest: "gleam.toml",
                language_id: "toml",
                fixture_name: "gleam-path-dependencies-resolve-as-directories.txt",
                package: "localdep",
                ecosystem: Hex,
                response: r#"{"releases":[{"version":"9.9.9"}]}"#,
            })
            .await;

            assert_two_directory_suggestions(&output, "./localdep", "vendor/localdep");
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn gleam_git_dependencies_are_fixed_without_registry_updates() {
            let session = standard_session();
            let output = crate::test_support::resolve_fixture_with_response(
                crate::test_support::FixtureResolutionCase {
                    session: &session,
                    uri: "file:///repo/gleam.toml",
                    language_id: "toml",
                    fixture_name: "gleam-git-dependencies-are-fixed-without-registry-updates.toml",
                    package: "my_library",
                    ecosystem: Hex,
                    response: r#"{"releases":[{"version":"9.9.9"}]}"#,
                },
            )
            .await;

            crate::test_support::assert_fixed_git_repository(&output);
        }

        #[tokio::test]
        async fn mix_umbrella_dependencies_are_fixed_without_registry_updates() {
            let session = standard_session();
            let output = crate::test_support::resolve_fixture_with_response(crate::test_support::FixtureResolutionCase { session: &session, uri: "file:///repo/mix.exs", language_id: "elixir", fixture_name: "mix-umbrella-dependencies-are-fixed-without-registry-updates.exs", package: "shared_app", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# }).await;

            assert_fixed_without_edits(&output, "umbrella dependency");
        }

        #[tokio::test]
        async fn rebar_git_dependencies_are_fixed_without_registry_updates() {
            let session = standard_session();
            let output = crate::test_support::resolve_fixture_with_response(crate::test_support::FixtureResolutionCase { session: &session, uri: "file:///repo/rebar.config", language_id: "erlang", fixture_name: "rebar-git-dependencies-are-fixed-without-registry-updates.config", package: "gettext", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# }).await;

            crate::test_support::assert_fixed_git_repository(&output);
        }

        #[tokio::test]
        async fn rebar_mercurial_dependencies_are_fixed_without_registry_updates() {
            let session = standard_session();
            let output = crate::test_support::resolve_fixture_with_response(crate::test_support::FixtureResolutionCase { session: &session, uri: "file:///repo/rebar.config", language_id: "erlang", fixture_name: "rebar-mercurial-dependencies-are-fixed-without-registry-updates.config", package: "legacy", ecosystem: Hex, response: r#"{"releases":[{"version":"9.9.9"}]}"# }).await;

            assert_fixed_without_edits(&output, "hg repository");
        }

        fn assert_fixed_without_edits(
            output: &crate::contract::ResolveDocumentOutput,
            latest: &str,
        ) {
            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(output.suggestions[0].latest.as_deref(), Some(latest));
            assert_eq!(output.edits.len(), 0);
        }

        fn assert_two_directory_suggestions(
            output: &crate::contract::ResolveDocumentOutput,
            first: &str,
            second: &str,
        ) {
            assert_eq!(output.suggestions.len(), 2);
            for (suggestion, expected) in output.suggestions.iter().zip([first, second]) {
                assert_eq!(suggestion.status, "directory");
                assert_eq!(suggestion.latest.as_deref(), Some(expected));
            }
            assert_eq!(output.edits.len(), 0);
        }

        fn create_two_local_dependencies(root: &std::path::Path) {
            create_dir_all(root.join("localdep")).unwrap();
            create_dir_all(root.join("vendor/localdep")).unwrap();
        }

        struct LocalFixtureCase<'a> {
            session: &'a crate::VersionLensSession,
            root: &'a std::path::Path,
            manifest: &'a str,
            language_id: &'a str,
            fixture_name: &'a str,
            package: &'a str,
            ecosystem: versionlens_model::Ecosystem,
            response: &'a str,
        }

        async fn resolve_local_fixture(
            case: LocalFixtureCase<'_>,
        ) -> crate::contract::ResolveDocumentOutput {
            case.session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&case.root.join(case.manifest)),
                        case.language_id.to_owned(),
                        package_file_fixture(case.fixture_name),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        case.package.to_owned(),
                        case.ecosystem,
                        case.response.to_owned(),
                    )],
                )
                .await
        }

        #[tokio::test]
        async fn python_direct_url_requirements_remain_fixed_without_registry_lookup() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/requirements.txt".to_owned(), "pip-requirements".to_owned(), package_file_fixture(
                "python-direct-url-requirements-resolve-as-blank-versions-like-upstream.txt",
            ), None),
        &[RegistryResponseInput::new("local".to_owned(), Python, r#"{"info":{"version":"9.9.9"}}"#.to_owned())],
    ).await;

            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("https://example.test/local.whl#sha256=abc")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn npm_workspace_and_catalog_dependencies_are_surfaced_without_registry_lookup() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("workspace-and-catalog-dependencies-are-skipped.json"),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 2);
            crate::test_support::assert_all_fixed_without_edits(&output);
        }

        #[tokio::test]
        async fn bun_trusted_dependency_name_arrays_are_fixed_by_default() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "bun-trusted-dependency-name-arrays-are-fixed-by-default.json",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "my-trusted-package".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("trusted dependency")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn npm_bundle_name_arrays_are_fixed_by_default() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("bundle-name-arrays-are-fixed-by-default.json"),
                        None,
                    ),
                    &[
                        RegistryResponseInput::new(
                            "left-pad".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                        ),
                        RegistryResponseInput::new(
                            "right-pad".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
                        ),
                    ],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "fixed");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("bundled dependency")
            );
            assert_eq!(output.suggestions[1].status, "fixed");
            assert_eq!(
                output.suggestions[1].latest.as_deref(),
                Some("bundled dependency")
            );
            assert_eq!(output.edits.len(), 0);
        }
        #[tokio::test]
        async fn composer_platform_dependencies_are_fixed() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/composer.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("platform-dependencies-are-fixed.json"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "phpunit/phpunit".to_owned(),
                        Composer,
                        r#"{"packages":{"phpunit/phpunit":[{"version":"10.5.0"}]}}"#.to_owned(),
                    )],
                )
                .await;

            assert_suggestion(&output, 0, "fixed", Some("^8.3"));
            assert_eq!(output.suggestions[1].status, "fixed");
            assert_eq!(output.suggestions[1].latest.as_deref(), Some("*"));
            assert_eq!(output.suggestions[2].latest.as_deref(), Some("10.5.0"));
        }

        #[tokio::test]
        async fn composer_stability_flags_allow_prerelease_updates() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture("stability-flags-allow-prerelease-updates.json"), None),
        &[RegistryResponseInput::new("acme/pkg".to_owned(), Composer, r#"{"packages":{"acme/pkg":[{"version":"1.0.0"},{"version":"1.1.0-beta.1"}]}}"#
                .to_owned())],
    ).await;

            assert_eq!(output.suggestions.len(), 1);
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("1.1.0-beta.1")
            );
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "^1.1.0-beta.1@beta");
        }

        #[tokio::test]
        async fn fixed_composer_release_resolves_fixed_with_release_update_choices() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///repo/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("release-resolves-fixed-with-release-update-choices.json"),
                None,
            );

            let output = resolve_composer(
                &session,
                &input,
                r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v1.1.0" },
                  { "version": "v1.1.1" },
                  { "version": "v1.1.2" },
                  { "version": "v1.2.0" },
                  { "version": "v1.2.2" },
                  { "version": "v2.0.0" },
                  { "version": "v2.2.2" }
                ]
              }
            }"#,
            )
            .await;

            let (titles, arguments) = analyze_composer(&session, input);

            crate::test_support::assert_suggestion_without_edits(
                &output,
                0,
                "fixed",
                Some("1.1.1"),
            );
            assert_eq!(
                titles,
                [
                    "🟡 fixed 1.1.1",
                    "↑  patch 1.1.2",
                    "↑  minor 1.2.2",
                    "↑  latest 2.2.2"
                ]
            );
            assert_eq!(
                arguments,
                [
                    vec!["updatePatch", "1.1.2"],
                    vec!["updateMinor", "1.2.2"],
                    vec!["update", "2.2.2"]
                ]
            );
        }

        #[tokio::test]
        async fn missing_fixed_composer_registry_version_resolves_no_match_with_update_choices() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///repo/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(
                    "missing-fixed-composer-registry-version-resolves-no-match-with-update-choices.json",
                ),
                None,
            );

            let output = resolve_missing_composer(&session, &input).await;

            let (titles, arguments) = analyze_composer(&session, input);

            crate::test_support::assert_suggestion_without_edits(&output, 0, "noMatch", None);
            assert_eq!(
                titles,
                [
                    "⚪ no match",
                    "↑  patch 0.5.1",
                    "↑  minor 0.6.0",
                    "↑  latest 1.0.0"
                ]
            );
            assert_eq!(
                arguments,
                [
                    vec!["updatePatch", "0.5.1"],
                    vec!["updateMinor", "0.6.0"],
                    vec!["update", "1.0.0"]
                ]
            );
        }

        #[tokio::test]
        async fn invalid_composer_requirement_resolves_invalid_without_registry_lookup() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/composer.json".to_owned(), "json".to_owned(), package_file_fixture(
                "invalid-composer-requirement-resolves-invalid-without-registry-lookup.json",
            ), None),
        &[RegistryResponseInput::new("php-parallel-lint/php-parallel-lint".to_owned(), Composer, r#"{"packages":{"php-parallel-lint/php-parallel-lint":[{"version":"v9.9.9"}]}}"#
                .to_owned())],
    ).await;

            assert_eq!(output.suggestions[0].status, "invalid");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("invalid version")
            );
            assert_eq!(output.edits.len(), 0);
        }

        async fn resolve_composer(
            session: &crate::VersionLensSession,
            input: &DocumentInput,
            body: &str,
        ) -> crate::contract::ResolveDocumentOutput {
            session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "php-parallel-lint/php-parallel-lint".to_owned(),
                        Composer,
                        body.to_owned(),
                    )],
                )
                .await
        }

        async fn resolve_missing_composer(
            session: &crate::VersionLensSession,
            input: &DocumentInput,
        ) -> crate::contract::ResolveDocumentOutput {
            resolve_composer(
                session,
                input,
                r#"{
              "packages": {
                "php-parallel-lint/php-parallel-lint": [
                  { "version": "v0.5.1" },
                  { "version": "v0.6.0" },
                  { "version": "v1.0.0" }
                ]
              }
            }"#,
            )
            .await
        }

        fn analyze_composer(
            session: &crate::VersionLensSession,
            input: DocumentInput,
        ) -> (Vec<String>, Vec<Vec<String>>) {
            let analysis = session.analyze_document(input);
            let (titles, arguments) =
                crate::session::resolution::tests::lens_titles_and_arguments(&analysis);
            (titles.into_iter().map(str::to_owned).collect(), arguments)
        }
        #[tokio::test]
        async fn ruby_path_block_dependencies_resolve_as_directories() {
            assert_ruby_path_dependency_fixture(
                "ruby-path-block-dependencies-resolve-as-directories.txt",
                "local_one",
                "ruby-path-block-directory",
            )
            .await;
        }

        #[tokio::test]
        async fn git_dependencies_are_fixed() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///Cargo.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture("git-dependencies-are-fixed.toml"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "remote".to_owned(),
                        Cargo,
                        r#"{"crate":{"max_version":"9.9.9"}}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_fixed_git_repository(&output);
        }

        #[test]
        fn cargo_registry_dependencies_use_workspace_cargo_config_urls() {
            let (root, input) = cargo_registry_input(
                "versionlens-cargo-registry",
                "cargo-registry-dependencies-use-workspace-cargo-config-urls.txt",
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(dependencies[0].name, "private");
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://cargo.example.test/api/private/versions"]
            );

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn cargo_renamed_registry_dependencies_use_package_identity_for_lookup() {
            let (root, input) = cargo_registry_input(
                "versionlens-cargo-renamed-registry",
                "cargo-renamed-registry-dependencies-use-package-identity-for-lookup.txt",
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(dependencies[0].name, "local_name");
            assert_eq!(
                dependencies[0].hosted_name.as_deref(),
                Some("registry-name")
            );
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://cargo.example.test/api/registry-name/versions"]
            );

            remove_dir_all(root).unwrap();
        }

        fn cargo_registry_input(
            root_name: &str,
            fixture: &str,
        ) -> (std::path::PathBuf, DocumentInput) {
            let root = temp_dir().join(format!("{root_name}-{}", id()));
            create_dir_all(root.join(".cargo")).unwrap();
            write(
                root.join(".cargo/config.toml"),
                "[registries.private]\nindex = 'https://cargo.example.test/api/'\n",
            )
            .unwrap();
            let input = DocumentInput::new(
                format!("file://{}", root.join("Cargo.toml").display()),
                "toml".to_owned(),
                package_file_fixture(fixture),
                Some(root.to_string_lossy().into_owned()),
            );
            (root, input)
        }

        #[test]
        fn cargo_crates_io_source_replacement_uses_workspace_cargo_config_url() {
            let root = temp_dir().join(format!("versionlens-cargo-replace-{}", id()));
            create_dir_all(root.join(".cargo")).unwrap();
            write(
        root.join(".cargo/config.toml"),
        "[source.crates-io]\nreplace-with = 'mirror'\n[source.mirror]\nregistry = 'sparse+https://mirror.example.test/api/'\n",
    )
    .unwrap();

            let input = DocumentInput::new(
                format!("file://{}", root.join("Cargo.toml").display()),
                "toml".to_owned(),
                package_file_fixture(
                    "cargo-crates-io-source-replacement-uses-workspace-cargo-config-url.txt",
                ),
                Some(root.to_string_lossy().into_owned()),
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(dependencies[0].name, "serde");
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://mirror.example.test/api/serde/versions"]
            );

            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn cargo_workspace_inherited_dependencies_do_not_create_registry_updates() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///repo/member/Cargo.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "cargo-workspace-inherited-dependencies-do-not-create-registry-updates.toml",
            ), Some("/repo".to_owned())),
        &[
            RegistryResponseInput::new("regex".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
            RegistryResponseInput::new("cc".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
            RegistryResponseInput::new("rand".to_owned(), Cargo, r#"{"versions":[{"num":"9.9.9"}]}"#.to_owned()),
        ],
    ).await;

            assert_eq!(output.suggestions.len(), 3);
            assert!(
                output
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.status == "fixed")
            );
            assert!(
                output
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.latest.as_deref() == Some("workspace:true"))
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[test]
        fn npm_dependencies_use_workspace_bunfig_registry_urls() {
            let root = temp_dir().join(format!("versionlens-bunfig-registry-{}", id()));
            create_dir_all(&root).unwrap();
            write(
        root.join("bunfig.toml"),
        "[install]\nregistry = 'https://${REGISTRY_HOST}/npm'\n[install.scopes]\n'@scope' = { url = 'https://${SCOPE_HOST}/npm', token = '${BUN_SCOPE_TOKEN}' }\n",
    )
    .unwrap();
            write(
        root.join(".env"),
        "REGISTRY_HOST=registry.example.test\nSCOPE_HOST=scope.example.test\nBUN_SCOPE_TOKEN=secret\n",
    )
    .unwrap();

            let input = DocumentInput::new(
                format!("file://{}", root.join("package.json").display()),
                "json".to_owned(),
                package_file_fixture("dependencies-use-workspace-bunfig-registry-urls.txt"),
                Some(root.to_string_lossy().into_owned()),
            );
            let (_session, context, _dependencies) =
                crate::session::resolution::tests::registry_case_with_expected_urls(
                    &input,
                    &[
                        &["https://registry.example.test/npm/left-pad"],
                        &["https://scope.example.test/npm/@scope%2fpkg"],
                    ],
                );
            assert_eq!(
                context.auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg")[0]
                    .value,
                "Bearer secret"
            );

            remove_dir_all(root).unwrap();
        }
        #[tokio::test]
        async fn composer_inline_alias_dependencies_are_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/composer.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "inline-alias-dependencies-are-fixed-without-registry-updates.json",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "acme/pkg".to_owned(),
                        Composer,
                        r#"{"packages":{"acme/pkg":[{"version":"1.1.0"}]}}"#.to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_fixed_suggestion(&output, "dev-bugfix as 1.0.x-dev");
        }

        #[tokio::test]
        async fn composer_inline_package_repository_resolves_without_registry_response() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///repo/composer.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "inline-package-repository-resolves-without-registry-response.json",
                        ),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 1);
            assert_suggestion(&output, 0, "updateAvailable", Some("3.1.7"));
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, "3.1.7");
        }

        #[test]
        fn composer_repository_filters_route_matching_packages_only() {
            let input = DocumentInput::new(
                "file:///repo/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("repository-filters-route-matching-packages-only.json"),
                None,
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(dependencies[0].name, "acme/private");
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://private.packages.example.test/acme/private.json"]
            );
            assert_eq!(
                session.registry_urls_with_context(&dependencies[1], &context),
                vec!["https://repo.packagist.org/p2/acme/blocked.json"]
            );
            assert_eq!(
                session.registry_urls_with_context(&dependencies[2], &context),
                vec!["https://repo.packagist.org/p2/vendor/public.json"]
            );
        }

        #[test]
        fn composer_can_disable_default_packagist_registry() {
            let input = DocumentInput::new(
                "file:///repo/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("can-disable-default-packagist-registry.json"),
                None,
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://private.packages.example.test/acme/private.json"]
            );
            assert_eq!(
                session
                    .registry_urls_with_context(&dependencies[1], &context)
                    .len(),
                0
            );
        }

        #[tokio::test]
        async fn explicit_docker_registries_return_no_match_from_mcr_shaped_responses() {
            let session = standard_session();

            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///compose.yaml".to_owned(), "yaml".to_owned(), package_file_fixture(
                "explicit-docker-registries-return-no-match-from-mcr-shaped-responses.yaml",
            ), None),
        &[RegistryResponseInput::new("team/app".to_owned(), Docker, r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#.to_owned())],
    ).await;

            crate::test_support::assert_suggestion_without_edits(&output, 0, "noMatch", None);
        }

        #[tokio::test]
        async fn docker_compose_bare_build_contexts_resolve_as_directories() {
            let session = standard_session();
            let root = local_test_root("docker-directory");
            let local = root.join("backend/dockerfile");
            create_dir_all(&local).unwrap();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        file_uri(&root.join("compose.yaml")),
                        "yaml".to_owned(),
                        package_file_fixture(
                            "docker-compose-bare-build-contexts-resolve-as-directories.txt",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "backend/dockerfile".to_owned(),
                        Docker,
                        r#"{"results":[{"name":"2.0.0","images":[{"digest":"sha256:abc"}]}]}"#
                            .to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "directory");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("backend/dockerfile")
            );
            assert_eq!(output.edits.len(), 0);
            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn npm_git_dependencies_distinguish_hosted_and_unsupported_git() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "git-dependencies-distinguish-hosted-and-unsupported-git.json",
                        ),
                        None,
                    ),
                    &[],
                )
                .await;

            crate::session::resolution::tests::assert_fixed_git_repository_suggestion(&output);
            assert_eq!(output.suggestions[1].status, "notSupported");
            assert_eq!(output.suggestions[1].latest, None);
            assert_eq!(output.suggestions[2].status, "notSupported");
            assert_eq!(output.suggestions[2].latest, None);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn unity_local_and_git_dependencies_resolve_as_fixed_without_registry_updates() {
            let session = standard_session();
            let output = session.resolve_document(DocumentInput::new("file:///work/Packages/manifest.json".to_owned(), "json".to_owned(), package_file_fixture(
            "unity-local-and-git-dependencies-resolve-as-fixed-without-registry-updates.json",
        ), None)).await;

            assert_eq!(output.suggestions.len(), 2);
            crate::test_support::assert_all_fixed_without_edits(&output);
        }
    }
    mod github {
        use super::*;
        use std::io::{Read, Write};
        use std::thread::{JoinHandle, sleep, spawn};
        use std::time::{Duration, Instant};

        mod support {
            use super::*;

            pub(super) async fn resolve_github_fixture(
                fixture: &str,
                package: &str,
                ecosystem: Ecosystem,
                body: &str,
            ) -> ResolveDocumentOutput {
                let session = standard_session();
                let (uri, language) = if fixture.ends_with("Gemfile") {
                    ("file:///Gemfile", "ruby")
                } else {
                    ("file:///package.json", "json")
                };
                session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            uri.to_owned(),
                            language.to_owned(),
                            package_file_fixture(fixture),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            package.to_owned(),
                            ecosystem,
                            body.to_owned(),
                        )],
                    )
                    .await
            }

            pub(super) fn assert_no_action_update(output: &ResolveDocumentOutput) {
                assert_eq!(output.edits.len(), 0);
                assert!(
                    output
                        .suggestions
                        .iter()
                        .all(|suggestion| suggestion.status != "updateAvailable")
                );
            }
        }
        use support::{assert_no_action_update, resolve_github_fixture};

        pub(super) fn github_api_server<T: AsRef<str> + Send + 'static>(
            responses: Vec<(u16, T)>,
        ) -> (String, JoinHandle<Vec<String>>) {
            let listener = crate::test_support::tcp_listener_bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let base_url = format!("http://{}/repos/", listener.local_addr().unwrap());
            let server = spawn(move || {
                let deadline = Instant::now() + Duration::from_secs(3);
                let mut paths = vec![];
                for (status, body) in responses {
                    let body = body.as_ref();
                    let mut accepted = None;
                    while Instant::now() < deadline {
                        if let Ok(connection) = listener.accept() {
                            accepted = Some(connection);
                            break;
                        }
                        sleep(Duration::from_millis(5));
                    }
                    let Some((mut stream, _)) = accepted else {
                        break;
                    };
                    stream.set_nonblocking(false).unwrap();
                    stream
                        .set_read_timeout(Some(Duration::from_secs(3)))
                        .unwrap();
                    let mut request = [0_u8; 2048];
                    let length = stream.read(&mut request).unwrap();
                    let request = String::from_utf8_lossy(&request[..length]);
                    let path = request
                        .lines()
                        .next()
                        .and_then(|line| line.split_whitespace().nth(1))
                        .unwrap()
                        .to_owned();
                    let reason = if status == 200 { "OK" } else { "Not Found" };
                    let response = format!(
                        "HTTP/1.1 {status} {reason}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).unwrap();
                    paths.push(path);
                }
                paths
            });
            (base_url, server)
        }

        async fn resolve_checkout_tag(body: &str) -> ResolveDocumentOutput {
            standard_session()
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml".to_owned(),
                        "yaml".to_owned(),
                        "steps:\n  - uses: actions/checkout@v4\n".to_owned(),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "actions/checkout".to_owned(),
                        GitHub,
                        body.to_owned(),
                    )],
                )
                .await
        }

        async fn resolve_action_source(source: &str, body: &str) -> ResolveDocumentOutput {
            standard_session()
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml".to_owned(),
                        "yaml".to_owned(),
                        source.to_owned(),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "actions/checkout".to_owned(),
                        GitHub,
                        body.to_owned(),
                    )],
                )
                .await
        }

        fn action_choices(
            source: &str,
            latest: &str,
            body: &str,
        ) -> Vec<versionlens_update::UpdateChoice> {
            let input = DocumentInput::new(
                "file:///work/.github/workflows/ci.yml".to_owned(),
                "yaml".to_owned(),
                source.to_owned(),
                None,
            );
            let dependency = parse_document(&input).into_iter().next().unwrap();
            crate::fetch::response_update_choices(&dependency, latest, body, true, &[])
        }

        #[tokio::test]
        async fn resolves_github_action_tag_references_with_incremental_choices() {
            let output = resolve_checkout_tag(&github_action_tags()).await;

            assert_update(&output, "v4.2.0");
        }

        #[tokio::test]
        async fn resolves_major_action_refs_when_registry_has_only_concrete_release_tags() {
            let output =
                resolve_checkout_tag(r#"[{"name":"v4.2.0"},{"name":"v4.1.0"},{"name":"v4.0.0"}]"#)
                    .await;

            assert_update(&output, "v4.2.0");
        }

        #[tokio::test]
        async fn sha_pinned_action_at_latest_has_no_redundant_latest_choice() {
            let sha = "3d3c42e5aac5ba805825da76410c181273ba90b1";
            let source = format!("steps:\n  - uses: actions/checkout@{sha} # v7.0.1\n");
            let output = resolve_action_source(
        &source,
        &format!(
            r#"[{{"name":"v7.0.1","commit":{{"sha":"{sha}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
        ),
    )
    .await;

            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.suggestions[0].status, "current");
            let choices = action_choices(
                &source,
                "7.0.1",
                &format!(
                    r#"[{{"name":"v7.0.1","commit":{{"sha":"{sha}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
                ),
            );
            assert!(
                choices
                    .iter()
                    .all(|choice| !choice.label.starts_with("latest"))
            );
            assert!(choices.iter().all(|choice| choice.version != "6.0.0"));
        }

        #[tokio::test]
        async fn concrete_azure_login_release_pin_resolves_as_current() {
            let sha = "7ddb5af1ef8758cf1353cf3b42f940aee27ba21c";
            let source = format!("steps:\n  - uses: azure/login@{sha} # v3.0.2\n");
            let output = standard_session().resolve_document_with_responses(
        DocumentInput::new(
            "file:///work/.github/workflows/publish.yml".to_owned(),
            "yaml".to_owned(),
            source,
            None,
        ),
        &[RegistryResponseInput::new(
            "azure/login".to_owned(),
            GitHub,
            format!(
                r#"[{{"name":"v3.0.2","commit":{{"sha":"{sha}"}}}},{{"name":"v3.0.1","commit":{{"sha":"f5d393ae46f8fde4be8b75f32e3fc50e654ad0ca"}}}}]"#
            ),
        )],
    ).await;

            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.suggestions[0].status, "current");
            assert_eq!(output.suggestions[0].latest.as_deref(), Some("3.0.2"));
        }

        #[tokio::test]
        async fn annotated_azure_login_tag_object_is_not_reported_as_a_verified_commit_pin() {
            let (base_url, server) = github_api_server(vec![
                (
                    200,
                    r#"[{"name":"v2","commit":{"sha":"7184910d9eb2b1c5e48f7073824a90609bb9b6d6"}}]"#,
                ),
                (
                    200,
                    r#"{"ref":"refs/tags/v2","object":{"type":"tag","sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac"}}"#,
                ),
                (
                    200,
                    r#"{"sha":"8216e11d8cd9b42fe925c852af8e76311ff067ac","object":{"type":"commit","sha":"7184910d9eb2b1c5e48f7073824a90609bb9b6d6"}}"#,
                ),
            ]);

            let session = github_session(base_url);
            let output = session
                .resolve_document(DocumentInput::new(
                    "file:///work/.github/workflows/publish.yml".to_owned(),
                    "yaml".to_owned(),
                    "steps:\n  - uses: azure/login@8216e11d8cd9b42fe925c852af8e76311ff067ac # v2\n"
                        .to_owned(),
                    None,
                ))
                .await;
            let paths = server.join().unwrap();

            assert_eq!(
                paths,
                [
                    "/repos/azure/login/tags",
                    "/repos/azure/login/git/ref/tags/v2",
                    "/repos/azure/login/git/tags/8216e11d8cd9b42fe925c852af8e76311ff067ac"
                ]
            );
            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.suggestions[0].status, "error");
        }

        #[tokio::test]
        async fn missing_exact_github_ref_reports_an_unverified_action_reference() {
            let (base_url, server) = github_api_server(vec![
                (
                    200,
                    r#"[{"name":"v2","commit":{"sha":"7184910d9eb2b1c5e48f7073824a90609bb9b6d6"}}]"#,
                ),
                (404, r#"{"message":"Not Found"}"#),
            ]);
            let session = github_session(base_url);

            let output = session
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/publish.yml".to_owned(),
            "yaml".to_owned(),
            "steps:\n  - uses: azure/login@8216e11d8cd9b42fe925c852af8e76311ff067ac # missing-v2\n"
                .to_owned(),
            None,
        ))
        .await;
            let paths = server.join().unwrap();

            assert_eq!(
                paths,
                [
                    "/repos/azure/login/tags",
                    "/repos/azure/login/git/ref/tags/missing-v2"
                ]
            );
            assert_eq!(output.edits.len(), 0);
            assert_eq!(output.suggestions[0].status, "error");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("GitHub action reference does not match a verified published release")
            );
        }

        #[tokio::test]
        async fn sha_pinned_action_updates_sha_and_annotation_atomically() {
            let current = "3d3c42e5aac5ba805825da76410c181273ba90b1";
            let target = "7777777777777777777777777777777777777777";
            let source = format!("steps:\n  - uses: actions/checkout@{current} # v7.0.1\n");
            let body = format!(
                r#"[{{"name":"v7.1.0","commit":{{"sha":"{target}"}}}},{{"name":"v7.0.1","commit":{{"sha":"{current}"}}}},{{"name":"v6.0.0","commit":{{"sha":"6666666666666666666666666666666666666666"}}}}]"#
            );
            let output = resolve_action_source(&source, &body).await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(output.edits.len(), 1);
            assert_eq!(output.edits[0].new_text, format!("{target} # v7.1.0"));
            let choices = action_choices(&source, "7.1.0", &body);
            assert!(choices.iter().all(|choice| choice.version != "6.0.0"));
        }

        #[tokio::test]
        async fn abbreviated_sha_pin_is_proven_by_the_full_tag_commit() {
            let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@3d3c42e # v7.0.1\n",
        r#"[{"name":"v7.1.0","commit":{"sha":"7777777777777777777777777777777777777777"}},{"name":"v7.0.1","commit":{"sha":"3d3c42e5aac5ba805825da76410c181273ba90b1"}}]"#,
    ).await;

            assert_eq!(
                output.edits[0].new_text,
                "7777777777777777777777777777777777777777 # v7.1.0"
            );
        }

        #[tokio::test]
        async fn mismatched_sha_and_annotation_are_not_treated_as_a_proven_current_ref() {
            let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@3d3c42e # v7.0.1\n",
        r#"[{"name":"v7.1.0","commit":{"sha":"7777777777777777777777777777777777777777"}},{"name":"v7.0.1","commit":{"sha":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}}]"#,
    ).await;

            assert_no_action_update(&output);
        }

        #[tokio::test]
        async fn action_tags_update_only_within_their_path_qualified_tag_lineage() {
            let output = resolve_action_source(
        "steps:\n  - uses: actions/checkout@release/action-v2.3.0\n",
        r#"[{"name":"v99.0.0"},{"name":"release/action-v2.4.0"},{"name":"release/action-v2.3.0"}]"#,
    )
    .await;

            assert_eq!(output.edits[0].new_text, "release/action-v2.4.0");
        }

        #[tokio::test]
        async fn does_not_resolve_major_action_ref_from_a_mismatched_major() {
            let session = standard_session();
            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml".to_owned(),
                        "yaml".to_owned(),
                        "steps:\n  - uses: actions/checkout@v3\n".to_owned(),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "actions/checkout".to_owned(),
                        GitHub,
                        r#"[{"name":"v4.2.0"},{"name":"v4.1.0"}]"#.to_owned(),
                    )],
                )
                .await;

            assert_no_action_update(&output);
        }

        #[tokio::test]
        async fn does_not_suggest_for_numeric_or_dotted_refs_without_a_matching_tag() {
            for requirement in ["2024", "2024.01", "${{ matrix.action_ref }}", "main"] {
                let session = standard_session();
                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///work/.github/workflows/ci.yml".to_owned(),
                            "yaml".to_owned(),
                            format!("steps:\n  - uses: actions/checkout@{requirement}\n"),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "actions/checkout".to_owned(),
                            GitHub,
                            github_action_tags(),
                        )],
                    )
                    .await;

                assert!(output.edits.is_empty(), "unexpected edit for {requirement}");
                assert!(
                    output
                        .suggestions
                        .iter()
                        .all(|suggestion| suggestion.status != "updateAvailable")
                );
            }
        }

        fn github_action_tags() -> String {
            r#"[{"name":"v4.2.0"},{"name":"v4.1.0"},{"name":"v4.0.0"},{"name":"v4"}]"#.to_owned()
        }

        #[tokio::test]
        async fn resolves_reusable_workflow_tags_using_repository_identity() {
            let session = standard_session();
            let input = DocumentInput::new(
                "file:///work/.github/workflows/release.yml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("reusable-workflow-tags.yaml"),
                None,
            );
            let dependencies = parse_document(&input);
            assert_eq!(dependencies.len(), 1);
            assert_eq!(
                dependencies[0].hosted_name.as_deref(),
                Some("acme/automation")
            );
            let response = RegistryResponseInput::new(
                "acme/automation".to_owned(),
                GitHub,
                r#"[{"name":"v1.3.0"},{"name":"v1"}]"#.to_owned(),
            );
            assert_eq!(
                session.latest_from_responses(&dependencies[0], std::slice::from_ref(&response)),
                Some("1.3.0".to_owned())
            );

            let output = session
                .resolve_document_with_responses(input, std::slice::from_ref(&response))
                .await;

            assert_update(&output, "v1.3.0");
        }

        #[tokio::test]
        async fn resolves_npm_github_dependencies_from_tags() {
            let output = resolve_github_fixture(
                "npm-github-dependencies-from-tags.json",
                "octokit/core.js",
                Npm,
                r#"[{"name":"v2.5.0"},{"name":"v1.9.0"}]"#,
            )
            .await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(
                output.edits[0].new_text,
                "github:octokit/core.js#semver:2.5.0"
            );
        }

        #[tokio::test]
        async fn resolves_npm_github_commit_dependencies_from_commits() {
            let output = resolve_github_fixture(
        "npm-github-commit-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            assert_update(&output, "github:owner/commit#abcdef1");
        }

        #[tokio::test]
        async fn resolves_npm_github_url_commit_dependencies_from_commits() {
            let output = resolve_github_fixture(
        "npm-github-url-commit-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            assert_update(&output, "git+https://github.com/owner/commit.git#abcdef1");
        }

        #[tokio::test]
        async fn resolves_npm_github_git_ssh_dependencies_from_commits() {
            let output = resolve_github_fixture(
        "npm-github-git-ssh-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            assert_update(&output, "git+ssh://git@github.com/owner/commit.git#abcdef1");
        }

        #[tokio::test]
        async fn resolves_npm_github_git_ssh_colon_dependencies_from_commits() {
            let output = resolve_github_fixture(
        "npm-github-git-ssh-colon-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            assert_update(&output, "git+ssh://git@github.com:owner/commit.git#abcdef1");
        }

        #[tokio::test]
        async fn leaves_npm_github_dependencies_without_refs_fixed() {
            let output = resolve_github_fixture(
        "npm-github-dependencies-without-refs-from-commits.json",
        "owner/bare",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            crate::test_support::assert_fixed_suggestion(&output, "branch reference");
        }

        #[tokio::test]
        async fn leaves_npm_github_branch_dependencies_fixed() {
            let output = resolve_github_fixture(
        "npm-github-branch-dependencies-fixed.json",
        "owner/branch",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            crate::test_support::assert_fixed_suggestion(&output, "branch reference");
        }

        #[tokio::test]
        async fn keeps_full_length_for_npm_github_sha_pins() {
            let output = resolve_github_fixture(
        "npm-github-full-sha-dependencies-from-commits.json",
        "owner/commit",
        Npm,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890123456789012345678901234567890"}]"#,
    )
    .await;

            assert_update(
                &output,
                "github:owner/commit#abcdef1234567890abcdef1234567890abcdef12",
            );
        }

        #[tokio::test]
        async fn routes_npm_github_tag_dependencies_to_tags() {
            let session = standard_session();
            let dependencies = parse_document(&DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("npm-github-tag-dependencies-to-tags.json"),
                None,
            ));

            assert_eq!(
                session.registry_urls(&dependencies[0]),
                vec!["https://api.github.com/repos/octokit/core.js/tags"]
            );
        }

        #[tokio::test]
        async fn routes_npm_github_commit_dependencies_to_commits() {
            let session = standard_session();
            let dependencies = parse_document(&DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("npm-github-commit-dependencies-to-commits.json"),
                None,
            ));

            assert_eq!(
                session.registry_urls(&dependencies[0]),
                vec!["https://api.github.com/repos/owner/commit/commits"]
            );
        }

        #[tokio::test]
        async fn resolves_ruby_github_tag_dependencies_from_tags() {
            let output = resolve_github_fixture(
                "ruby-github-tag-dependencies-from-tagsGemfile",
                "rspec/rspec-rails",
                Ruby,
                r#"[{"name":"v6.1.0"},{"name":"v6.0.1"}]"#,
            )
            .await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(output.edits[0].new_text, r#"tag: "v6.1.0""#);
        }

        #[tokio::test]
        async fn leaves_ruby_github_dependencies_without_ref_fixed() {
            let output = resolve_github_fixture(
        "ruby-github-dependencies-without-ref-from-commitsGemfile",
        "heartcombo/devise",
        Ruby,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            crate::test_support::assert_fixed_suggestion(&output, "branch reference");
        }

        #[tokio::test]
        async fn resolves_ruby_github_ref_dependencies_from_commits() {
            let output = resolve_github_fixture(
        "ruby-github-ref-dependencies-from-commitsGemfile",
        "rspec/rspec-core",
        Ruby,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            assert_update(&output, "abcdef1");
        }

        #[tokio::test]
        async fn keeps_full_length_for_ruby_github_sha_pins() {
            let output = resolve_github_fixture(
        "ruby-github-full-sha-ref-dependencies-from-commitsGemfile",
        "rspec/rspec-core",
        Ruby,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890123456789012345678901234567890"}]"#,
    )
    .await;

            assert_update(&output, "abcdef1234567890abcdef1234567890abcdef12");
        }

        #[tokio::test]
        async fn leaves_ruby_github_branch_and_named_refs_fixed() {
            for fixture in [
                "ruby-github-branch-dependencies-fixedGemfile",
                "ruby-github-named-ref-dependencies-fixedGemfile",
            ] {
                let output = resolve_github_fixture(
                    fixture,
                    "rspec/rspec-core",
                    Ruby,
                    r#"[{"sha":"abcdef1234567890"}]"#,
                )
                .await;

                crate::test_support::assert_fixed_suggestion(&output, "branch reference");
            }
        }

        #[tokio::test]
        async fn resolves_ruby_git_github_tag_dependencies_from_tags() {
            let output = resolve_github_fixture(
                "ruby-git-github-tag-dependencies-from-tagsGemfile",
                "rails/rails",
                Ruby,
                r#"[{"name":"v8.0.0"},{"name":"v7.0.0"}]"#,
            )
            .await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(output.edits[0].new_text, r#"tag: "v8.0.0""#);
        }

        #[tokio::test]
        async fn leaves_ruby_git_github_dependencies_without_ref_fixed() {
            let output = resolve_github_fixture(
        "ruby-git-github-dependencies-without-ref-from-commitsGemfile",
        "rails/rails",
        Ruby,
        r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"},{"sha":"1234567890abcdef1234567890abcdef12345678"}]"#,
    )
    .await;

            crate::test_support::assert_fixed_suggestion(&output, "branch reference");
        }

        async fn resolve_inline(
            uri: &str,
            text: &str,
            package: &str,
            ecosystem: Ecosystem,
            body: &str,
        ) -> ResolveDocumentOutput {
            let language = if uri.ends_with("Gemfile") {
                "ruby"
            } else {
                "json"
            };
            standard_session()
                .resolve_document_with_responses(
                    DocumentInput::new(uri.to_owned(), language.to_owned(), text.to_owned(), None),
                    &[RegistryResponseInput::new(
                        package.to_owned(),
                        ecosystem,
                        body.to_owned(),
                    )],
                )
                .await
        }

        #[tokio::test]
        async fn bumps_ruby_github_version_refs_to_tags() {
            let output = resolve_inline(
                "file:///Gemfile",
                r#"gem "rspec-core", github: "rspec/rspec-core", ref: "v3.12.0""#,
                "rspec/rspec-core",
                Ruby,
                r#"[{"name":"v3.13.0"},{"name":"v3.12.0"}]"#,
            )
            .await;

            assert_update(&output, "v3.13.0");
        }

        #[tokio::test]
        async fn leaves_hex_named_refs_unbumped_when_no_listed_commit_matches() {
            for (uri, text, package, ecosystem) in [
                (
                    "file:///Gemfile",
                    r#"gem "rspec-core", github: "rspec/rspec-core", ref: "deadbee""#,
                    "rspec/rspec-core",
                    Ruby,
                ),
                (
                    "file:///package.json",
                    r#"{ "dependencies": { "commit": "github:owner/commit#deadbee" } }"#,
                    "owner/commit",
                    Npm,
                ),
            ] {
                let output = resolve_inline(
                    uri,
                    text,
                    package,
                    ecosystem,
                    r#"[{"sha":"abcdef1234567890abcdef1234567890abcdef12"}]"#,
                )
                .await;

                assert_no_action_update(&output);
            }
        }

        #[tokio::test]
        async fn routes_ruby_github_ref_dependencies_to_commits() {
            let session = standard_session();
            let dependencies = parse_document(&DocumentInput::new(
                "file:///Gemfile".to_owned(),
                "ruby".to_owned(),
                package_file_fixture("ruby-github-ref-dependencies-to-commitsGemfile"),
                None,
            ));

            assert_eq!(
                session.registry_urls(&dependencies[0]),
                vec!["https://api.github.com/repos/rspec/rspec-core/commits"]
            );
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/github", name)
        }
        use super::assert_update;

        mod checking {
            use super::*;

            fn assert_local_reference_statuses(output: &ResolveDocumentOutput) {
                assert_eq!(
                    output
                        .suggestions
                        .iter()
                        .map(|suggestion| suggestion.status.as_str())
                        .collect::<Vec<_>>(),
                    ["fixed", "fixed", "error", "error"]
                );
                assert_eq!(output.edits.len(), 0);
            }

            #[tokio::test]
            async fn non_version_action_refs_are_fixed_without_a_request() {
                // The server answers one request if one comes; none must come.
                let (base_url, server) = github_api_server(vec![(200, r#"[{"name":"v1.0.0"}]"#)]);
                let output = github_session(base_url)
                    .resolve_document(DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        package_file_fixture("floating-action-ref.yaml"),
                        None,
                    ))
                    .await;

                assert_eq!(server.join().unwrap(), Vec::<String>::new());
                assert_eq!(output.suggestions[0].status, "fixed");
                assert_eq!(output.suggestions[0].latest.as_deref(), Some("stable"));
                assert_eq!(output.edits.len(), 0);
            }

            #[tokio::test]
            async fn local_action_references_use_the_declared_workspace_and_expressions_fail_explicitly()
             {
                let root = temp_dir().join(format!("versionlens-github-local-{}", id()));
                create_dir_all(root.join(".github/workflows")).unwrap();
                create_dir_all(root.join("actions/build")).unwrap();
                write(
                    root.join(".github/workflows/release.yml"),
                    "on: workflow_call\n",
                )
                .unwrap();
                write(
                    root.join("actions/build/action.yml"),
                    "runs: {using: composite, steps: []}\n",
                )
                .unwrap();
                let input = DocumentInput::new(
                    format!("file://{}/.github/workflows/ci.yml", root.display()),
                    "yaml",
                    package_file_fixture("local-action-reference-kinds.yaml"),
                    Some(root.to_string_lossy().into_owned()),
                );

                let output = session_without_vulnerabilities()
                    .resolve_document(input)
                    .await;

                assert_local_reference_statuses(&output);
                assert_eq!(
                    output.suggestions[0].latest.as_deref(),
                    Some("./.github/workflows/release.yml")
                );
                assert_eq!(
                    output.suggestions[1].latest.as_deref(),
                    Some("./actions/build")
                );
                assert!(
                    output.suggestions[2]
                        .latest
                        .as_deref()
                        .is_some_and(|value| value.contains("expression"))
                );
                assert_eq!(
                    output.suggestions[3].latest.as_deref(),
                    Some("invalid GitHub action reference")
                );
                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn unsaved_local_targets_are_verified_from_one_workspace_snapshot_without_network()
             {
                let root = temp_dir().join(format!("versionlens-github-unsaved-{}", id()));
                create_dir_all(&root).unwrap();
                let workspace_root = root.to_string_lossy().into_owned();
                let document = |relative: &str, text: &str, version| {
                    DocumentInput::new(
                        format!("file://{}/{relative}", root.display()),
                        "yaml",
                        text,
                        Some(workspace_root.clone()),
                    )
                    .with_version(version)
                };
                let action = document(
                    "actions/build/action.yml",
                    "name: build\nruns: {using: composite, steps: []}\n",
                    1,
                );
                let workflow = document(
                    ".github/workflows/release.yml",
                    "on:\n  workflow_call:\n",
                    1,
                );
                let input = document(
                    ".github/workflows/ci.yml",
                    &package_file_fixture("local-action-reference-kinds.yaml"),
                    1,
                );
                let (base_url, server) = github_api_server(Vec::<(u16, String)>::new());
                let session = github_session(base_url);
                assert!(session.set_workspace_documents(vec![action, workflow]));

                let output = session.resolve_document(input).await;

                assert_local_reference_statuses(&output);
                assert_eq!(server.join().unwrap().len(), 0);
                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn edited_unsaved_workflow_invalidates_a_cached_local_result() {
                let root = temp_dir().join(format!("versionlens-github-edit-{}", id()));
                create_dir_all(root.join(".github/workflows")).unwrap();
                write(root.join(".github/workflows/release.yml"), "on: push\n").unwrap();
                let workspace_root = root.to_string_lossy().into_owned();
                let target = |text: &str, version| {
                    DocumentInput::new(
                        format!("file://{}/.github/workflows/release.yml", root.display()),
                        "yaml",
                        text,
                        Some(workspace_root.clone()),
                    )
                    .with_version(version)
                };
                let input = DocumentInput::new(
                    format!("file://{}/.github/workflows/ci.yml", root.display()),
                    "yaml",
                    "jobs:\n  shared:\n    uses: ./.github/workflows/release.yml\n",
                    Some(workspace_root.clone()),
                );
                let session = session_without_vulnerabilities();
                assert!(
                    session.set_workspace_documents(vec![target("on:\n  workflow_call:\n", 1,)])
                );
                let valid = session.resolve_document(input.clone()).await;
                assert_eq!(valid.suggestions[0].status, "fixed");

                assert!(session.set_workspace_documents(vec![target("on: push\n", 2)]));
                let invalid = session.resolve_document(input).await;

                assert_eq!(invalid.suggestions[0].status, "error");
                assert!(
                    invalid.suggestions[0]
                        .latest
                        .as_deref()
                        .is_some_and(|value| value.contains("invalid"))
                );
                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn local_action_references_report_unavailable_roots_and_ambiguous_metadata() {
                let without_root = session_without_vulnerabilities()
                    .resolve_document(DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        "steps:\n  - uses: ./actions/build\n",
                        None,
                    ))
                    .await;
                assert_eq!(without_root.suggestions[0].status, "error");
                assert!(
                    without_root.suggestions[0]
                        .latest
                        .as_deref()
                        .is_some_and(|value| value.contains("workspace root unavailable"))
                );

                let root = temp_dir().join(format!("versionlens-github-contained-{}", id()));
                create_dir_all(&root).unwrap();
                let escaped = session_without_vulnerabilities()
                    .resolve_document(DocumentInput::new(
                        format!("file://{}/.github/workflows/ci.yml", root.display()),
                        "yaml",
                        "steps:\n  - uses: ./../outside\n",
                        Some(root.to_string_lossy().into_owned()),
                    ))
                    .await;
                assert_eq!(escaped.suggestions[0].status, "error");
                assert_eq!(
                    escaped.suggestions[0].latest.as_deref(),
                    Some("invalid local GitHub action path")
                );
                remove_dir_all(root).unwrap();

                let root = temp_dir().join(format!("versionlens-github-ambiguous-{}", id()));
                create_dir_all(root.join("actions/build")).unwrap();
                write(root.join("actions/build/action.yml"), "name: first\n").unwrap();
                write(root.join("actions/build/action.yaml"), "name: second\n").unwrap();
                let output = session_without_vulnerabilities()
                    .resolve_document(DocumentInput::new(
                        format!("file://{}/.github/workflows/ci.yml", root.display()),
                        "yaml",
                        "steps:\n  - uses: ./actions/build\n",
                        Some(root.to_string_lossy().into_owned()),
                    ))
                    .await;
                assert_eq!(output.suggestions[0].status, "error");
                assert!(
                    output.suggestions[0]
                        .latest
                        .as_deref()
                        .is_some_and(|value| value.contains("ambiguous"))
                );
                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn ignored_and_excluded_local_actions_are_never_read() {
                let root = temp_dir().join(format!("versionlens-github-ignored-{}", id()));
                for action in ["vendor/build", "actions/build"] {
                    create_dir_all(root.join(action)).unwrap();
                    write(
                        root.join(action).join("action.yml"),
                        "runs: {using: composite, steps: []}\n",
                    )
                    .unwrap();
                }
                write(root.join(".gitignore"), "vendor/\n").unwrap();
                let input = |action: &str| {
                    DocumentInput::new(
                        format!("file://{}/.github/workflows/ci.yml", root.display()),
                        "yaml",
                        format!("steps:\n  - uses: ./{action}\n"),
                        Some(root.to_string_lossy().into_owned()),
                    )
                };
                let session = session_without_vulnerabilities();
                let statuses = async |action| {
                    session
                        .resolve_document(input(action))
                        .await
                        .suggestions
                        .into_iter()
                        .map(|suggestion| suggestion.status)
                        .collect::<Vec<_>>()
                };

                assert_eq!(statuses("vendor/build").await.len(), 0);
                assert_eq!(statuses("actions/build").await, ["fixed"]);
                session.set_workspace_exclusions(vec!["**/actions".to_owned()]);
                assert_eq!(statuses("actions/build").await.len(), 0);
                assert_eq!(statuses("missing/build").await, ["error"]);
                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn escaped_workflow_references_preserve_surrounding_yaml_on_update() {
                let source = r#"steps: [{uses: "acme\u002faction\u0040v\u0031.0.0", name: keep}] # retained"#;
                let output = session_without_vulnerabilities()
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///work/.github/workflows/ci.yml",
                            "yaml",
                            source,
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "acme/action",
                            GitHub,
                            r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#,
                        )],
                    )
                    .await;
                assert_eq!(output.edits.len(), 1, "{output:?}");
                let edit = &output.edits[0];
                let mut updated = source.to_owned();
                updated.replace_range(
                    edit.range.start.character as usize..edit.range.end.character as usize,
                    &edit.new_text,
                );
                assert_eq!(
                    updated,
                    r#"steps: [{uses: "acme\u002faction\u0040v2.0.0", name: keep}] # retained"#
                );
            }

            #[tokio::test]
            async fn cached_action_results_are_isolated_by_tag_family_and_commit() {
                let session = session_without_vulnerabilities();
                let input = |reference: &str| {
                    DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        format!("steps:\n  - uses: acme/action@{reference}\n"),
                        None,
                    )
                };
                let first = input("v1.0.0");
                session
                    .resolve_document_with_responses(
                        first,
                        &[RegistryResponseInput::new(
                            "acme/action".to_owned(),
                            GitHub,
                            r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned(),
                        )],
                    )
                    .await;
                for reference in [
                    "release-v1.0.0",
                    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v1.0.0",
                ] {
                    let dependency = parse_document(&input(reference)).remove(0);
                    assert!(
                        session
                            .cached_resolved_suggestion(
                                &dependency,
                                &cache_scope(&session, &input(reference))
                            )
                            .is_none()
                    );
                }
            }

            #[tokio::test]
            async fn github_tag_resolution_reads_subsequent_pages() {
                let page = serde_json::Value::Array(
                    (0..30)
                        .map(|index| serde_json::json!({"name": format!("unrelated-v1.0.{index}")}))
                        .collect(),
                )
                .to_string();
                let (base_url, server) = github_api_server(vec![
                    (200, page),
                    (200, r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned()),
                ]);
                let session = github_session(base_url);
                let output = session
                    .resolve_document(DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        "steps:\n  - uses: acme/action@v1.0.0\n",
                        None,
                    ))
                    .await;
                assert_eq!(
                    server.join().unwrap(),
                    [
                        "/repos/acme/action/tags",
                        "/repos/acme/action/tags?per_page=30&page=2"
                    ]
                );
                assert_eq!(output.edits[0].new_text, "v2.0.0");
            }

            #[tokio::test]
            async fn cached_action_suggestions_use_the_current_document_ranges() {
                let session = session_without_vulnerabilities();
                let document = |text: &str| {
                    DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", text, None)
                };
                session
                    .resolve_document_with_responses(
                        document("steps:\n  - uses: acme/action@v1.0.0\n"),
                        &[RegistryResponseInput::new(
                            "acme/action".to_owned(),
                            GitHub,
                            r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned(),
                        )],
                    )
                    .await;
                let moved = document("# moved\nsteps:\n  - uses: acme/action@v1.0.0\n");
                let dependency = parse_document(&moved).remove(0);
                let cached = session
                    .cached_resolved_suggestion(&dependency, &cache_scope(&session, &moved))
                    .unwrap();
                assert_eq!(
                    cached.dependency.requirement_range,
                    dependency.requirement_range
                );
                assert_eq!(cached.dependency.requirement_range.start.line, 2);
            }

            fn cache_scope(
                session: &VersionLensSession,
                input: &DocumentInput,
            ) -> crate::session::CacheScope {
                let context = session.registry_context(input, session.classify_document(input));
                session.document_cache_scope(&context, input)
            }

            #[tokio::test]
            async fn nested_annotated_tags_resolve_to_the_pinned_commit() {
                let commit = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
                let outer = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
                let inner = "cccccccccccccccccccccccccccccccccccccccc";
                let mut responses = annotated_tag_start(outer);
                responses.extend([
                    (
                        200,
                        serde_json::json!({"sha":outer,"object":{"type":"tag","sha":inner}})
                            .to_string(),
                    ),
                    (
                        200,
                        serde_json::json!({"sha":inner,"object":{"type":"commit","sha":commit}})
                            .to_string(),
                    ),
                ]);
                let (base_url, server) = github_api_server(responses);
                let output = github_session(base_url)
                    .resolve_document(DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        format!("steps:\n  - uses: acme/action@{commit} # v2\n"),
                        None,
                    ))
                    .await;
                assert_eq!(
                    server.join().unwrap(),
                    [
                        "/repos/acme/action/tags".to_owned(),
                        "/repos/acme/action/git/ref/tags/v2".to_owned(),
                        format!("/repos/acme/action/git/tags/{outer}"),
                        format!("/repos/acme/action/git/tags/{inner}"),
                    ]
                );
                assert_eq!(output.suggestions[0].status, "current");
            }

            #[tokio::test]
            async fn cyclic_annotated_tags_finish_with_a_specific_failure() {
                let tag = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
                let mut responses = annotated_tag_start(tag);
                responses.push((
                    200,
                    serde_json::json!({"sha":tag,"object":{"type":"tag","sha":tag}}).to_string(),
                ));
                let (base_url, server) = github_api_server(responses);
                let output = github_session(base_url)
        .resolve_document(DocumentInput::new(
            "file:///work/.github/workflows/release.yml",
            "yaml",
            "steps:\n  - uses: acme/action@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa # v2\n",
            None,
        ))
        .await;
                assert_eq!(server.join().unwrap().len(), 3);
                assert_eq!(output.edits.len(), 0);
                assert_eq!(output.suggestions[0].status, "error");
                assert!(
                    output.suggestions[0]
                        .latest
                        .as_deref()
                        .unwrap()
                        .contains("cyclic GitHub tag reference")
                );
            }

            #[tokio::test]
            async fn commit_pins_resolve_release_families_and_keep_pin_format() {
                let current = "a".repeat(40);
                let latest = "b".repeat(40);
                let body = commit_tags(&current, &latest);
                let responses = [RegistryResponseInput::new("acme/action", GitHub, &body)];
                let session = session_without_vulnerabilities();
                for (pin, expected_status) in [(&current, "updateAvailable"), (&latest, "current")]
                {
                    let input = DocumentInput::new(
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        format!("steps: [{{uses: 'acme/action@{pin}', name: keep}}] # preserved\n"),
                        None,
                    );
                    let output = session
                        .resolve_document_with_responses(input, &responses)
                        .await;
                    assert_eq!(output.suggestions.len(), 1, "{output:?}");
                    assert_eq!(output.suggestions[0].status, expected_status, "{output:?}");
                    if pin == &current {
                        assert_eq!(output.edits.len(), 1);
                        assert_eq!(output.edits[0].new_text, latest);
                        assert_eq!(
                            output.edits[0].range.end.character
                                - output.edits[0].range.start.character,
                            40
                        );
                    } else {
                        assert_eq!(output.edits.len(), 0);
                    }
                }
            }

            #[tokio::test]
            async fn commit_pins_with_multiple_tag_families_have_a_checked_outcome() {
                let commit = "a".repeat(40);
                let body = serde_json::json!([
                    {"name":"v1.0.0","commit":{"sha":commit}},
                    {"name":"release-v3.0.0","commit":{"sha":commit}}
                ])
                .to_string();
                let session = session_without_vulnerabilities();
                let input = DocumentInput::new(
                    "file:///work/.github/workflows/ci.yml",
                    "yaml",
                    format!("steps:\n  - uses: acme/action@{commit}\n"),
                    None,
                );
                let output = session
                    .resolve_document_with_responses(
                        input.clone(),
                        &[RegistryResponseInput::new("acme/action", GitHub, &body)],
                    )
                    .await;
                assert_eq!(output.suggestions.len(), 1);
                assert_eq!(output.suggestions[0].status, "error");
                assert_eq!(output.edits.len(), 0);
                assert!(session.document_is_fresh(&input));
            }

            #[tokio::test]
            async fn abbreviated_action_commit_is_checked_through_the_repository_endpoint() {
                let current = "a".repeat(40);
                let latest = "b".repeat(40);
                let body = commit_tags(&current, &latest);
                let (base_url, server) = github_api_server(vec![(200, body)]);
                let output = github_session(base_url)
                    .resolve_document(DocumentInput::new(
                        "file:///work/.github/workflows/release.yml",
                        "yaml",
                        "steps:\n  - uses: acme/action@aaaaaaa # reviewed\n",
                        None,
                    ))
                    .await;
                assert_eq!(server.join().unwrap(), ["/repos/acme/action/tags"]);
                assert_eq!(output.edits.len(), 1, "{output:?}");
                assert_eq!(output.edits[0].new_text, latest);
            }

            #[tokio::test]
            async fn action_commit_selections_require_a_verified_release_replacement() {
                let current = "a".repeat(40);
                let body = serde_json::json!([
                    {"name":"v1.0.0","commit":{"sha":"c".repeat(40)}},
                    {"name":"v2.0.0","commit":{"sha":current}},
                    {"name":"v3.0.0","commit":{"sha":"b".repeat(40)}}
                ])
                .to_string();
                let responses = [RegistryResponseInput::new("acme/action", GitHub, &body)];
                let input = DocumentInput::new(
                    "file:///work/.github/workflows/ci.yml",
                    "yaml",
                    format!("steps:\n  - uses: acme/action@{current}\n"),
                    None,
                );
                for version in ["1.0.0", "99.0.0"] {
                    let output = session_without_vulnerabilities()
                        .apply_command_with_selected_version(crate::ApplyCommandRequest {
                            input: input.clone(),
                            command: Some("update"),
                            dependency_name: Some("acme/action"),
                            selected_version: Some(version),
                            responses: &responses,
                        })
                        .await;
                    assert!(output.edits.is_empty(), "{output:?}");
                }
            }

            fn annotated_tag_start(tag: &str) -> Vec<(u16, String)> {
                vec![
                    (200, r#"[{"name":"v2"}]"#.to_owned()),
                    (
                        200,
                        serde_json::json!({"ref":"refs/tags/v2","object":{"type":"tag","sha":tag}})
                            .to_string(),
                    ),
                ]
            }

            fn commit_tags(current: &str, latest: &str) -> String {
                serde_json::json!([
                    {"name":"v1.0.0","commit":{"sha":current}},
                    {"name":"v2.0.0","commit":{"sha":latest}}
                ])
                .to_string()
            }
        }

        fn github_session(base_url: String) -> VersionLensSession {
            crate::test_support::session_with_provider_settings(
                ProviderSettings {
                    registry_urls: vec![RegistryUrlConfig {
                        ecosystem: GitHub,
                        url: base_url,
                    }],
                    ..Default::default()
                },
                false,
            )
        }
    }
    mod go {
        use super::*;

        #[tokio::test]
        async fn go_mod_exclude_versions_are_fixed_without_registry_updates() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///go.mod".to_owned(),
                        "go.mod".to_owned(),
                        package_file_fixture(
                            "go-mod-exclude-versions-are-fixed-without-registry-updates.mod",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "example.test/bad".to_owned(),
                        Go,
                        "v1.0.0\nv1.1.0\n".to_owned(),
                    )],
                )
                .await;

            crate::test_support::assert_fixed_suggestion(&output, "excluded version");
        }

        fn assert_go_proxy_configuration(env: &str, fixture: &str, expected_urls: &[&str]) {
            let root = temp_dir().join(format!(
                "versionlens-go-proxy-{}-{}",
                id(),
                fixture.replace('.', "-")
            ));
            create_dir_all(&root).unwrap();
            write(root.join(".env"), env).unwrap();
            let input = DocumentInput::new(
                format!("file://{}", root.join("go.mod").display()),
                "go.mod".to_owned(),
                package_file_fixture(fixture),
                Some(root.to_string_lossy().into_owned()),
            );
            registry_case_with_expected_urls(&input, &[expected_urls]);
            remove_dir_all(root).unwrap();
        }

        #[test]
        fn go_mod_uses_workspace_go_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=https://proxy.example.test/,direct|https://fallback.example.test|off\n",
                "go-mod-uses-workspace-go-proxy-urls.txt",
                &[
                    "https://proxy.example.test/!go.uber.org/!zap/@v/list",
                    "https://proxy.example.test/!go.uber.org/!zap/@latest",
                    "https://fallback.example.test/!go.uber.org/!zap/@v/list",
                    "https://fallback.example.test/!go.uber.org/!zap/@latest",
                ],
            );
        }

        #[test]
        fn go_mod_goproxy_off_disables_default_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=off\n",
                "go-mod-goproxy-off-disables-default-proxy-urls.txt",
                &[],
            );
        }

        #[test]
        fn go_mod_goproxy_direct_disables_default_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=direct\n",
                "go-mod-goproxy-direct-disables-default-proxy-urls.txt",
                &[],
            );
        }

        #[test]
        fn go_mod_goprivate_dependencies_do_not_use_module_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=corp.example.com\n",
                "go-mod-goprivate-dependencies-do-not-use-module-proxy-urls.txt",
                &[],
            );
        }

        #[test]
        fn go_mod_goprivate_path_match_patterns_disable_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=[a-z].corp.example.com\n",
                "go-mod-goprivate-path-match-patterns-disable-proxy-urls.txt",
                &[],
            );
        }

        #[test]
        fn go_mod_goproxy_off_in_list_disables_later_proxy_urls() {
            assert_go_proxy_configuration(
                "GOPROXY=off,https://proxy.example.test\n",
                "go-mod-goproxy-off-in-list-disables-later-proxy-urls.txt",
                &[],
            );
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/go", name)
        }
    }
    mod hex {
        use super::{DocumentInput, session_without_vulnerabilities};
        use crate::RegistryResponseInput;
        use versionlens_model::Ecosystem::Hex;

        #[tokio::test]
        async fn resolves_mix_hex_alias_dependencies_against_target_package() {
            let session = session_without_vulnerabilities();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///mix.exs".to_owned(),
                        "elixir".to_owned(),
                        package_file_fixture(
                            "resolves-mix-hex-alias-dependencies-against-target-package.exs",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "plug".to_owned(),
                        Hex,
                        r#"{"releases":[{"version":"2.0.0"},{"version":"1.20.0"}]}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "plug");
            assert_eq!(
                output.suggestions[0].dependency.hosted_name.as_deref(),
                Some("plug_alias")
            );
            assert_update(&output, "2.0.0");
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/hex", name)
        }
        use super::assert_update;
    }
    mod maven {
        use super::*;

        #[tokio::test]
        async fn maven_registry_urls_preserve_configured_fallback_order() {
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings {
                    registry_urls: vec![
                        RegistryUrlConfig {
                            ecosystem: Maven,
                            url: "https://mirror.example.test/maven2".to_owned(),
                        },
                        RegistryUrlConfig {
                            ecosystem: Maven,
                            url: "https://repo.maven.apache.org/maven2".to_owned(),
                        },
                    ],
                    ..Default::default()
                },
                false,
            );
            let input = DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                package_file_fixture("urls-preserve-configured-fallback-order.xml"),
                None,
            );
            let dependencies = parse_document(&input);

            assert_eq!(
                session.registry_urls(&dependencies[0]),
                vec![
                    "https://mirror.example.test/maven2/org/example/demo/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
                ]
            );

            let output = session.resolve_document_with_responses(
        input,
        &[
            RegistryResponseInput::new("org.example:demo".to_owned(), Maven, "<metadata><versioning><versions></versions></versioning></metadata>"
                    .to_owned()),
            RegistryResponseInput::new("org.example:demo".to_owned(), Maven, "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>"
                    .to_owned()),
        ],
    ).await;

            assert_update(&output, "1.1.0");
        }

        #[test]
        fn maven_registry_urls_include_pom_repositories_before_central() {
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                package_file_fixture("urls-include-pom-repositories-before-central.xml"),
                None,
            );
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);

            assert_maven_urls(
                &session,
                &context,
                &dependencies,
                &[
                    "https://packages.example.test/maven/org/example/demo/maven-metadata.xml",
                    "https://profile.example.test/releases/org/example/demo/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
                ],
            );
        }

        #[test]
        fn maven_registry_urls_include_pom_plugin_repositories_before_central() {
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                package_file_fixture("urls-include-pom-plugin-repositories-before-central.xml"),
                None,
            );
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);

            assert_maven_urls(
                &session,
                &context,
                &dependencies,
                &[
                    "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
                ],
            );
        }

        #[test]
        fn maven_registry_urls_resolve_project_and_parent_interpolation_properties() {
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                crate::test_support::fixture(
                    "tests/fixtures/versionlens-parsers/src/maven_xml/tests",
                    "resolves-maven-project-and-parent-interpolation-properties.xml",
                ),
                None,
            );
            let dependencies = parse_document(&input);

            assert_eq!(dependencies[1].name, "org.parent:runtime");
            assert_eq!(dependencies[1].requirement, "3.4.5");
            assert_eq!(
                session.registry_urls(&dependencies[1]),
                vec!["https://repo.maven.apache.org/maven2/org/parent/runtime/maven-metadata.xml"]
            );
        }

        #[test]
        fn maven_documents_use_workspace_settings_repositories_and_auth() {
            let root = temp_dir().join(format!("versionlens-maven-settings-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join("settings.xml"),
                settings_with_private_repository(
                    r"<servers>
    <server>
      <id>private</id>
      <username>user</username>
      <password>pass</password>
    </server>
  </servers>",
                ),
            )
            .unwrap();

            let (session, context, dependencies) = workspace_dependencies(
                &root,
                "maven-documents-use-workspace-settings-repositories-and-auth.txt",
            );

            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec![
                    "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
                ]
            );

            let headers = context.auth_headers_for_url(
        Maven,
        "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
    );
            assert_eq!(headers.len(), 1);
            assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn maven_documents_use_workspace_settings_plugin_repositories_and_auth() {
            let root = temp_dir().join(format!("versionlens-maven-plugin-settings-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join("settings.xml"),
                r"<settings>
  <servers>
    <server>
      <id>private-plugins</id>
      <username>plugin-user</username>
      <password>plugin-pass</password>
    </server>
  </servers>
  <profiles>
    <profile>
      <pluginRepositories>
        <pluginRepository>
          <id>private-plugins</id>
          <url>https://plugins.example.test/maven</url>
        </pluginRepository>
      </pluginRepositories>
    </profile>
  </profiles>
</settings>",
            )
            .unwrap();

            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = workspace_pom_input(
                &root,
                "maven-documents-use-workspace-settings-plugin-repositories-and-auth.txt",
            );
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);

            assert_maven_urls(
                &session,
                &context,
                &dependencies,
                &[
                    "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
                ],
            );

            let headers = context.auth_headers_for_url(
                Maven,
                "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
            );
            assert_eq!(headers.len(), 1);
            assert_eq!(headers[0].value, "Basic cGx1Z2luLXVzZXI6cGx1Z2luLXBhc3M=");

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn maven_documents_use_only_active_workspace_settings_profile_repositories() {
            let root = temp_dir().join(format!(
                "versionlens-maven-active-profile-settings-{}",
                id()
            ));
            create_dir_all(&root).unwrap();
            write(
                root.join("settings.xml"),
                r"<settings>
  <profiles>
    <profile>
      <id>active</id>
      <repositories>
        <repository>
          <id>active-repo</id>
          <url>https://active.example.test/maven</url>
        </repository>
      </repositories>
    </profile>
    <profile>
      <id>inactive</id>
      <repositories>
        <repository>
          <id>inactive-repo</id>
          <url>https://inactive.example.test/maven</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
  <activeProfiles>
    <activeProfile>active</activeProfile>
  </activeProfiles>
</settings>",
            )
            .unwrap();

            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = workspace_pom_input(
                &root,
                "maven-documents-use-only-active-workspace-settings-profile-repositories.txt",
            );
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);

            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec![
                    "https://active.example.test/maven/org/example/demo/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
                ]
            );

            remove_dir_all(root).unwrap();
        }

        #[tokio::test]
        async fn maven_local_repository_metadata_resolves_before_remote_registries() {
            let root = temp_dir().join(format!("versionlens-maven-local-{}", id()));
            let local_repo = root.join(".m2").join("repository");
            let metadata_dir = local_repo.join("org").join("example").join("demo");
            create_dir_all(&metadata_dir).unwrap();
            write(
                root.join("settings.xml"),
                format!(
                    "<settings><localRepository>{}</localRepository></settings>",
                    local_repo.display()
                ),
            )
            .unwrap();
            write(
        metadata_dir.join("maven-metadata.xml"),
        "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>",
    )
    .unwrap();

            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = workspace_pom_input(
                &root,
                "maven-local-repository-metadata-resolves-before-remote-registries.txt",
            );
            let output = session.resolve_document(input).await;

            crate::test_support::assert_suggestion(&output, 0, "updateAvailable", Some("1.1.0"));
            assert_eq!(output.edits[0].new_text, "1.1.0");

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn maven_settings_mirror_overrides_pom_and_settings_repositories() {
            let root = temp_dir().join(format!("versionlens-maven-mirror-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join("settings.xml"),
                settings_with_private_repository(
                    r"<servers>
    <server>
      <id>internal</id>
      <username>mirror-user</username>
      <password>mirror-pass</password>
    </server>
  </servers>
  <mirrors>
    <mirror>
      <id>internal</id>
      <mirrorOf>*</mirrorOf>
      <url>https://maven.example.test/mirror</url>
    </mirror>
  </mirrors>",
                ),
            )
            .unwrap();

            let (session, context, dependencies) = workspace_dependencies(
                &root,
                "maven-settings-mirror-overrides-pom-and-settings-repositories.txt",
            );

            assert_maven_urls(
                &session,
                &context,
                &dependencies,
                &["https://maven.example.test/mirror/org/example/demo/maven-metadata.xml"],
            );

            let headers = context.auth_headers_for_url(
                Maven,
                "https://maven.example.test/mirror/org/example/demo/maven-metadata.xml",
            );
            assert_eq!(headers.len(), 1);
            assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn maven_settings_exact_mirror_replaces_matching_repository_only() {
            let root = temp_dir().join(format!("versionlens-maven-exact-mirror-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join("settings.xml"),
                r"<settings>
  <servers>
    <server>
      <id>private-mirror</id>
      <username>mirror-user</username>
      <password>mirror-pass</password>
    </server>
  </servers>
  <mirrors>
    <mirror>
      <id>private-mirror</id>
      <mirrorOf>private</mirrorOf>
      <url>https://maven.example.test/private-mirror</url>
    </mirror>
  </mirrors>
</settings>",
            )
            .unwrap();

            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = workspace_pom_input(
                &root,
                "maven-settings-exact-mirror-replaces-matching-repository-only.txt",
            );
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);

            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec![
                    "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
                    "https://public.example.test/maven/org/example/demo/maven-metadata.xml",
                    "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
                ]
            );

            let headers = context.auth_headers_for_url(
                Maven,
                "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
            );
            assert_eq!(headers.len(), 1);
            assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

            remove_dir_all(root).unwrap();
        }

        fn workspace_pom_input(root: &std::path::Path, fixture: &str) -> DocumentInput {
            DocumentInput::new(
                format!("file://{}", root.join("pom.xml").display()),
                "xml".to_owned(),
                package_file_fixture(fixture),
                Some(root.to_string_lossy().into_owned()),
            )
        }

        fn workspace_dependencies(
            root: &std::path::Path,
            fixture: &str,
        ) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
            let session = crate::test_support::session_with_provider_settings(
                ProviderSettings::default(),
                false,
            );
            let input = workspace_pom_input(root, fixture);
            let (context, dependencies) =
                crate::test_support::registry_context_and_dependencies(&session, &input);
            (session, context, dependencies)
        }

        fn assert_maven_urls(
            session: &VersionLensSession,
            context: &RegistryContext,
            dependencies: &[Dependency],
            expected: &[&str],
        ) {
            assert_dependency_registry_url(session, dependencies, 0, context, expected);
        }

        fn settings_with_private_repository(prefix: &str) -> String {
            format!(
                r"<settings>{prefix}
  <profiles>
    <profile>
      <repositories>
        <repository>
          <id>private</id>
          <url>https://maven.example.test/repository/releases</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>"
            )
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/maven", name)
        }
        use super::assert_update;
    }
    mod npm {
        use super::*;

        mod cache {
            use super::*;
            use std::sync::Arc;
            use std::sync::atomic::{AtomicBool, AtomicUsize};
            use std::thread::current;
            use std::thread::sleep;
            use std::thread::spawn;
            use std::time::Duration;

            #[tokio::test]
            async fn registry_context_resolution_reuses_cached_response_body_by_url() {
                let root = temp_dir().join(format!(
                    "versionlens-request-cache-{}-{}",
                    id(),
                    current().name().unwrap_or("test")
                ));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "registry=https://registry.example.test/\n",
                )
                .unwrap();

                let session = session_without_vulnerabilities();
                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "registry-context-resolution-reuses-cached-response-body-by-url.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );

                let first = session
                    .resolve_document_with_responses(
                        input.clone(),
                        &[crate::test_support::npm_latest_response(
                            "left-pad", "1.1.0",
                        )],
                    )
                    .await;
                let second = session.resolve_document_with_responses(input, &[]).await;

                assert_eq!(first.edits[0].new_text, "1.1.0");
                assert_eq!(second.edits[0].new_text, "1.1.0");

                remove_dir_all(root).unwrap();
            }

            #[tokio::test]
            async fn concurrent_registry_resolution_deduplicates_inflight_request_body_fetches() {
                use std::io::{Read, Write};
                use std::sync::atomic::Ordering::SeqCst;

                let listener = crate::test_support::tcp_listener_bind("127.0.0.1:0").unwrap();
                listener.set_nonblocking(true).unwrap();
                let registry_url = format!("http://{}/", listener.local_addr().unwrap());
                let request_count = Arc::new(<AtomicUsize>::default());
                let stop = Arc::new(<AtomicBool>::default());
                let server_request_count = crate::test_support::clone_arc(&request_count);
                let server_stop = crate::test_support::clone_arc(&stop);
                let server = spawn(move || {
                    while !server_stop.load(SeqCst) {
                        let Ok((mut stream, _)) = listener.accept() else {
                            sleep(Duration::from_millis(5));
                            continue;
                        };
                        server_request_count.fetch_add(1, SeqCst);
                        let mut buffer = [0_u8; 1024];
                        drop(stream.read(&mut buffer));
                        sleep(Duration::from_millis(75));
                        let body = r#"{"dist-tags":{"latest":"1.1.0"}}"#;
                        let response = format!(
                            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        stream.write_all(response.as_bytes()).unwrap();
                    }
                });

                let root = temp_dir().join(format!(
                    "versionlens-inflight-request-cache-{}-{}",
                    id(),
                    current().name().unwrap_or("test")
                ));
                create_dir_all(&root).unwrap();
                write(root.join(".npmrc"), format!("registry={registry_url}\n")).unwrap();

                let session = session_without_vulnerabilities();
                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "concurrent-registry-resolution-deduplicates-inflight-request-body-fetches.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let barrier = Arc::new(tokio::sync::Barrier::new(3));
                let first_session = session.clone();
                let first_barrier = crate::test_support::clone_arc(&barrier);
                let first_input = input.clone();
                let first = tokio::spawn(async move {
                    first_barrier.wait().await;
                    first_session
                        .resolve_document_with_responses(first_input, &[])
                        .await
                });
                let second_session = session.clone();
                let second_barrier = crate::test_support::clone_arc(&barrier);
                let second = tokio::spawn(async move {
                    second_barrier.wait().await;
                    second_session
                        .resolve_document_with_responses(input, &[])
                        .await
                });
                barrier.wait().await;
                let (first, second) = tokio::join!(first, second);
                assert_eq!(first.unwrap().edits[0].new_text, "1.1.0");
                assert_eq!(second.unwrap().edits[0].new_text, "1.1.0");

                stop.store(true, SeqCst);
                server.join().unwrap();
                assert_eq!(request_count.load(SeqCst), 1);

                remove_dir_all(root).unwrap();
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/npm_request_cache",
                    name,
                )
            }

            #[tokio::test]
            async fn background_resolution_runs_twelve_registry_requests_and_preserves_order() {
                use std::io::{Read, Write};
                use std::sync::atomic::Ordering::SeqCst;
                use std::sync::{Condvar, Mutex};
                use std::time::Duration;

                const UNIQUE_REQUESTS: usize = 12;
                let listener = crate::test_support::tcp_listener_bind("127.0.0.1:0").unwrap();
                let registry_url = format!("http://{}/", listener.local_addr().unwrap());
                let in_flight = Arc::new((Mutex::new(0_usize), Condvar::new()));
                let request_count = Arc::new(AtomicUsize::new(0));
                let server_in_flight = crate::test_support::clone_arc(&in_flight);
                let server_request_count = crate::test_support::clone_arc(&request_count);
                let server = spawn(move || {
                    let mut workers = Vec::new();
                    for _ in 0..UNIQUE_REQUESTS {
                        let Some(mut stream) =
                            versionlens_test_support::accept_within_deadline(&listener)
                        else {
                            break;
                        };
                        let worker_in_flight = crate::test_support::clone_arc(&server_in_flight);
                        let worker_request_count =
                            crate::test_support::clone_arc(&server_request_count);
                        workers.push(spawn(move || {
                            let mut buffer = [0_u8; 1024];
                            assert!(stream.read(&mut buffer).unwrap() > 0);
                            worker_request_count.fetch_add(1, SeqCst);
                            let (lock, ready) = &*worker_in_flight;
                            let mut count = lock.lock().unwrap();
                            *count += 1;
                            ready.notify_all();
                            while *count < UNIQUE_REQUESTS {
                                let (next, timeout) =
                                    ready.wait_timeout(count, Duration::from_secs(5)).unwrap();
                                assert!(
                                    !timeout.timed_out(),
                                    "registry requests did not run concurrently"
                                );
                                count = next;
                            }
                            drop(count);
                            let body = r#"{"dist-tags":{"latest":"2.0.0"}}"#;
                            write!(
                    stream,
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                    body.len()
                )
                .unwrap();
                        }));
                    }
                    for worker in workers {
                        worker.join().unwrap();
                    }
                });

                let dependencies = (0..UNIQUE_REQUESTS)
                    .map(|index| format!(r#""pkg{index}":"1.0.0""#))
                    .chain(std::iter::once(r#""alias":"npm:pkg0@1.0.0""#.to_owned()))
                    .collect::<Vec<_>>()
                    .join(",");
                let mut config =
                    crate::test_support::session_config(ProviderSettings::default(), false);
                config.providers.registry_urls = vec![RegistryUrlConfig {
                    ecosystem: Npm,
                    url: registry_url,
                }];
                let mut session = VersionLensSession::new(config);
                session.storage_state.task_priority = crate::concurrency::Priority::Background;
                let output = session
                    .resolve_document(DocumentInput::new(
                        "file:///package.json",
                        "json",
                        format!(r#"{{"dependencies":{{{dependencies}}}}}"#),
                        None,
                    ))
                    .await;
                server.join().unwrap();

                let names = output
                    .suggestions
                    .iter()
                    .map(|suggestion| suggestion.dependency.name.as_str())
                    .collect::<Vec<_>>();
                let expected = (0..UNIQUE_REQUESTS)
                    .map(|index| format!("pkg{index}"))
                    .chain(std::iter::once("pkg0".to_owned()))
                    .collect::<Vec<_>>();
                assert_eq!(names, expected);
                assert_eq!(request_count.load(SeqCst), UNIQUE_REQUESTS);
            }
        }
        mod dist_tags {
            use super::{DocumentInput, session_without_vulnerabilities, standard_session};
            use crate::RegistryResponseInput;
            use versionlens_model::Ecosystem::Npm;

            #[tokio::test]
            async fn npm_latest_dist_tag_caps_stable_update_choices() {
                let session = session_without_vulnerabilities();
                let input = DocumentInput::new(
                    "file:///package.json".to_owned(),
                    "json".to_owned(),
                    package_file_fixture("latest-dist-tag-caps-stable-update-choices.json"),
                    None,
                );

                let output = session
                    .resolve_document_with_responses(
                        input.clone(),
                        &[RegistryResponseInput::new(
                            "left-pad".to_owned(),
                            Npm,
                            r#"{
              "dist-tags": { "latest": "7.0.0" },
              "versions": {
                "7.0.0": {},
                "8.0.0": {}
              }
            }"#
                            .to_owned(),
                        )],
                    )
                    .await;

                let analysis = session.analyze_document(input);
                let titles = analysis
                    .code_lenses
                    .iter()
                    .map(|lens| lens.title.as_str())
                    .collect::<Vec<_>>();

                crate::test_support::assert_suggestion_without_edits(
                    &output,
                    0,
                    "current",
                    Some("7.0.0"),
                );
                assert_eq!(titles, ["🟢 latest 7.0.0"]);
            }

            #[tokio::test]
            async fn resolves_npm_dist_tag_requirements_against_dist_tags() {
                let session = standard_session();

                let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///package.json".to_owned(), "json".to_owned(), package_file_fixture("resolves-npm-dist-tag-requirements-against-dist-tags.json"), None),
        &[RegistryResponseInput::new("typescript".to_owned(), Npm, r#"{"dist-tags":{"latest":"6.0.3","next":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#.to_owned())],
    ).await;

                assert_eq!(
                    output.suggestions[0].latest.as_deref(),
                    Some("7.0.0-beta.1")
                );
                assert_eq!(output.suggestions[0].status, "updateAvailable");
                assert_eq!(output.edits[0].new_text, "7.0.0-beta.1");
            }

            #[tokio::test]
            async fn missing_npm_dist_tag_requirement_resolves_no_match() {
                let session = standard_session();

                let output = session
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///package.json".to_owned(),
                            "json".to_owned(),
                            package_file_fixture(
                                "missing-npm-dist-tag-requirement-resolves-no-match.json",
                            ),
                            None,
                        ),
                        &[RegistryResponseInput::new(
                            "typescript".to_owned(),
                            Npm,
                            r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{}}}"#
                                .to_owned(),
                        )],
                    )
                    .await;

                assert_eq!(output.suggestions[0].status, "noMatch");
                assert_eq!(output.edits.len(), 0);
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/npm/dist_tags",
                    name,
                )
            }
        }
        mod http {
            use super::*;

            #[test]
            fn npm_http_config_uses_document_npmrc_fetch_timeout() {
                let root = temp_dir().join(format!("versionlens-npmrc-timeout-{}", id()));
                create_dir_all(&root).unwrap();
                write(root.join(".npmrc"), "fetch-timeout=45000\n").unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture("npm-http-config-uses-document-npmrc-fetch-timeout.txt"),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, "https://registry.npmjs.org/left-pad");

                assert_eq!(http.timeout_ms, 45_000);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_registry_http_config_uses_npm_registry_fetch_default_timeout() {
                let input = DocumentInput::new(
                    "file:///workspace/package.json".to_owned(),
                    "json".to_owned(),
                    package_file_fixture(
                        "registry-http-config-uses-npm-registry-fetch-default-timeout.json",
                    ),
                    Some("/workspace".to_owned()),
                );
                let http = http_config(&input, "https://registry.npmjs.org/left-pad");

                assert_eq!(http.timeout_ms, 300_000);
            }

            #[test]
            fn npm_github_http_config_keeps_extension_timeout() {
                let input = DocumentInput::new(
                    "file:///workspace/package.json".to_owned(),
                    "json".to_owned(),
                    package_file_fixture("github-http-config-keeps-extension-timeout.json"),
                    Some("/workspace".to_owned()),
                );
                let http = http_config(&input, "https://api.github.com/repos/owner/repo/tags");

                assert_eq!(http.timeout_ms, 10_000);
            }

            #[test]
            fn npm_registry_http_config_maps_zero_fetch_timeout_to_npm_registry_fetch_fallback() {
                let root = temp_dir().join(format!("versionlens-npmrc-zero-timeout-{}", id()));
                create_dir_all(&root).unwrap();
                write(root.join(".npmrc"), "fetch-timeout=0\n").unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-registry-http-config-maps-zero-fetch-timeout-to-npm-registry-fetch-fallback.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, "https://registry.npmjs.org/left-pad");

                assert_eq!(http.timeout_ms, 30_000);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_http_config_bypasses_proxy_for_noproxy_host() {
                let http = http_config_from_npmrc(
                    "versionlens-npmrc-noproxy",
                    "https-proxy=http://proxy.example.test:8080\nnoproxy=registry.npmjs.org\n",
                    None,
                    "npm-http-config-bypasses-proxy-for-noproxy-host.txt",
                    "https://registry.npmjs.org/left-pad",
                );
                assert_eq!(http.proxy, None);
            }

            #[test]
            fn npm_http_config_uses_generic_https_proxy_from_env_when_npm_proxy_absent() {
                let http = http_config_from_npmrc(
                    "versionlens-npmrc-env-proxy",
                    "registry=https://registry.npmjs.org/\n",
                    Some("HTTPS_PROXY=http://generic-proxy.example.test:8080\n"),
                    "npm-http-config-uses-generic-https-proxy-from-env-when-npm-proxy-absent.txt",
                    "https://registry.npmjs.org/left-pad",
                );

                assert_eq!(
                    http.proxy.as_deref(),
                    Some("http://generic-proxy.example.test:8080")
                );
            }

            #[test]
            fn npm_https_registry_http_config_ignores_generic_http_proxy_without_https_proxy() {
                let http = http_config_from_npmrc(
                    "versionlens-npmrc-env-https-http-proxy",
                    "registry=https://registry.npmjs.org/\n",
                    Some(
                        "HTTP_PROXY=http://http-proxy.example.test:8080\nPROXY=http://plain-proxy.example.test:8080\n",
                    ),
                    "npm-https-registry-http-config-ignores-generic-http-proxy-without-https-proxy.txt",
                    "https://registry.npmjs.org/left-pad",
                );

                assert_eq!(http.proxy, None);
            }

            #[test]
            fn npm_https_registry_http_config_ignores_generic_plain_proxy_without_https_proxy() {
                let http = http_config_from_npmrc(
                    "versionlens-npmrc-env-https-plain-proxy",
                    "registry=https://registry.npmjs.org/\n",
                    Some("PROXY=http://plain-proxy.example.test:8080\n"),
                    "npm-https-registry-http-config-ignores-generic-plain-proxy-without-https-proxy.txt",
                    "https://registry.npmjs.org/left-pad",
                );
                assert_eq!(http.proxy, None);
            }

            #[test]
            fn npm_http_registry_http_config_uses_generic_http_proxy_when_https_proxy_absent() {
                let root = temp_dir().join(format!("versionlens-npmrc-env-http-proxy-{}", id()));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "registry=http://registry.example.test/
",
                )
                .unwrap();
                write(
                    root.join(".env"),
                    "HTTP_PROXY=http://http-proxy.example.test:8080
PROXY=http://plain-proxy.example.test:8080
",
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-http-registry-http-config-uses-generic-http-proxy-when-https-proxy-absent.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, "http://registry.example.test/left-pad");

                assert_eq!(
                    http.proxy.as_deref(),
                    Some("http://http-proxy.example.test:8080")
                );

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_http_config_uses_direct_tls_pem_options() {
                let root = temp_dir().join(format!("versionlens-npmrc-direct-tls-{}", id()));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "ca=direct-ca\ncert=direct-cert\nkey=direct-key\n",
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture("npm-http-config-uses-direct-tls-pem-options.txt"),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, "https://registry.example.test/left-pad");

                assert_eq!(http.ca.as_deref(), Some("direct-ca"));
                assert_eq!(http.cert.as_deref(), Some("direct-cert"));
                assert_eq!(http.key.as_deref(), Some("direct-key"));

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_registry_scoped_client_cert_files_override_direct_cert_and_key() {
                let root = temp_dir().join(format!("versionlens-npmrc-mtls-override-{}", id()));
                create_dir_all(&root).unwrap();
                let cert_file = root.join("client-cert.pem");
                let key_file = root.join("client-key.pem");
                write_client_cert_config(&root, &cert_file, Some(&key_file), true);

                let input = http_fixture_input(
                    &root,
                    "npm-registry-scoped-client-cert-files-override-direct-cert-and-key.txt",
                );
                let context = RegistryContext::from_document(&input);
                let session = standard_session();
                let (matching, other) = scoped_request_configs(&context, &session);

                assert_overridden_client_cert_config(&matching, &other, &cert_file, &key_file);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_partial_registry_scoped_client_cert_files_do_not_override_direct_cert_and_key() {
                let root = temp_dir().join(format!("versionlens-npmrc-mtls-partial-{}", id()));
                create_dir_all(&root).unwrap();
                let cert_file = root.join("client-cert.pem");
                write(
                    root.join(".npmrc"),
                    format!(
                        "cert=direct-cert\nkey=direct-key\n//registry.example.test/:certfile={}\n",
                        cert_file.display()
                    ),
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-partial-registry-scoped-client-cert-files-do-not-override-direct-cert-and-key.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, "https://registry.example.test/left-pad");

                assert_eq!(http.cert.as_deref(), Some("direct-cert"));
                assert_eq!(http.key.as_deref(), Some("direct-key"));
                assert_eq!(http.cert_file, None);
                assert_eq!(http.key_file, None);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_http_config_uses_registry_scoped_client_cert_files() {
                let root = temp_dir().join(format!("versionlens-npmrc-mtls-{}", id()));
                create_dir_all(&root).unwrap();
                let cert_file = root.join("client-cert.pem");
                let key_file = root.join("client-key.pem");
                write(
        root.join(".npmrc"),
        format!(
            "//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
            cert_file.display(),
            key_file.display()
        ),
    )
    .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-http-config-uses-registry-scoped-client-cert-files.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let context = RegistryContext::from_document(&input);
                let session = standard_session();
                let matching = context.http_config_for_request(
                    Npm,
                    "https://registry.example.test/left-pad",
                    session.http_config(Npm),
                );
                let other = context.http_config_for_request(
                    Npm,
                    "https://other.example.test/left-pad",
                    session.http_config(Npm),
                );

                assert_regular_client_cert_config(&matching, &other, &cert_file, &key_file);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npmrc_proxy_false_disables_extension_proxy_for_npm_registry_fetches() {
                let root = temp_dir().join(format!("versionlens-npmrc-proxy-false-{}", id()));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "proxy=false
",
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npmrc-proxy-false-disables-extension-proxy-for-npm-registry-fetches.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let context = RegistryContext::from_document(&input);
                let mut base = standard_session().http_config(Npm);
                base.proxy = Some("http://extension-proxy.example.test:8080".to_owned());
                let http = context.http_config_for_request(
                    Npm,
                    "https://registry.npmjs.org/left-pad",
                    base,
                );

                assert_eq!(http.proxy, None);

                remove_dir_all(root).unwrap();
            }

            fn http_config(input: &DocumentInput, url: &str) -> versionlens_fetch::HttpConfig {
                let context = RegistryContext::from_document(input);
                let session = standard_session();
                context.http_config_for_request(Npm, url, session.http_config(Npm))
            }

            fn package_file_fixture(name: &str) -> String {
                crate::test_support::fixture(
                    "tests/fixtures/session/resolution/tests/npm/http",
                    name,
                )
            }

            fn http_fixture_input(root: &std::path::Path, fixture: &str) -> DocumentInput {
                DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(fixture),
                    Some(root.to_string_lossy().into_owned()),
                )
            }

            fn write_client_cert_config(
                root: &std::path::Path,
                cert_file: &std::path::Path,
                key_file: Option<&std::path::Path>,
                direct: bool,
            ) {
                let direct = if direct {
                    "cert=direct-cert\nkey=direct-key\n"
                } else {
                    ""
                };
                let scoped = match key_file {
                    Some(key) => format!(
                        "//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
                        cert_file.display(),
                        key.display()
                    ),
                    None => format!(
                        "//registry.example.test/:certfile={}\n",
                        cert_file.display()
                    ),
                };
                write(root.join(".npmrc"), format!("{direct}{scoped}")).unwrap();
            }

            fn scoped_request_configs(
                context: &RegistryContext,
                session: &VersionLensSession,
            ) -> (versionlens_fetch::HttpConfig, versionlens_fetch::HttpConfig) {
                (
                    context.http_config_for_request(
                        Npm,
                        "https://registry.example.test/left-pad",
                        session.http_config(Npm),
                    ),
                    context.http_config_for_request(
                        Npm,
                        "https://other.example.test/left-pad",
                        session.http_config(Npm),
                    ),
                )
            }

            fn assert_overridden_client_cert_config(
                matching: &versionlens_fetch::HttpConfig,
                other: &versionlens_fetch::HttpConfig,
                cert_file: &std::path::Path,
                key_file: &std::path::Path,
            ) {
                assert_eq!(matching.cert, None);
                assert_eq!(matching.key, None);
                assert_client_cert_files(matching, cert_file, key_file);
                assert_eq!(other.cert.as_deref(), Some("direct-cert"));
                assert_eq!(other.key.as_deref(), Some("direct-key"));
            }

            fn assert_regular_client_cert_config(
                matching: &versionlens_fetch::HttpConfig,
                other: &versionlens_fetch::HttpConfig,
                cert_file: &std::path::Path,
                key_file: &std::path::Path,
            ) {
                assert_client_cert_files(matching, cert_file, key_file);
                assert_eq!(other.cert_file, None);
                assert_eq!(other.key_file, None);
            }

            fn assert_client_cert_files(
                config: &versionlens_fetch::HttpConfig,
                cert_file: &std::path::Path,
                key_file: &std::path::Path,
            ) {
                assert_eq!(
                    config.cert_file.as_deref(),
                    Some(cert_file.to_string_lossy().as_ref())
                );
                assert_eq!(
                    config.key_file.as_deref(),
                    Some(key_file.to_string_lossy().as_ref())
                );
            }

            fn http_config_from_npmrc(
                root_name: &str,
                npmrc: &str,
                env: Option<&str>,
                fixture: &str,
                url: &str,
            ) -> versionlens_fetch::HttpConfig {
                let root = temp_dir().join(format!("{root_name}-{}", id()));
                create_dir_all(&root).unwrap();
                write(root.join(".npmrc"), npmrc).unwrap();
                if let Some(env) = env {
                    write(root.join(".env"), env).unwrap();
                }
                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(fixture),
                    Some(root.to_string_lossy().into_owned()),
                );
                let http = http_config(&input, url);
                remove_dir_all(root).unwrap();
                http
            }
        }
        mod registry_config {
            use super::*;
            #[test]
            fn npm_registry_urls_use_document_npmrc_scope_registry() {
                let root = temp_dir().join(format!("versionlens-npmrc-{}", id()));
                let package_dir = root.join("package");
                create_dir_all(&package_dir).unwrap();
                write(
                    package_dir.join(".npmrc"),
                    "registry=${DEFAULT_REGISTRY}\n@scope:registry=${SCOPE_REGISTRY}\n",
                )
                .unwrap();
                write(
        package_dir.join(".env"),
        "DEFAULT_REGISTRY=https://registry.example.test/\nSCOPE_REGISTRY=https://scope.example.test/npm\n",
    )
    .unwrap();

                assert_registry_case(
                    &document_input(
                        &package_dir.join("package.json"),
                        &root,
                        "npm-registry-urls-use-document-npmrc-scope-registry.txt",
                    ),
                    &[
                        &["https://scope.example.test/npm/@scope%2fpkg"],
                        &["https://registry.example.test/left-pad"],
                    ],
                );

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn package_npmrc_registry_takes_precedence_over_workspace_npmrc() {
                let root = temp_dir().join(format!("versionlens-npmrc-precedence-{}", id()));
                let package_dir = root.join("package");
                create_dir_all(&package_dir).unwrap();
                write(
                    package_dir.join(".npmrc"),
                    "registry=https://package-registry.example.test/\n",
                )
                .unwrap();
                write(
        root.join(".npmrc"),
        "registry=https://workspace-registry.example.test/\n//package-registry.example.test/:_authToken=workspace-secret\n",
    )
    .unwrap();

                let input = document_input(
                    &package_dir.join("package.json"),
                    &root,
                    "package-npmrc-registry-takes-precedence-over-workspace-npmrc.txt",
                );
                let (session, context, dependencies) = registry_case(&input);

                assert_registry_urls(
                    &session,
                    &context,
                    &dependencies,
                    &[&["https://package-registry.example.test/left-pad"]],
                );
                let headers = context
                    .auth_headers_for_url(Npm, "https://package-registry.example.test/left-pad");
                assert_eq!(headers.len(), 0);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_auth_headers_use_most_specific_document_npmrc_token() {
                let root = temp_dir().join(format!("versionlens-npmrc-auth-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join(".npmrc"),
        "//registry.example.test/:_authToken=${DEFAULT_TOKEN}\n//registry.example.test/npm/:_authToken=${SCOPED_TOKEN}\n",
    )
    .unwrap();
                write(
                    root.join(".env"),
                    "DEFAULT_TOKEN=default-secret\nSCOPED_TOKEN=scoped-secret\n",
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-auth-headers-use-most-specific-document-npmrc-token.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let context = RegistryContext::from_document(&input);

                let default_headers =
                    context.auth_headers_for_url(Npm, "https://registry.example.test/left-pad");
                let scoped_headers =
                    context.auth_headers_for_url(Npm, "https://registry.example.test/npm/left-pad");
                let other_headers =
                    context.auth_headers_for_url(Npm, "https://other.example.test/left-pad");

                assert_eq!(default_headers.len(), 1);
                assert_eq!(default_headers[0].name, "authorization");
                assert_eq!(default_headers[0].value, "Bearer default-secret");
                assert_eq!(scoped_headers.len(), 1);
                assert_eq!(scoped_headers[0].value, "Bearer scoped-secret");
                assert_eq!(other_headers.len(), 0);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_http_config_uses_document_npmrc_proxy_and_strict_ssl() {
                let root = temp_dir().join(format!("versionlens-npmrc-http-{}", id()));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "strict-ssl=false\nhttps-proxy=${NPM_PROXY}\ncafile=/tmp/npm-ca.pem\n",
                )
                .unwrap();
                write(
                    root.join(".env"),
                    "NPM_PROXY=http://proxy.example.test:8080\n",
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-http-config-uses-document-npmrc-proxy-and-strict-ssl.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let context = RegistryContext::from_document(&input);
                let session = standard_session();
                let http = context.http_config_for_request(
                    Npm,
                    "https://registry.npmjs.org/left-pad",
                    session.http_config(Npm),
                );

                assert!(!http.strict_ssl);
                assert_eq!(
                    http.proxy.as_deref(),
                    Some("http://proxy.example.test:8080")
                );
                assert_eq!(http.ca_file.as_deref(), Some("/tmp/npm-ca.pem"));

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_env_file_without_npmrc_does_not_override_registry_or_http_defaults() {
                let root = temp_dir().join(format!("versionlens-npm-env-{}", id()));
                let package_dir = root.join("package");
                create_dir_all(&package_dir).unwrap();
                write(
        package_dir.join(".env"),
        format!(
            "HOME={}\nnpm_config_registry=https://env-registry.example.test/\nnpm_config_strict_ssl=false\nnpm_config_https_proxy=http://env-proxy.example.test:8080\nnpm_config_cafile=/tmp/env-ca.pem\n",
            root.join("home").display()
        ),
    )
    .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", package_dir.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-env-file-without-npmrc-does-not-override-registry-or-http-defaults.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let (session, context, dependencies) = registry_case(&input);
                let http = context.http_config_for_request(
                    Npm,
                    "https://registry.npmjs.org/left-pad",
                    session.http_config(Npm),
                );

                assert_registry_urls(
                    &session,
                    &context,
                    &dependencies,
                    &[&["https://registry.npmjs.org/left-pad"]],
                );
                assert!(http.strict_ssl);
                assert_eq!(http.proxy, None);
                assert_eq!(http.ca_file, None);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_env_file_without_npmrc_does_not_select_userconfig() {
                let root = temp_dir().join(format!("versionlens-npmrc-userconfig-{}", id()));
                let package_dir = root.join("package");
                create_dir_all(&package_dir).unwrap();
                let userconfig_path = root.join("user.npmrc");
                write(
        &userconfig_path,
        "registry=https://user-registry.example.test/\n//user-registry.example.test/:_authToken=user-secret\n",
    )
    .unwrap();
                write(
                    package_dir.join(".env"),
                    format!("NPM_CONFIG_USERCONFIG={}\n", userconfig_path.display()),
                )
                .unwrap();

                let input = document_input(
                    &package_dir.join("package.json"),
                    &root,
                    "npm-env-file-without-npmrc-does-not-select-userconfig.txt",
                );
                let (session, context, dependencies) = registry_case(&input);

                assert_registry_urls(
                    &session,
                    &context,
                    &dependencies,
                    &[&["https://registry.npmjs.org/left-pad"]],
                );

                let headers = context
                    .auth_headers_for_url(Npm, "https://user-registry.example.test/left-pad");
                assert_eq!(headers.len(), 0);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn npm_env_file_without_npmrc_does_not_select_home_userconfig() {
                let root = temp_dir().join(format!("versionlens-npmrc-home-userconfig-{}", id()));
                let package_dir = root.join("package");
                let home_dir = root.join("home");
                create_dir_all(&package_dir).unwrap();
                create_dir_all(&home_dir).unwrap();
                write(
        home_dir.join(".npmrc"),
        "registry=https://home-registry.example.test/\n//home-registry.example.test/:_authToken=home-secret\n",
    )
    .unwrap();
                write(
                    package_dir.join(".env"),
                    format!("NPM_CONFIG_USERCONFIG=\nHOME={}\n", home_dir.display()),
                )
                .unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", package_dir.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture(
                        "npm-env-file-without-npmrc-does-not-select-home-userconfig.txt",
                    ),
                    Some(root.to_string_lossy().into_owned()),
                );
                let (session, context, dependencies) = registry_case(&input);

                assert_eq!(
                    session.registry_urls_with_context(&dependencies[0], &context),
                    vec!["https://registry.npmjs.org/left-pad"]
                );

                let headers = context
                    .auth_headers_for_url(Npm, "https://home-registry.example.test/left-pad");
                assert_eq!(headers.len(), 0);

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn package_json_uses_workspace_yarnrc_registry_and_token() {
                let root = temp_dir().join(format!("versionlens-yarnrc-registry-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmScopes:\n  scope:\n    npmRegistryServer: https://scope.example.test/npm\n    npmAuthToken: ${SCOPE_TOKEN}\n",
    )
    .unwrap();
                write(root.join(".env"), "SCOPE_TOKEN=scoped-secret\n").unwrap();

                let input = document_input(
                    &root.join("package.json"),
                    &root,
                    "package-json-uses-workspace-yarnrc-registry-and-token.txt",
                );
                let (session, context, dependencies) = registry_case(&input);

                assert_registry_urls(
                    &session,
                    &context,
                    &dependencies,
                    &[
                        &["https://scope.example.test/npm/@scope%2fpkg"],
                        &["https://registry.example.test/left-pad"],
                    ],
                );

                let headers = context
                    .auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg");
                assert_eq!(headers.len(), 1);
                assert_eq!(headers[0].name, "authorization");
                assert_eq!(headers[0].value, "Bearer scoped-secret");

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn yarnrc_document_ignores_unsaved_registry_text() {
                let input = DocumentInput::new(
                    "file:///work/.yarnrc.yml".to_owned(),
                    "yaml".to_owned(),
                    package_file_fixture(
                        "yarnrc-document-ignores-unsaved-registry-text.yarnrc.yml",
                    ),
                    None,
                );
                let (session, context, dependencies) = registry_case(&input);

                assert_eq!(dependencies[0].name, "left-pad");
                assert_eq!(
                    session.registry_urls_with_context(&dependencies[0], &context),
                    vec!["https://registry.npmjs.org/left-pad"]
                );
            }

            #[test]
            fn npm_basic_auth_headers_use_document_npmrc_auth() {
                let root = temp_dir().join(format!("versionlens-npmrc-basic-{}", id()));
                create_dir_all(&root).unwrap();
                write(
                    root.join(".npmrc"),
                    "//registry.example.test/:_auth=${BASIC_TOKEN}\n",
                )
                .unwrap();
                write(root.join(".env"), "BASIC_TOKEN=dXNlcjpwYXNz\n").unwrap();

                let input = document_input(
                    &root.join("package.json"),
                    &root,
                    "npm-basic-auth-headers-use-document-npmrc-auth.txt",
                );
                let context = RegistryContext::from_document(&input);
                assert_single_auth_header(
                    &context,
                    Npm,
                    "https://registry.example.test/left-pad",
                    "Basic dXNlcjpwYXNz",
                );

                remove_dir_all(root).unwrap();
            }

            #[test]
            fn yarnrc_auth_ident_headers_use_workspace_yarnrc_auth() {
                let root = temp_dir().join(format!("versionlens-yarnrc-basic-{}", id()));
                create_dir_all(&root).unwrap();
                write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmAuthIdent: ${YARN_IDENT}\n",
    )
    .unwrap();
                write(root.join(".env"), "YARN_IDENT=user:pass\n").unwrap();

                let input = DocumentInput::new(
                    format!("file://{}", root.join("package.json").display()),
                    "json".to_owned(),
                    package_file_fixture("yarnrc-auth-ident-headers-use-workspace-yarnrc-auth.txt"),
                    Some(root.to_string_lossy().into_owned()),
                );
                let context = RegistryContext::from_document(&input);
                assert_single_auth_header(
                    &context,
                    Npm,
                    "https://registry.example.test/left-pad",
                    "Basic dXNlcjpwYXNz",
                );

                remove_dir_all(root).unwrap();
            }

            fn assert_registry_urls(
                session: &VersionLensSession,
                context: &RegistryContext,
                dependencies: &[Dependency],
                expected: &[&[&str]],
            ) {
                for (dependency, urls) in dependencies.iter().zip(expected) {
                    crate::test_support::assert_registry_urls(session, context, dependency, urls);
                }
            }

            fn assert_registry_case(input: &DocumentInput, expected: &[&[&str]]) {
                let (session, context, dependencies) = registry_case(input);
                assert_registry_urls(&session, &context, &dependencies, expected);
            }

            fn document_input(
                document: &std::path::Path,
                workspace_root: &std::path::Path,
                fixture: &str,
            ) -> DocumentInput {
                DocumentInput::new(
                    format!("file://{}", document.display()),
                    "json".to_owned(),
                    package_file_fixture(fixture),
                    Some(workspace_root.to_string_lossy().into_owned()),
                )
            }
        }

        #[tokio::test]
        async fn resolves_update_from_registry_response_body() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("update-from-registry-response-body.json"),
                        None,
                    ),
                    &[crate::test_support::npm_latest_response(
                        "left-pad", "1.1.0",
                    )],
                )
                .await;

            assert_update(&output, "1.1.0");
        }

        #[tokio::test]
        async fn resolves_npm_alias_dependencies_against_target_package() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("alias-dependencies-against-target-package.json"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "typescript".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"6.0.4"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "typescript");
            assert_update(&output, "npm:typescript@6.0.4");
        }

        #[tokio::test]
        async fn resolves_ranged_npm_alias_dependencies_preserving_range_prefix() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "ranged-npm-alias-dependencies-preserving-range-prefix.json",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "@types/react".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"20.0.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "@types/react");
            assert_update(&output, "npm:@types/react@^20.0.0");
        }

        #[tokio::test]
        async fn resolves_unversioned_npm_alias_dependencies_against_target_package() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "unversioned-npm-alias-dependencies-against-target-package.json",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "types-react".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"19.2.7"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "types-react");
            assert_update(&output, "npm:types-react@19.2.7");
        }

        #[tokio::test]
        async fn resolves_package_yaml_npm_alias_dependencies_preserving_alias() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.yaml".to_owned(),
                        "yaml".to_owned(),
                        package_file_fixture(
                            "package-yaml-npm-alias-dependencies-preserving-alias.yaml",
                        ),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "typescript".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"7.0.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "typescript");
            assert_update(&output, "npm:typescript@^7.0.0");
        }

        #[tokio::test]
        async fn resolves_deno_jsr_imports_preserving_specifier_scheme() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///deno.json".to_owned(),
                        "jsonc".to_owned(),
                        package_file_fixture("deno-jsr-imports-preserving-specifier-scheme.json"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "@std/assert".to_owned(),
                        Deno,
                        r#"{"latest":"1.1.0","versions":{"1.0.0":{},"1.1.0":{}}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "@std/assert");
            assert_update(&output, "jsr:@std/assert@1.1.0");
        }

        #[tokio::test]
        async fn resolves_deno_npm_imports_preserving_specifier_scheme() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///deno.json".to_owned(),
                        "jsonc".to_owned(),
                        package_file_fixture("deno-npm-imports-preserving-specifier-scheme.json"),
                        None,
                    ),
                    &[RegistryResponseInput::new(
                        "chalk".to_owned(),
                        Npm,
                        r#"{"dist-tags":{"latest":"5.4.0"}}"#.to_owned(),
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].dependency.name, "chalk");
            assert_update(&output, "npm:chalk@5.4.0");
        }

        #[tokio::test]
        async fn resolves_invalid_empty_ranges_as_invalid_range_with_latest_update() {
            let session = session_without_vulnerabilities();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///package.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture(
                            "invalid-empty-ranges-as-invalid-range-with-latest-update.json",
                        ),
                        None,
                    ),
                    &[crate::test_support::npm_latest_response(
                        "left-pad", "5.0.0",
                    )],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "invalidRange");
            assert_eq!(output.edits[0].new_text, "5.0.0");
        }

        struct NpmResolutionCase<'a> {
            fixture: &'a str,
            response_body: &'a str,
            expected_status: &'a str,
            expected_latest: Option<&'a str>,
            expected_titles: &'a [&'a str],
            expected_arguments: &'a [(&'a str, &'a str)],
        }

        async fn assert_npm_resolution_choices(case: NpmResolutionCase<'_>) {
            let session = session_without_vulnerabilities();
            let input = DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture(case.fixture),
                None,
            );
            let output = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "left-pad".to_owned(),
                        Npm,
                        case.response_body.to_owned(),
                    )],
                )
                .await;
            let analysis = session.analyze_document(input);

            assert_eq!(output.suggestions[0].status, case.expected_status);
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                case.expected_latest
            );
            assert_eq!(output.edits.len(), 0);
            assert_eq!(lens_titles(&analysis), case.expected_titles);
            assert_eq!(
                crate::test_support::code_lens_arguments(&analysis),
                case.expected_arguments
                    .iter()
                    .map(|(command, version)| vec![(*command).to_owned(), (*version).to_owned()])
                    .collect::<Vec<_>>()
            );
        }

        #[tokio::test]
        async fn missing_fixed_npm_registry_version_resolves_no_match_with_update_choices() {
            assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "missing-fixed-npm-registry-version-resolves-no-match-with-update-choices.json",
        response_body: r#"{
              "dist-tags": { "latest": "1.0.0" },
              "versions": { "0.5.1": {}, "0.6.0": {}, "1.0.0": {} }
            }"#,
        expected_status: "noMatch",
        expected_latest: None,
        expected_titles: &[
            "⚪ no match",
            "↑  patch 0.5.1",
            "↑  minor 0.6.0",
            "↑  latest 1.0.0",
        ],
        expected_arguments: &[
            ("updatePatch", "0.5.1"),
            ("updateMinor", "0.6.0"),
            ("update", "1.0.0"),
        ],
    })
    .await;
        }

        #[tokio::test]
        async fn fixed_npm_prerelease_resolves_fixed_with_prerelease_update_choice() {
            assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "fixed-npm-prerelease-resolves-fixed-with-prerelease-update-choice.json",
        response_body: r#"{"versions":{"1.0.0-beta.1":{},"1.0.0-beta.2":{},"1.0.0-beta.3":{}}}"#,
        expected_status: "fixed",
        expected_latest: Some("1.0.0-beta.1"),
        expected_titles: &["🟡 fixed 1.0.0-beta.1", "↑  beta 1.0.0-beta.3"],
        expected_arguments: &[("update", "1.0.0-beta.3")],
    })
    .await;
        }

        #[tokio::test]
        async fn fixed_npm_release_resolves_fixed_with_release_update_choices() {
            assert_npm_resolution_choices(NpmResolutionCase {
        fixture: "fixed-npm-release-resolves-fixed-with-release-update-choices.json",
        response_body: r#"{"versions":{"1.1.0":{},"1.1.1":{},"1.1.2":{},"1.2.0":{},"1.2.2":{},"2.0.0":{},"2.2.2":{}}}"#,
        expected_status: "fixed",
        expected_latest: Some("1.1.1"),
        expected_titles: &[
            "🟡 fixed 1.1.1",
            "↑  patch 1.1.2",
            "↑  minor 1.2.2",
            "↑  latest 2.2.2",
        ],
        expected_arguments: &[
            ("updatePatch", "1.1.2"),
            ("updateMinor", "1.2.2"),
            ("update", "2.2.2"),
        ],
    }).await;
        }

        #[test]
        fn deno_npm_imports_use_document_npmrc_registry() {
            let root = temp_dir().join(format!("versionlens-deno-npmrc-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join(".npmrc"),
                "registry=https://registry.example.test/\n",
            )
            .unwrap();

            let input = DocumentInput::new(
                format!("file://{}", root.join("deno.json").display()),
                "jsonc".to_owned(),
                package_file_fixture("deno-npm-imports-use-document-npmrc-registry.txt"),
                Some(root.to_string_lossy().into_owned()),
            );
            let (session, context, dependencies) = registry_case(&input);

            assert_eq!(dependencies[0].ecosystem, Npm);
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://registry.example.test/chalk"]
            );

            remove_dir_all(root).unwrap();
        }

        #[test]
        fn pnpm_yaml_dependencies_use_document_npmrc_registry() {
            let root = temp_dir().join(format!("versionlens-pnpm-npmrc-{}", id()));
            create_dir_all(&root).unwrap();
            write(
        root.join(".npmrc"),
        "@scope:registry=https://scope.example.test/npm\nregistry=https://registry.example.test/\n",
    )
    .unwrap();

            let input = DocumentInput::new(
                format!("file://{}", root.join("pnpm-workspace.yaml").display()),
                "yaml".to_owned(),
                package_file_fixture("pnpm-yaml-dependencies-use-document-npmrc-registry.txt"),
                Some(root.to_string_lossy().into_owned()),
            );
            let (_session, _context, _dependencies) = registry_case_with_expected_urls(
                &input,
                &[
                    &["https://scope.example.test/npm/@scope%2fpkg"],
                    &["https://registry.example.test/left-pad"],
                ],
            );

            remove_dir_all(root).unwrap();
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/tests/npm", name)
        }
        use super::assert_update;
    }
    mod project {
        use std::env::temp_dir;
        use std::fs::create_dir_all;
        use std::fs::remove_dir_all;
        use std::fs::write;
        use std::process::id;

        use super::{DocumentInput, standard_session};
        use versionlens_model::Ecosystem::Composer;

        #[tokio::test]
        async fn cargo_project_version_is_not_resolved_without_registry_response() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///Cargo.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture("version-without-registry-response.toml"),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions.len(), 0);
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn resolves_jsr_project_version_without_registry_response() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///jsr.json".to_owned(),
                        "json".to_owned(),
                        package_file_fixture("jsr-project-version-without-registry-response.json"),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(output.suggestions[0].dependency.name, "@scope/pkg");
            assert_eq!(output.edits[0].new_text, "1.2.4");
        }

        #[tokio::test]
        async fn resolves_gleam_project_version_without_registry_response() {
            let session = standard_session();

            let output = session
                .resolve_document_with_responses(
                    DocumentInput::new(
                        "file:///gleam.toml".to_owned(),
                        "toml".to_owned(),
                        package_file_fixture(
                            "gleam-project-version-without-registry-response.toml",
                        ),
                        None,
                    ),
                    &[],
                )
                .await;

            assert_eq!(output.suggestions[0].status, "updateAvailable");
            assert_eq!(output.suggestions[0].dependency.name, "my_package");
            assert_eq!(output.suggestions[0].dependency.group, "version");
            assert_eq!(output.edits[0].new_text, "1.2.4");
        }

        #[test]
        fn analyzes_project_version_code_lens_without_registry_response() {
            let session = standard_session();

            let output = session.analyze_document(DocumentInput::new(
                "file:///pubspec.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture(
                    "analyzes-project-version-code-lens-without-registry-response.yaml",
                ),
                None,
            ));

            let titles = output
                .code_lenses
                .iter()
                .map(|lens| lens.title.as_str())
                .collect::<Vec<_>>();
            let commands = crate::test_support::code_lens_commands(&output);

            assert_eq!(titles.len(), 3);
            assert!(titles.iter().any(|title| title.contains("major 2.0.0")));
            assert!(titles.iter().any(|title| title.contains("minor 1.3.0")));
            assert!(titles.iter().any(|title| title.contains("patch 1.2.4")));
            assert!(titles.iter().all(|title| !title.contains(" available")));
            assert_eq!(commands, ["updateMajor", "updateMinor", "updatePatch"]);
        }

        #[test]
        fn composer_repositories_override_registry_urls() {
            let input = DocumentInput::new(
                "file:///repo/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("composer-repositories-override-registry-urls.json"),
                None,
            );
            let (session, context, dependencies) =
                crate::session::resolution::tests::registry_case(&input);

            assert_eq!(dependencies[0].name, "phpunit/phpunit");
            assert_eq!(
                session.registry_urls_with_context(&dependencies[0], &context),
                vec!["https://composer.example.test/phpunit/phpunit.json"]
            );
        }

        #[test]
        fn composer_auth_json_supplies_request_scoped_auth_headers() {
            let root = temp_dir().join(format!("versionlens-composer-auth-{}", id()));
            create_dir_all(&root).unwrap();
            write(
                root.join("auth.json"),
                r#"{
  "http-basic": {
    "composer.example.test": {"username":"user","password":"pass"},
    "composer.example.test/private": {"username":"scoped","password":"secret"}
  },
  "bearer": {
    "bearer.example.test": "token"
  }
}"#,
            )
            .unwrap();

            let input = DocumentInput::new(
                format!("file://{}", root.join("composer.json").display()),
                "json".to_owned(),
                package_file_fixture("composer-auth-json-supplies-request-scoped-auth-headers.txt"),
                Some(root.to_string_lossy().into_owned()),
            );
            let context = crate::registry::RegistryContext::from_document(&input);
            let default_headers = context.auth_headers_for_url(
                Composer,
                "https://composer.example.test/p2/phpunit/phpunit.json",
            );
            let scoped_headers = context.auth_headers_for_url(
                Composer,
                "https://composer.example.test/private/p2/phpunit/phpunit.json",
            );
            let bearer_headers = context.auth_headers_for_url(
                Composer,
                "https://bearer.example.test/p2/phpunit/phpunit.json",
            );
            let other_headers = context.auth_headers_for_url(
                Composer,
                "https://other.example.test/p2/phpunit/phpunit.json",
            );

            assert_eq!(default_headers.len(), 1);
            assert_eq!(default_headers[0].value, "Basic dXNlcjpwYXNz");
            assert_eq!(scoped_headers.len(), 1);
            assert_eq!(scoped_headers[0].value, "Basic c2NvcGVkOnNlY3JldA==");
            assert_eq!(bearer_headers.len(), 1);
            assert_eq!(bearer_headers[0].value, "Bearer token");
            assert_eq!(other_headers.len(), 0);

            remove_dir_all(root).unwrap();
        }

        fn package_file_fixture(name: &str) -> String {
            crate::test_support::fixture("tests/fixtures/session/resolution/project", name)
        }
    }
    mod runtime {
        use super::*;

        mod constraints {
            use super::super::*;

            #[tokio::test]
            async fn ci_runtime_channels_and_numeric_constraints_keep_the_declared_selector() {
                for (action, field, context, name, ecosystem, requirement, latest, body, status) in [
                    (
                        "actions/setup-go",
                        "go-version",
                        "",
                        "go",
                        Go,
                        "stable",
                        "1.25.1",
                        r#"[{"version":"go1.25.1","stable":true},{"version":"go1.24.7","stable":true}]"#,
                        "current",
                    ),
                    (
                        "denoland/setup-deno",
                        "deno-version",
                        "",
                        "deno",
                        Deno,
                        "lts",
                        "2.1.4",
                        "v2.1.4",
                        "current",
                    ),
                    (
                        "actions/setup-python",
                        "python-version",
                        "",
                        "python",
                        Python,
                        "3.12",
                        "3.12.2",
                        r#"[{"version":"3.12.2","stable":true},{"version":"3.13.0","stable":true}]"#,
                        "satisfiesLatest",
                    ),
                    (
                        "actions/setup-java",
                        "java-version",
                        "distribution: temurin, ",
                        "java",
                        Maven,
                        "21",
                        "21.0.12.1",
                        r#"{"releases":["jdk-21.0.12+8","jdk-21.0.12.1+1"]}"#,
                        "satisfiesLatest",
                    ),
                    (
                        "actions/setup-dotnet",
                        "dotnet-version",
                        "",
                        "dotnet",
                        Dotnet,
                        "8.0.x",
                        "8.0.405",
                        r#"{"releases":[{"sdk":{"version":"8.0.405"}}]}"#,
                        "satisfiesLatest",
                    ),
                    (
                        "ruby/setup-ruby",
                        "ruby-version",
                        "",
                        "ruby",
                        Ruby,
                        "3.3",
                        "3.3.2",
                        r#"{"ruby":["3.3.2","3.4.0"]}"#,
                        "satisfiesLatest",
                    ),
                ] {
                    let text = format!(
                        "steps: [{{uses: {action}@v1, with: {{{context}{field}: '{requirement}'}}}}]\n"
                    );
                    let output = session_without_vulnerabilities()
                        .resolve_document_with_responses(
                            DocumentInput::new(
                                "file:///work/.github/workflows/runtime.yml",
                                "yaml",
                                text,
                                None,
                            ),
                            &[
                                RegistryResponseInput::new(action, GitHub, r#"[{"name":"v1"}]"#),
                                RegistryResponseInput::new(name, ecosystem, body),
                            ],
                        )
                        .await;
                    let runtime = output
                        .suggestions
                        .iter()
                        .find(|suggestion| suggestion.dependency.name == name)
                        .unwrap();
                    assert_eq!(runtime.status, status, "{name}: {runtime:?}");
                    assert_eq!(runtime.latest.as_deref(), Some(latest), "{name}");
                    assert!(output.edits.is_empty(), "{name}: {output:?}");
                }
            }

            #[tokio::test]
            async fn ci_runtime_variants_without_a_release_catalog_are_not_supported() {
                for (action, input, context, name, _ecosystem, requirement) in [
                    (
                        "actions/setup-python",
                        "python-version",
                        "",
                        "python",
                        Python,
                        "pypy3.10",
                    ),
                    (
                        "actions/setup-java",
                        "java-version",
                        "distribution: zulu, ",
                        "java",
                        Maven,
                        "17",
                    ),
                    (
                        "actions/setup-dotnet",
                        "dotnet-version",
                        "dotnet-quality: preview, ",
                        "dotnet",
                        Dotnet,
                        "8.0.x",
                    ),
                    (
                        "ruby/setup-ruby",
                        "ruby-version",
                        "",
                        "ruby",
                        Ruby,
                        "jruby-9.4",
                    ),
                ] {
                    let text = format!(
                        "steps: [{{uses: {action}@v1, with: {{{context}{input}: '{requirement}'}}}}]\n"
                    );
                    let output = session_without_vulnerabilities()
                        .resolve_document_with_responses(
                            DocumentInput::new(
                                "file:///work/.github/workflows/runtime.yml",
                                "yaml",
                                text,
                                None,
                            ),
                            &[RegistryResponseInput::new(
                                action,
                                GitHub,
                                r#"[{"name":"v1"}]"#,
                            )],
                        )
                        .await;
                    let runtime = output
                        .suggestions
                        .iter()
                        .find(|suggestion| suggestion.dependency.name == name)
                        .unwrap();
                    assert_eq!(runtime.status, "notSupported", "{name}: {runtime:?}");
                }
            }
        }
        mod package_managers {
            use super::*;

            async fn resolve_package_manager(
                name: &str,
                requirement: &str,
                body: impl Into<String>,
            ) -> ResolveDocumentOutput {
                session_without_vulnerabilities()
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///work/package.json",
                            "json",
                            format!(r#"{{"packageManager":"{name}@{requirement}"}}"#),
                            None,
                        ),
                        &[RegistryResponseInput::new(name, Npm, body.into())],
                    )
                    .await
            }

            fn sha224_registry_metadata(current_url: &str, selected_url: &str) -> String {
                format!(
                    r#"{{"dist-tags":{{"latest":"2.0.0"}},"versions":{{"1.0.0":{{"dist":{{"tarball":"{current_url}"}}}},"2.0.0":{{"dist":{{"tarball":"{selected_url}"}}}}}}}}"#
                )
            }

            const CURRENT_ARTIFACT_SHA224: &str =
                "e8c69735a0b5c2320a34aa2234434719d06d2b71a18d74904ebe874c";

            #[tokio::test]
            async fn package_manager_pins_use_published_package_versions() {
                let output = resolve_package_manager(
                    "pnpm",
                    "9.1.2",
                    r#"{"dist-tags":{"latest":"10.34.4"},"versions":{"9.1.2":{},"10.34.4":{}}}"#,
                )
                .await;
                assert!(!output.edits.is_empty(), "{output:?}");
                assert_eq!(output.edits[0].new_text, "10.34.4");
            }

            #[tokio::test]
            async fn yarn_pins_use_the_release_catalog_for_their_version_family() {
                for (requirement, body, expected) in [
                    (
                        "1.22.19",
                        r#"{"dist-tags":{"latest":"1.22.22"},"versions":{"1.22.19":{},"1.22.22":{}}}"#,
                        "1.22.22",
                    ),
                    (
                        "4.1.0",
                        r#"{"aliases":{"latest":"4.2.0","stable":"4.1.1"},"tags":["4.2.0","4.1.1","4.1.0"]}"#,
                        "4.2.0",
                    ),
                ] {
                    let output = resolve_package_manager("yarn", requirement, body).await;
                    assert_eq!(output.edits.len(), 1, "{output:?}");
                    assert_eq!(output.edits[0].new_text, expected);
                }
            }

            #[tokio::test]
            async fn yarn_aliases_do_not_replace_a_newer_pinned_release() {
                let output = resolve_package_manager(
                    "yarn",
                    "4.3.0",
                    r#"{"aliases":{"latest":"4.2.0"},"tags":["4.3.0","4.2.0"]}"#,
                )
                .await;
                assert!(output.edits.is_empty(), "{output:?}");
                assert_eq!(output.suggestions[0].status, "current");
                assert_eq!(output.suggestions[0].latest.as_deref(), Some("4.3.0"));
            }

            #[tokio::test]
            async fn yarn_classic_sha1_pins_are_verified_and_preserved() {
                let current = "00112233445566778899aabbccddeeff00112233";
                let selected = "ffeeddccbbaa99887766554433221100ffeeddcc";
                let body = format!(
                    r#"{{"dist-tags":{{"latest":"1.22.22"}},"versions":{{"1.22.19":{{"dist":{{"shasum":"{current}"}}}},"1.22.22":{{"dist":{{"shasum":"{selected}"}}}}}}}}"#
                );
                let output =
                    resolve_package_manager("yarn", &format!("1.22.19+sha1.{current}"), body).await;
                assert_eq!(output.edits.len(), 1, "{output:?}");
                assert_eq!(output.edits[0].new_text, format!("1.22.22+sha1.{selected}"));
            }

            #[tokio::test]
            async fn sha224_pins_hash_the_current_and_selected_registry_artifacts() {
                let (base_url, server) =
                    github::github_api_server(vec![(200, "current"), (200, "selected")]);
                let current_url = format!("{base_url}current.tgz");
                let selected_url = format!("{base_url}selected.tgz");
                let body = sha224_registry_metadata(&current_url, &selected_url);
                let output = resolve_package_manager(
                    "pnpm",
                    &format!("1.0.0+sha224.{CURRENT_ARTIFACT_SHA224}"),
                    body,
                )
                .await;
                assert_eq!(output.edits.len(), 1, "{output:?}");
                assert_eq!(
                    output.edits[0].new_text,
                    "2.0.0+sha224.1bb4223297e1f5140bf49830b53cd9e844a5142a3dfa5f8a2b024aa9"
                );
                assert_eq!(
                    server.join().unwrap(),
                    ["/repos/current.tgz", "/repos/selected.tgz"]
                );
            }

            #[tokio::test]
            async fn sha224_rejects_a_bad_current_pin_before_fetching_the_target() {
                let (base_url, server) = github::github_api_server(vec![(200, "current")]);
                let body = sha224_registry_metadata(
                    &format!("{base_url}current.tgz"),
                    &format!("{base_url}selected.tgz"),
                );
                let output = resolve_package_manager(
                    "pnpm",
                    &format!("1.0.0+sha224.{}", "00".repeat(28)),
                    body,
                )
                .await;
                assert!(output.edits.is_empty(), "{output:?}");
                assert_eq!(output.suggestions[0].status, "error");
                assert!(
                    output.suggestions[0]
                        .latest
                        .as_deref()
                        .unwrap()
                        .contains("integrity pin does not match")
                );
                assert_eq!(server.join().unwrap(), ["/repos/current.tgz"]);
            }

            #[tokio::test]
            async fn sha224_artifact_fetch_failures_are_explicit() {
                for status in [401, 404] {
                    let (base_url, server) = github::github_api_server(vec![(status, "missing")]);
                    let current_url = format!("{base_url}current.tgz");
                    let body =
                        sha224_registry_metadata(&current_url, &format!("{base_url}selected.tgz"));
                    let output = resolve_package_manager(
                        "pnpm",
                        &format!("1.0.0+sha224.{CURRENT_ARTIFACT_SHA224}"),
                        body,
                    )
                    .await;
                    assert!(output.edits.is_empty(), "{output:?}");
                    assert_eq!(output.suggestions[0].status, "error");
                    assert!(output.suggestions[0].latest.is_some());
                    assert_eq!(server.join().unwrap(), ["/repos/current.tgz"]);
                    if status == 401 {
                        assert_eq!(output.authorization_required_count, 1);
                        assert_eq!(
                            output.authorization_required_requests[0].request_url,
                            current_url
                        );
                    } else {
                        assert_eq!(output.authorization_required_count, 0);
                    }
                }
            }

            #[tokio::test]
            async fn package_manager_integrity_pins_are_verified_and_preserved() {
                let current_digest = "00".repeat(64);
                let target_digest = "ff".repeat(64);
                let body = serde_json::json!({
                    "dist-tags": {"latest": "10.34.4"},
                    "versions": {
                        "9.1.2": {"dist": {"integrity": format!("sha512-{}==", "A".repeat(86))}},
                        "10.34.4": {"dist": {"integrity": format!("sha512-{}w==", "/".repeat(85))}}
                    }
                })
                .to_string();
                let session = session_without_vulnerabilities();
                let input = DocumentInput::new(
                    "file:///work/package.json",
                    "json",
                    format!(r#"{{"packageManager":"pnpm@9.1.2+sha512.{current_digest}"}}"#),
                    None,
                );
                let responses = [RegistryResponseInput::new("pnpm", Npm, &body)];
                let output = session
                    .resolve_document_with_responses(input, &responses)
                    .await;
                assert_eq!(output.edits.len(), 1, "{output:?}");
                assert_eq!(
                    output.edits[0].new_text,
                    format!("10.34.4+sha512.{target_digest}")
                );
                let current_input = DocumentInput::new(
                    "file:///work/package.json",
                    "json",
                    format!(r#"{{"packageManager":"pnpm@10.34.4+sha512.{target_digest}"}}"#),
                    None,
                );
                let current = session
                    .resolve_document_with_responses(current_input.clone(), &responses)
                    .await;
                assert_eq!(current.edits.len(), 0);
                assert_eq!(current.suggestions[0].status, "current");
                assert!(
                    session
                        .analyze_document(current_input)
                        .code_lenses
                        .iter()
                        .all(|lens| lens.action_kind().is_empty())
                );
                for pin in [format!("sha512.{target_digest}"), "sha224.1234".to_owned()] {
                    let input = DocumentInput::new(
                        "file:///work/package.json",
                        "json",
                        format!(r#"{{"packageManager":"pnpm@9.1.2+{pin}"}}"#),
                        None,
                    );
                    let output = session
                        .resolve_document_with_responses(input, &responses)
                        .await;
                    assert_eq!(output.edits.len(), 0);
                    assert_eq!(output.suggestions[0].status, "error");
                }
            }

            #[tokio::test]
            async fn package_manager_release_channels_follow_published_metadata() {
                let body = r#"{"dist-tags":{"latest":"10.0.0","next":"11.0.0-beta.1"},"versions":{"9.0.0":{},"10.0.0":{},"11.0.0":{},"11.0.0-beta.1":{}}}"#;
                for (requirement, latest, status) in [
                    ("9.0.0", "10.0.0", "updateAvailable"),
                    ("11.0.0", "11.0.0", "current"),
                    ("latest", "10.0.0", "current"),
                    ("next", "11.0.0-beta.1", "current"),
                ] {
                    let output = session_without_vulnerabilities()
                        .resolve_document_with_responses(
                            DocumentInput::new(
                                "file:///work/package.json",
                                "json",
                                format!(r#"{{"packageManager":"pnpm@{requirement}"}}"#),
                                None,
                            ),
                            &[RegistryResponseInput::new("pnpm", Npm, body)],
                        )
                        .await;
                    assert_eq!(
                        output.suggestions[0].latest.as_deref(),
                        Some(latest),
                        "{output:?}"
                    );
                    assert_eq!(output.suggestions[0].status, status);
                    if status == "current" {
                        assert_eq!(output.edits.len(), 0);
                    }
                }
            }

            #[tokio::test]
            async fn package_manager_channels_require_a_published_target() {
                for body in [
                    r#"{"versions":{"10.0.0":{}}}"#,
                    r#"{"dist-tags":{"latest":"11.0.0"},"versions":{"10.0.0":{}}}"#,
                    r#"{"dist-tags":{"latest":"invalid"},"versions":{"invalid":{}}}"#,
                ] {
                    let input = DocumentInput::new(
                        "file:///work/package.json",
                        "json",
                        r#"{"packageManager":"pnpm@latest"}"#,
                        None,
                    );
                    let session = session_without_vulnerabilities();
                    let output = session
                        .resolve_document_with_responses(
                            input.clone(),
                            &[RegistryResponseInput::new("pnpm", Npm, body)],
                        )
                        .await;
                    assert_eq!(output.suggestions[0].status, "error");
                    assert_eq!(output.edits.len(), 0);
                    assert!(session.document_is_fresh(&input));
                }
            }
        }
        mod requests {
            use super::super::*;

            #[tokio::test]
            async fn runtime_release_requests_use_configured_source_providers() {
                for (uri, language, text, ecosystem, body, path, expected) in [
                    (
                        "file:///work/package.json",
                        "json",
                        r#"{"packageManager":"pnpm@9.0.0"}"#,
                        Npm,
                        r#"{"dist-tags":{"latest":"10.0.0"},"versions":{"9.0.0":{},"10.0.0":{}}}"#,
                        "/repos/pnpm",
                        "10.0.0",
                    ),
                    (
                        "file:///work/.github/workflows/ci.yml",
                        "yaml",
                        "steps:\n  - uses: dtolnay/rust-toolchain@1.80.0\n",
                        GitHub,
                        r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
                        "/repos/rust-lang/rust/tags",
                        "1.90.0",
                    ),
                ] {
                    let (url, server) = github::github_api_server(vec![(200, body)]);
                    let session = crate::test_support::session_with_provider_settings(
                        ProviderSettings {
                            registry_urls: vec![RegistryUrlConfig { ecosystem, url }],
                            ..Default::default()
                        },
                        false,
                    );
                    let output = session
                        .resolve_document(DocumentInput::new(uri, language, text, None))
                        .await;
                    assert_eq!(server.join().unwrap(), [path]);
                    assert_eq!(output.edits.len(), 1, "{output:?}");
                    assert_eq!(output.edits[0].new_text, expected);
                }
            }
        }

        struct CiRuntimeCase<'a> {
            action: &'a str,
            field: &'a str,
            context: &'a str,
            name: &'a str,
            ecosystem: Ecosystem,
            requirement: &'a str,
            body: &'a str,
        }

        async fn resolve_ci_runtime(
            case: CiRuntimeCase<'_>,
        ) -> (VersionLensSession, DocumentInput, ResolveDocumentOutput) {
            let CiRuntimeCase {
                action,
                field,
                context,
                name,
                ecosystem,
                requirement,
                body,
            } = case;
            let text = format!(
                "steps: [{{uses: {action}@v1, with: {{{context}{field}: '{requirement}'}}}}]\n"
            );
            let input = DocumentInput::new(
                "file:///work/.github/workflows/runtime.yml",
                "yaml",
                text,
                None,
            );
            let responses = [
                RegistryResponseInput::new(action, GitHub, r#"[{"name":"v1"}]"#),
                RegistryResponseInput::new(name, ecosystem, body),
            ];
            let session = session_without_vulnerabilities();
            let output = session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            (session, input, output)
        }

        fn runtime_suggestion<'a>(
            output: &'a ResolveDocumentOutput,
            name: &str,
        ) -> &'a crate::SuggestionView {
            output
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == name)
                .unwrap_or_else(|| panic!("missing resolved {name}: {output:?}"))
        }

        #[tokio::test]
        async fn escaped_runtime_versions_use_the_full_encoded_edit_range() {
            for (source, name, ecosystem, body, encoded, replacement) in [
                (
                    r#"steps: [{uses: "dtolnay/rust-toolchain\u00401.\u0038\u0030.0"}]"#,
                    "rust",
                    Cargo,
                    r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
                    r"1.\u0038\u0030.0",
                    "1.90.0",
                ),
                (
                    r#"steps: [{uses: actions/setup-node@v4, with: {node-version: "\u0032\u0030.0.0"}}]"#,
                    "node",
                    Npm,
                    r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
                    r"\u0032\u0030.0.0",
                    "22.0.0",
                ),
            ] {
                let output = session_without_vulnerabilities()
                    .resolve_document_with_responses(
                        DocumentInput::new(
                            "file:///work/.github/workflows/ci.yml",
                            "yaml",
                            source,
                            None,
                        ),
                        &[
                            RegistryResponseInput::new(name, ecosystem, body),
                            RegistryResponseInput::new(
                                "actions/setup-node",
                                GitHub,
                                r#"[{"name":"v4"}]"#,
                            ),
                        ],
                    )
                    .await;
                assert_eq!(output.edits.len(), 1, "{output:?}");
                let edit = &output.edits[0];
                assert_eq!(
                    &source[edit.range.start.character as usize..edit.range.end.character as usize],
                    encoded
                );
                assert_eq!(edit.new_text, replacement);
            }
        }

        #[tokio::test]
        async fn ci_tool_versions_are_checked_and_edited_at_their_literal_ranges() {
            let session = session_without_vulnerabilities();
            let text = "jobs: {build: {steps: [{name: '🦀', uses: actions/setup-node@v4, with: {node-version: '20.0.0'}}, {uses: oven-sh/setup-bun@v2, with: {bun-version: \"1.0.0\"}}, {uses: dtolnay/rust-toolchain@1.80.0}]}} # keep\n";
            let input =
                DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", text, None);
            let responses = [
                RegistryResponseInput::new("actions/setup-node", GitHub, r#"[{"name":"v4"}]"#),
                RegistryResponseInput::new("oven-sh/setup-bun", GitHub, r#"[{"name":"v2"}]"#),
                RegistryResponseInput::new(
                    "node",
                    Npm,
                    r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
                ),
                RegistryResponseInput::new(
                    "bun",
                    Npm,
                    r#"[{"name":"bun-v1.0.0"},{"name":"bun-v1.2.0"}]"#,
                ),
                RegistryResponseInput::new(
                    "rust",
                    Cargo,
                    r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
                ),
            ];
            let first = session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            assert_eq!(
                first
                    .edits
                    .iter()
                    .map(|edit| edit.new_text.as_str())
                    .collect::<Vec<_>>(),
                ["22.0.0", "1.2.0", "1.90.0"]
            );
            let start = text.find("20.0.0").unwrap();
            assert!(!first.edits.is_empty(), "{first:?}");
            assert_eq!(
                first.edits[0].range.start.character as usize,
                text[..start].encode_utf16().count()
            );
            assert_eq!(
                first.edits[0].range.end.character - first.edits[0].range.start.character,
                6
            );
            assert!(session.document_is_fresh(&input));
            let mut release = input;
            release.uri = "file:///work/.github/workflows/release.yml".to_owned();
            let second = session
                .resolve_document_with_responses(release, &responses)
                .await;
            assert_eq!(first.edits, second.edits);
        }

        #[tokio::test]
        async fn current_bun_pin_has_one_status_lens_and_no_update() {
            let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
                action: "oven-sh/setup-bun",
                field: "bun-version",
                context: "",
                name: "bun",
                ecosystem: Npm,
                requirement: "1.4.2",
                body: r#"[{"name":"bun-v1.4.2"}]"#,
            })
            .await;
            let bun = runtime_suggestion(&output, "bun");
            assert_eq!(bun.status, "current");
            assert_eq!(output.edits.len(), 0);

            let lenses = session.analyze_document(input).code_lenses;
            assert_eq!(
                lenses
                    .iter()
                    .filter(|lens| lens.title.ends_with("latest 1.4.2"))
                    .count(),
                1
            );
            assert!(
                lenses
                    .iter()
                    .filter(|lens| lens.title.ends_with("latest 1.4.2"))
                    .all(|lens| lens.action_kind().is_empty()),
                "{lenses:?}"
            );
        }

        #[tokio::test]
        async fn compatible_bun_selector_is_not_suppressed_as_an_exact_pin() {
            let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
                action: "oven-sh/setup-bun",
                field: "bun-version",
                context: "",
                name: "bun",
                ecosystem: Npm,
                requirement: "1.4",
                body: r#"[{"name":"bun-v1.4.2"}]"#,
            })
            .await;
            let bun = runtime_suggestion(&output, "bun");
            assert_eq!(bun.status, "updateAvailable");
            assert_eq!(bun.latest.as_deref(), Some("1.4.2"));
            assert_eq!(output.edits.len(), 1, "{output:?}");
            assert_eq!(output.edits[0].new_text, "1.4.2");
            let lenses = session.analyze_document(input).code_lenses;
            let runtime_lenses = lenses
                .iter()
                .filter(|lens| lens.title.ends_with("latest 1.4.2"))
                .collect::<Vec<_>>();
            assert_eq!(runtime_lenses.len(), 1, "{lenses:?}");
            assert!(
                runtime_lenses
                    .iter()
                    .all(|lens| !lens.action_kind().is_empty())
            );
        }

        #[tokio::test]
        async fn exact_node_and_java_pins_have_current_commandless_runtime_lenses() {
            for (action, field, context, name, ecosystem, version, body) in [
                (
                    "actions/setup-node",
                    "node-version",
                    "",
                    "node",
                    Npm,
                    "22.0.0",
                    r#"[{"version":"v22.0.0"}]"#,
                ),
                (
                    "actions/setup-java",
                    "java-version",
                    "distribution: temurin, ",
                    "java",
                    Maven,
                    "21.0.12.1",
                    r#"{"releases":["jdk-21.0.12.1+1"]}"#,
                ),
            ] {
                let (session, input, output) = resolve_ci_runtime(CiRuntimeCase {
                    action,
                    field,
                    context,
                    name,
                    ecosystem,
                    requirement: version,
                    body,
                })
                .await;
                let runtime = runtime_suggestion(&output, name);
                assert_eq!(runtime.status, "current", "{name}: {runtime:?}");
                assert_eq!(runtime.latest.as_deref(), Some(version), "{name}");
                assert!(output.edits.is_empty(), "{name}: {output:?}");
                let lenses = session.analyze_document(input).code_lenses;
                let runtime_lenses = lenses
                    .iter()
                    .filter(|lens| lens.title.ends_with(&format!("latest {version}")))
                    .collect::<Vec<_>>();
                assert_eq!(runtime_lenses.len(), 1, "{name}: {lenses:?}");
                assert!(
                    runtime_lenses[0].action_kind().is_empty(),
                    "{name}: {lenses:?}"
                );
            }
        }

        #[tokio::test]
        async fn ci_runtime_pins_resolve_from_their_upstream_release_shapes_and_replay() {
            for (action, field, context, name, ecosystem, current, latest, body) in [
                (
                    "actions/setup-go",
                    "go-version",
                    "",
                    "go",
                    Go,
                    "1.24.0",
                    "1.25.1",
                    r#"[{"version":"go1.24.0","stable":true},{"version":"go1.25.1","stable":true}]"#,
                ),
                (
                    "denoland/setup-deno",
                    "deno-version",
                    "",
                    "deno",
                    Deno,
                    "2.0.0",
                    "2.1.0",
                    r#"{"cli":["v2.0.0","v2.1.0"]}"#,
                ),
                (
                    "actions/setup-python",
                    "python-version",
                    "",
                    "python",
                    Python,
                    "3.12.0",
                    "3.12.2",
                    r#"[{"version":"3.12.0","stable":true},{"version":"3.12.2","stable":true}]"#,
                ),
                (
                    "actions/setup-java",
                    "java-version",
                    "distribution: temurin, ",
                    "java",
                    Maven,
                    "17.0.1",
                    "17.0.2",
                    r#"{"releases":["jdk-17.0.1+12","jdk-17.0.2+9"]}"#,
                ),
                (
                    "actions/setup-dotnet",
                    "dotnet-version",
                    "",
                    "dotnet",
                    Dotnet,
                    "8.0.100",
                    "8.0.200",
                    r#"{"releases":[{"sdk":{"version":"8.0.100"},"sdks":[{"version":"8.0.200"}]}]}"#,
                ),
                (
                    "ruby/setup-ruby",
                    "ruby-version",
                    "",
                    "ruby",
                    Ruby,
                    "3.3.0",
                    "3.3.2",
                    r#"{"ruby":["3.3.0","3.3.2"],"jruby":["9.4.0.0"]}"#,
                ),
            ] {
                let (session, input, checked) = resolve_ci_runtime(CiRuntimeCase {
                    action,
                    field,
                    context,
                    name,
                    ecosystem,
                    requirement: current,
                    body,
                })
                .await;
                assert_eq!(checked.edits.len(), 1, "{name}: {checked:?}");
                assert_eq!(checked.edits[0].new_text, latest, "{name}");
                let replayed = session.resolve_document(input).await;
                assert_eq!(replayed.suggestions, checked.suggestions, "{name}");
                assert_eq!(replayed.edits, checked.edits, "{name}");
            }
        }

        #[tokio::test]
        async fn checked_runtime_constraints_preserve_their_declared_minimums() {
            let session = session_without_vulnerabilities();
            for (uri, language, text, name, ecosystem, body) in [
                (
                    "file:///work/package.json",
                    "json",
                    r#"{"engines":{"node":">=20"}}"#,
                    "node",
                    Npm,
                    r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
                ),
                (
                    "file:///work/Cargo.toml",
                    "toml",
                    "[package]\nname = \"example\"\nrust-version = \"1.9\"\n",
                    "rust",
                    Cargo,
                    r#"[{"name":"1.9.0"},{"name":"1.80.0"}]"#,
                ),
                (
                    "file:///work/package.json",
                    "json",
                    r#"{"engines":{"npm":"next"}}"#,
                    "npm",
                    Npm,
                    r#"{"dist-tags":{"next":"11.0.0-beta.1"},"versions":{"11.0.0-beta.1":{},"99.0.0":{}}}"#,
                ),
                (
                    "file:///work/package.json",
                    "json",
                    r#"{"devEngines":{"packageManager":{"name":"npm","version":"^10.0.0"}}}"#,
                    "npm",
                    Npm,
                    r#"{"versions":{"10.0.0":{},"10.9.0":{},"11.0.0":{}}}"#,
                ),
            ] {
                let input = DocumentInput::new(uri, language, text, None);
                let responses = [RegistryResponseInput::new(name, ecosystem, body)];
                let output = session
                    .resolve_document_with_responses(input.clone(), &responses)
                    .await;
                assert_eq!(output.edits.len(), 0);
                assert_eq!(output.suggestions[0].status, "satisfiesLatest");
                assert!(
                    session
                        .analyze_document(input.clone())
                        .code_lenses
                        .iter()
                        .all(|lens| lens.action_kind().is_empty())
                );
                let command = session
                    .apply_command_with_selected_version(crate::ApplyCommandRequest {
                        input,
                        command: Some("update"),
                        dependency_name: Some(name),
                        selected_version: Some("99.0.0"),
                        responses: &responses,
                    })
                    .await;
                assert_eq!(command.edits.len(), 0);
            }
        }

        #[tokio::test]
        async fn runtime_only_documents_resolve_cache_and_edit_pins() {
            for (file, text, name, ecosystem, body, selected) in [
                (
                    ".nvmrc",
                    "v20.0.0 # supported\n",
                    "node",
                    Npm,
                    r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
                    "v22.0.0",
                ),
                (
                    ".node-version",
                    "20.0.0\n",
                    "node",
                    Npm,
                    r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
                    "22.0.0",
                ),
                (
                    ".bun-version",
                    "1.0.0\n",
                    "bun",
                    Npm,
                    r#"[{"name":"bun-v1.0.0"},{"name":"bun-v1.2.0"}]"#,
                    "1.2.0",
                ),
                (
                    "rust-toolchain",
                    "1.80.0\n",
                    "rust",
                    Cargo,
                    r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
                    "1.90.0",
                ),
                (
                    "rust-toolchain.toml",
                    "[toolchain]\nchannel = '1.80.0' # keep\n",
                    "rust",
                    Cargo,
                    r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#,
                    "1.90.0",
                ),
            ] {
                let session = session_without_vulnerabilities();
                let provider = if ecosystem == Cargo { "cargo" } else { "npm" };
                let mut config = session.config.clone();
                config.enabled_providers =
                    vec![crate::enabled_provider_config_from_name(provider).unwrap()];
                let session = crate::version_lens_session(config.clone());
                let input =
                    DocumentInput::new(format!("file:///work/{file}"), "plaintext", text, None);
                let responses = [RegistryResponseInput::new(name, ecosystem, body)];
                let checked = session
                    .resolve_document_with_responses(input.clone(), &responses)
                    .await;
                assert_eq!(checked.suggestions.len(), 1, "{file}: {checked:?}");
                assert_eq!(checked.suggestions[0].status, "updateAvailable", "{file}");
                assert_eq!(checked.edits.len(), 1, "{file}");
                assert_eq!(checked.edits[0].new_text, selected);
                assert!(session.document_is_fresh(&input));
                let analysis = session.analyze_document(input.clone());
                assert!(analysis.is_supported_manifest);
                assert_ne!(analysis.code_lenses.len(), 0);
                let command = session
                    .apply_command_with_selected_version(crate::ApplyCommandRequest {
                        input: input.clone(),
                        command: Some("update"),
                        dependency_name: Some(name),
                        selected_version: None,
                        responses: &responses,
                    })
                    .await;
                assert_eq!(command.edits, checked.edits);
                config.enabled_providers =
                    vec![crate::enabled_provider_config_from_name("composer").unwrap()];
                let disabled = crate::version_lens_session(config).analyze_document(input);
                assert!(!disabled.is_supported_manifest);
                assert_eq!(disabled.dependencies.len(), 0);
            }
        }

        #[tokio::test]
        async fn floating_rust_channels_are_verified_without_changing_the_channel() {
            let session = session_without_vulnerabilities();
            let output = session.resolve_document_with_responses(
        DocumentInput::new("file:///work/.github/workflows/ci.yml", "yaml", "steps:\n  - uses: dtolnay/rust-toolchain@nightly\n", None),
        &[RegistryResponseInput::new("rust", Cargo, "date = \"2026-09-09\"\n[pkg.rust]\nversion = \"1.100.0-nightly (4aa1fbcf4 2026-09-08)\"\n")],
    ).await;
            assert_eq!(output.suggestions[0].status, "current");
            assert_eq!(
                output.suggestions[0].latest.as_deref(),
                Some("1.100.0-nightly")
            );
            assert_eq!(output.edits.len(), 0);
        }

        #[tokio::test]
        async fn unresolved_runtime_expressions_are_skipped_without_an_error() {
            let session = session_without_vulnerabilities();
            let input = DocumentInput::new(
                "file:///work/.github/workflows/ci.yml",
                "yaml",
                "steps:\n  - uses: actions/setup-node@v4\n    with:\n      node-version: ${{ inputs.node }}\n",
                None,
            );
            let output = session
                .resolve_document_with_responses(
                    input.clone(),
                    &[RegistryResponseInput::new(
                        "actions/setup-node",
                        GitHub,
                        r#"[{"name":"v4"}]"#,
                    )],
                )
                .await;
            assert!(
                output
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.dependency.name != "node"),
                "expression was looked up: {:?}",
                output.suggestions
            );
            assert!(
                output
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.status != "error"),
                "{:?}",
                output.suggestions
            );
            assert!(session.document_is_fresh(&input));
        }
    }

    fn lens_titles(output: &crate::AnalyzeDocumentOutput) -> Vec<&str> {
        output
            .code_lenses
            .iter()
            .map(|lens| lens.title.as_str())
            .collect()
    }

    fn lens_titles_and_arguments(
        output: &crate::AnalyzeDocumentOutput,
    ) -> (Vec<&str>, Vec<Vec<String>>) {
        (
            lens_titles(output),
            crate::test_support::code_lens_arguments(output),
        )
    }

    fn registry_case(
        input: &DocumentInput,
    ) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
        crate::test_support::registry_case(input)
    }

    pub(super) fn registry_case_with_expected_urls(
        input: &DocumentInput,
        expected_urls: &[&[&str]],
    ) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
        let (session, context, dependencies) = registry_case(input);
        for (dependency, expected) in dependencies.iter().zip(expected_urls) {
            assert_eq!(
                session.registry_urls_with_context(dependency, &context),
                expected
                    .iter()
                    .map(|url| (*url).to_owned())
                    .collect::<Vec<_>>()
            );
        }
        (session, context, dependencies)
    }

    pub(super) fn assert_single_auth_header(
        context: &RegistryContext,
        ecosystem: Ecosystem,
        url: &str,
        value: &str,
    ) {
        let headers = context.auth_headers_for_url(ecosystem, url);
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].name, "authorization");
        assert_eq!(headers[0].value, value);
    }

    pub(super) fn assert_dependency_registry_url(
        session: &VersionLensSession,
        dependencies: &[Dependency],
        index: usize,
        context: &RegistryContext,
        expected: &[&str],
    ) {
        assert_eq!(
            session.registry_urls_with_context(&dependencies[index], context),
            expected
                .iter()
                .map(|url| (*url).to_owned())
                .collect::<Vec<_>>()
        );
    }

    pub(super) fn assert_fixed_git_repository_suggestion(output: &ResolveDocumentOutput) {
        assert_eq!(output.suggestions[0].status, "fixed");
        assert_eq!(
            output.suggestions[0].latest.as_deref(),
            Some("git repository")
        );
    }

    pub(super) fn assert_update(output: &ResolveDocumentOutput, expected: &str) {
        assert_eq!(output.suggestions[0].status, "updateAvailable");
        assert_eq!(output.edits[0].new_text, expected);
    }
}
