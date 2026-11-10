use crate::dependency::properties::is_enabled;
use std::sync::PoisonError;
use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind};
use versionlens_parsers::ecosystem_for_manifest;
use versionlens_parsers::parse_document_as_manifest_with_dependency_paths;

use crate::DependencyPropertyConfig;
use crate::VersionLensSession;
use versionlens_fetch::CacheKey;

impl VersionLensSession {
    pub(crate) fn dependencies(&self, input: &DocumentInput) -> Vec<Dependency> {
        let kind = self.classify_document(input);
        if let Some(ecosystem) = ecosystem_for_manifest(kind)
            && !self.provider_enabled_for_manifest(kind, ecosystem)
        {
            return vec![];
        }

        let dependency_paths = self.dependency_paths_for_manifest(kind);
        let key = serde_json::to_vec(&(
            &input.uri,
            &input.language_id,
            &input.text,
            &input.workspace_root,
            kind,
            &self.config.providers.dependency_properties,
        ))
        .ok()
        .map(|identity| CacheKey::content(&identity));
        let mut cache = self
            .result_state
            .parsed_dependencies
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if let Some(key) = &key
            && let Some(dependencies) = cache.get(key)
        {
            return dependencies.clone();
        }
        let dependencies: Vec<_> =
            parse_document_as_manifest_with_dependency_paths(input, kind, &dependency_paths)
                .into_iter()
                .filter(|dependency| self.dependency_property_enabled(dependency, kind))
                .collect();
        if let Some(key) = key {
            cache.insert(key, dependencies.clone());
        }
        dependencies
    }

    /// Dependencies plus the project version fields that only the
    /// project-version commands edit, such as the npm and Cargo `version`.
    pub(crate) fn command_dependencies(&self, input: &DocumentInput) -> Vec<Dependency> {
        let mut dependencies = self.dependencies(input);
        let kind = self.classify_document(input);
        if ecosystem_for_manifest(kind)
            .is_none_or(|ecosystem| self.provider_enabled_for_manifest(kind, ecosystem))
        {
            dependencies.extend(versionlens_parsers::parse_project_versions_as_manifest(
                input, kind,
            ));
        }
        dependencies
    }

    fn dependency_paths_for_manifest(&self, kind: ManifestKind) -> Vec<&str> {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return vec![];
        };

        self.dependency_property_configs(kind, ecosystem)
            .flat_map(|config| config.properties.iter().map(String::as_str))
            .collect()
    }

    fn dependency_property_enabled(&self, dependency: &Dependency, kind: ManifestKind) -> bool {
        let Some(ecosystem) = ecosystem_for_manifest(kind) else {
            return true;
        };

        is_enabled(
            dependency,
            ecosystem,
            self.dependency_property_configs(kind, ecosystem),
        )
    }

    fn dependency_property_configs(
        &self,
        kind: ManifestKind,
        ecosystem: Ecosystem,
    ) -> impl Iterator<Item = &DependencyPropertyConfig> {
        self.config
            .providers
            .dependency_properties
            .iter()
            .filter(move |config| config.ecosystem == ecosystem && config.applies_to_manifest(kind))
    }
}

#[cfg(test)]
mod tests {
    use versionlens_model::DocumentInput;

    use crate::{
        EnabledProviderConfig, FilePatternConfig, ProviderSettings, SessionConfig,
        VersionLensSession,
    };
    use versionlens_model::Ecosystem::*;
    use versionlens_model::ManifestKind::{
        Cabal, ComposerJson, DenoJson, DockerComposeYaml, DubJson, Gemfile, GleamToml,
        JuliaProjectToml, MixExs, NpmPackageJson, Opam, PnpmYaml, PythonRequirementsTxt,
        RDescription, RebarConfig,
    };

    use versionlens_model::Ecosystem::Opam as OpamEcosystem;

    #[test]
    fn disabled_providers_are_filtered_in_rust() {
        let session = crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![EnabledProviderConfig {
                ecosystem: Cargo,
                manifest_kind: None,
            }],
            providers: ProviderSettings::default(),
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: true,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("npm-dependencies.json"),
            None,
        ));

        assert_eq!(output.dependencies.len(), 0);
        assert_eq!(output.code_lenses.len(), 0);
        assert_eq!(output.diagnostics.len(), 0);
        assert!(!output.is_supported_manifest);
        assert!(!output.status.visible);
    }

    #[test]
    fn enabled_npm_provider_enables_package_json5() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Npm,
            manifest_kind: Some(NpmPackageJson),
        });
        let input = DocumentInput::new(
            "file:///package.json5".to_owned(),
            "json5".to_owned(),
            package_file_fixture("package.json5"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "left-pad");
    }

    #[test]
    fn enabled_npm_provider_enables_package_yaml() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Npm,
            manifest_kind: Some(NpmPackageJson),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///package.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("package.yaml"),
            None,
        ));

        assert!(output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 1);
        assert_eq!(output.dependencies[0].name, "react");
    }

    #[test]
    fn enabled_npm_provider_does_not_enable_pnpm_yaml() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Npm,
            manifest_kind: Some(NpmPackageJson),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///pnpm-workspace.yaml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("pnpm-workspace.yaml"),
            None,
        ));

        assert!(!output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 0);
    }

    #[test]
    fn enabled_pnpm_provider_does_not_enable_package_json() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Npm,
            manifest_kind: Some(PnpmYaml),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///package.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("npm-dependencies.json"),
            None,
        ));

        assert!(!output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 0);
    }

    #[test]
    fn enabled_deno_provider_enables_import_map_json() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Deno,
            manifest_kind: Some(DenoJson),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///import_map.json".to_owned(),
            "json".to_owned(),
            package_file_fixture("import_map.json"),
            None,
        ));

        assert!(output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 1);
        assert_eq!(output.dependencies[0].name, "@std/assert");
    }

    #[test]
    fn enabled_deno_provider_keeps_npm_prefixed_deno_imports() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Deno,
            manifest_kind: None,
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///deno.json".to_owned(),
            "jsonc".to_owned(),
            package_file_fixture("deno.json"),
            None,
        ));

        assert!(output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 1);
        assert_eq!(output.dependencies[0].ecosystem, "npm");
        assert_eq!(output.dependencies[0].name, "chalk");
    }

    #[test]
    fn enabled_hex_provider_enables_mix_exs() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Hex,
            manifest_kind: Some(MixExs),
        });
        let input = DocumentInput::new(
            "file:///mix.exs".to_owned(),
            "elixir".to_owned(),
            package_file_fixture("mix.exs"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "plug");
        assert_eq!(dependencies[0].group, "deps");
    }

    #[test]
    fn enabled_hex_provider_enables_rebar_config() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Hex,
            manifest_kind: Some(RebarConfig),
        });
        let input = DocumentInput::new(
            "file:///rebar.config".to_owned(),
            "erlang".to_owned(),
            package_file_fixture("rebar.config"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "cowboy");
        assert_eq!(dependencies[0].group, "deps");
    }

    #[test]
    fn enabled_hex_provider_enables_gleam_toml() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Hex,
            manifest_kind: Some(GleamToml),
        });
        let input = DocumentInput::new(
            "file:///gleam.toml".to_owned(),
            "toml".to_owned(),
            package_file_fixture("gleam.toml"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "gleam_stdlib");
        assert_eq!(dependencies[0].group, "dependencies");
    }

    #[test]
    fn enabled_opam_provider_enables_opam_files() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: OpamEcosystem,
            manifest_kind: Some(Opam),
        });
        let input = DocumentInput::new(
            "file:///demo.opam".to_owned(),
            "plaintext".to_owned(),
            package_file_fixture("demo.opam"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "lwt");
        assert_eq!(dependencies[0].group, "depends");
    }

    #[test]
    fn enabled_opam_provider_enables_dune_project_files() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: OpamEcosystem,
            manifest_kind: Some(Opam),
        });
        let input = DocumentInput::new(
            "file:///dune-project".to_owned(),
            "plaintext".to_owned(),
            package_file_fixture("dune-project"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "fmt");
        assert_eq!(dependencies[0].group, "depends");
    }

    #[test]
    fn enabled_hackage_provider_enables_cabal_files() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Hackage,
            manifest_kind: Some(Cabal),
        });
        let input = DocumentInput::new(
            "file:///demo.cabal".to_owned(),
            "plaintext".to_owned(),
            package_file_fixture("demo.cabal"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[1].name, "base");
        assert_eq!(dependencies[1].group, "build-depends");
    }

    #[test]
    fn enabled_julia_provider_enables_project_and_manifest_files() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Julia,
            manifest_kind: Some(JuliaProjectToml),
        });

        let project_dependencies = session.dependencies(&DocumentInput::new(
            "file:///Project.toml".to_owned(),
            "toml".to_owned(),
            package_file_fixture("Project.toml"),
            None,
        ));
        assert_eq!(project_dependencies.len(), 2);
        assert_eq!(project_dependencies[0].name, "Demo");
        assert_eq!(project_dependencies[0].group, "version");
        assert_eq!(project_dependencies[1].name, "Example");
        assert_eq!(project_dependencies[1].group, "compat");

        let manifest_dependencies = session.dependencies(&DocumentInput::new(
            "file:///Manifest-v1.11.toml".to_owned(),
            "toml".to_owned(),
            package_file_fixture("Manifest-v1.11.toml"),
            None,
        ));
        assert_eq!(manifest_dependencies.len(), 1);
        assert_eq!(manifest_dependencies[0].name, "Example");
        assert_eq!(manifest_dependencies[0].group, "deps");
    }

    #[test]
    fn enabled_cran_provider_enables_description_and_renv_lock_files() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Cran,
            manifest_kind: Some(RDescription),
        });

        let description_dependencies = session.dependencies(&DocumentInput::new(
            "file:///DESCRIPTION".to_owned(),
            "plaintext".to_owned(),
            package_file_fixture("DESCRIPTION"),
            None,
        ));
        assert_eq!(description_dependencies.len(), 2);
        assert_eq!(description_dependencies[1].name, "dplyr");
        assert_eq!(description_dependencies[1].group, "Imports");

        let renv_dependencies = session.dependencies(&DocumentInput::new(
            "file:///renv.lock".to_owned(),
            "json".to_owned(),
            package_file_fixture("renv.lock"),
            None,
        ));
        assert_eq!(renv_dependencies.len(), 1);
        assert_eq!(renv_dependencies[0].name, "dplyr");
        assert_eq!(renv_dependencies[0].group, "Packages");
    }

    #[test]
    fn enabled_ruby_provider_enables_gemspec() {
        let session = session_with_enabled_provider(EnabledProviderConfig {
            ecosystem: Ruby,
            manifest_kind: Some(Gemfile),
        });
        let input = DocumentInput::new(
            "file:///example.gemspec".to_owned(),
            "ruby".to_owned(),
            package_file_fixture("example.gemspec"),
            None,
        );

        let dependencies = session.dependencies(&input);

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "rack");
        assert_eq!(dependencies[0].group, "add_dependency");
    }

    #[test]
    fn configured_ruby_file_pattern_routes_gemspec_matches_to_gemspec_parser() {
        let session = session_with_file_pattern(FilePatternConfig {
            manifest_kind: Gemfile,
            pattern: "**/*.gemspec".to_owned(),
        });

        let output = session.analyze_document(DocumentInput::new(
            "file:///workspace/example.gemspec".to_owned(),
            "ruby".to_owned(),
            package_file_fixture("development.example.gemspec"),
            None,
        ));

        assert!(output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 1);
        assert_eq!(output.dependencies[0].ecosystem, "ruby");
        assert_eq!(output.dependencies[0].group, "add_development_dependency");
        assert_eq!(output.dependencies[0].name, "rspec");
        assert_eq!(output.dependencies[0].requirement, "~> 3.13");
    }
    #[derive(Clone, Copy)]
    struct ConfiguredPatternCase<'a> {
        manifest_kind: versionlens_model::ManifestKind,
        pattern: &'a str,
        uri: &'a str,
        language: &'a str,
        fixture: &'a str,
        workspace_root: Option<&'a str>,
        expected_ecosystem: &'a str,
        expected_group: &'a str,
        expected_name: &'a str,
        expected_requirement: &'a str,
    }

    fn assert_configured_pattern(case: ConfiguredPatternCase<'_>) {
        let session = session_with_file_pattern(FilePatternConfig {
            manifest_kind: case.manifest_kind,
            pattern: case.pattern.to_owned(),
        });
        let output = session.analyze_document(DocumentInput::new(
            case.uri.to_owned(),
            case.language.to_owned(),
            package_file_fixture(case.fixture),
            case.workspace_root.map(str::to_owned),
        ));

        assert!(output.is_supported_manifest);
        assert_eq!(output.dependencies.len(), 1);
        let dependency = &output.dependencies[0];
        assert_eq!(dependency.ecosystem, case.expected_ecosystem);
        assert_eq!(dependency.group, case.expected_group);
        assert_eq!(dependency.name, case.expected_name);
        assert_eq!(dependency.requirement, case.expected_requirement);
    }

    #[test]
    fn configured_file_pattern_classifies_custom_composer_manifest() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/acme.composer.json",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_brace_alternatives() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/{composer.json,acme.composer.json}",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_workspace_relative_recursive_segments() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "packages/**/acme.composer.json",
            uri: "file:///workspace/packages/backend/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: Some("/workspace"),
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_character_classes() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/acme.composer.jso[n]",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_character_class_ranges() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/acme.composer.jso[m-o]",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_negated_character_classes() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/acme.composer.jso[!x]",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_file_pattern_supports_micromatch_extglob_alternatives() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: ComposerJson,
            pattern: "**/@(composer|acme.composer).json",
            uri: "file:///workspace/acme.composer.json",
            language: "json",
            fixture: "acme.composer.json",
            workspace_root: None,
            expected_ecosystem: "composer",
            expected_group: "require",
            expected_name: "acme/package",
            expected_requirement: "1.2.3",
        });
    }

    #[test]
    fn configured_docker_file_pattern_routes_non_yaml_matches_to_dockerfile_parser() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: DockerComposeYaml,
            pattern: "**/Containerfile",
            uri: "file:///workspace/Containerfile",
            language: "plaintext",
            fixture: "Containerfile",
            workspace_root: None,
            expected_ecosystem: "docker",
            expected_group: "FROM",
            expected_name: "node",
            expected_requirement: "20",
        });
    }

    #[test]
    fn configured_pypi_file_pattern_routes_non_txt_matches_to_toml_parser() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: PythonRequirementsTxt,
            pattern: "**/pyproject-prod.toml",
            uri: "file:///workspace/pyproject-prod.toml",
            language: "toml",
            fixture: "pyproject-prod.toml",
            workspace_root: None,
            expected_ecosystem: "pypi",
            expected_group: "project.dependencies",
            expected_name: "requests",
            expected_requirement: "==2.32.0",
        });
    }

    #[test]
    fn configured_dub_file_pattern_routes_sdl_matches_to_sdl_parser() {
        assert_configured_pattern(ConfiguredPatternCase {
            manifest_kind: DubJson,
            pattern: "**/*.sdl",
            uri: "file:///workspace/dub.sdl",
            language: "plaintext",
            fixture: "dub.sdl",
            workspace_root: None,
            expected_ecosystem: "dub",
            expected_group: "dependencies",
            expected_name: "vibe-d",
            expected_requirement: "~>0.9.7",
        });
    }

    #[test]
    fn changed_snapshot_reconstructs_requirements_and_unicode_ranges() {
        let session = crate::test_support::test_session(false);
        let initial = DocumentInput::new(
            "file:///package.json",
            "json",
            r#"{"dependencies":{"example":"1.0.0"}}"#,
            None,
        );
        let first = session.dependencies(&initial);
        let mut changed = initial.clone();
        changed.text =
            "{\n  \"description\": \"🦀\", \"dependencies\": {\"example\": \"2.0.0\"}\n}"
                .to_owned();
        let second = session.dependencies(&changed);
        assert_eq!(first[0].requirement, "1.0.0");
        assert_eq!(second[0].requirement, "2.0.0");
        assert_eq!(second[0].requirement_range.start.line, 1);
        assert_ne!(first[0].requirement_range, second[0].requirement_range);
        assert_eq!(session.dependencies(&initial), first);
        assert_eq!(session.dependencies(&changed), second);
    }

    #[test]
    fn configured_patterns_match_decoded_workspace_paths() {
        let session = session_with_file_pattern(FilePatternConfig {
            manifest_kind: NpmPackageJson,
            pattern: "ci files/dépendances.custom".to_owned(),
        });
        for root in [
            "/workspace/project files",
            "file:///workspace/project%20files",
        ] {
            let input = DocumentInput {
                uri: "file:///workspace/project%20files/ci%20files/d%C3%A9pendances.custom"
                    .to_owned(),
                language_id: "plaintext".to_owned(),
                text: r#"{"dependencies":{"example":"1.0.0"}}"#.to_owned(),
                workspace_root: Some(root.to_owned()),
                version: None,
            };
            let analysis = session.analyze_document(input);
            assert!(analysis.is_supported_manifest);
            assert_eq!(analysis.dependencies.len(), 1);
            assert_eq!(analysis.dependencies[0].name, "example");
        }
    }
    fn session_with_enabled_provider(provider: EnabledProviderConfig) -> VersionLensSession {
        crate::version_lens_session(SessionConfig {
            update_severity: Some(crate::Severity::Information),
            cache_ttl_ms: 300_000,
            enabled_providers: vec![provider],
            providers: ProviderSettings::default(),
            suggestion_indicators: crate::standard_suggestion_indicators(),
            show_vulnerabilities: true,
            show_suggestion_stats: false,
            show_prereleases: false,
            http: versionlens_fetch::standard_http_config(),
        })
    }

    fn session_with_file_pattern(file_pattern: FilePatternConfig) -> VersionLensSession {
        crate::version_lens_session(crate::test_support::session_config(
            ProviderSettings {
                file_patterns: vec![file_pattern],
                ..Default::default()
            },
            true,
        ))
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/session/dependencies", name)
    }
}
