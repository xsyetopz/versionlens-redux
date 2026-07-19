#[test]
fn disabled_providers_are_filtered_in_rust() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![EnabledProviderConfig {
            ecosystem: Cargo,
            manifest_kind: None,
        }],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: true,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-dependencies.json"),
        workspace_root: None,
    });

    assert!(output.dependencies.is_empty());
    assert!(output.code_lenses.is_empty());
    assert!(output.diagnostics.is_empty());
    assert!(!output.is_supported_manifest);
    assert!(!output.status.visible);
}

#[test]
fn enabled_npm_provider_enables_package_json5() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Npm,
        manifest_kind: Some(NpmPackageJson),
    });
    let input = DocumentInput {
        uri: "file:///package.json5".to_owned(),
        language_id: "json5".to_owned(),
        text: package_file_fixture("package.json5"),
        workspace_root: None,
    };

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

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("package.yaml"),
        workspace_root: None,
    });

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

    let output = session.analyze_document(DocumentInput {
        uri: "file:///pnpm-workspace.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-workspace.yaml"),
        workspace_root: None,
    });

    assert!(!output.is_supported_manifest);
    assert!(output.dependencies.is_empty());
}

#[test]
fn enabled_pnpm_provider_does_not_enable_package_json() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Npm,
        manifest_kind: Some(PnpmYaml),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-dependencies.json"),
        workspace_root: None,
    });

    assert!(!output.is_supported_manifest);
    assert!(output.dependencies.is_empty());
}

#[test]
fn enabled_deno_provider_enables_import_map_json() {
    let session = session_with_enabled_provider(EnabledProviderConfig {
        ecosystem: Deno,
        manifest_kind: Some(DenoJson),
    });

    let output = session.analyze_document(DocumentInput {
        uri: "file:///import_map.json".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("import_map.json"),
        workspace_root: None,
    });

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

    let output = session.analyze_document(DocumentInput {
        uri: "file:///deno.json".to_owned(),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno.json"),
        workspace_root: None,
    });

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
    let input = DocumentInput {
        uri: "file:///mix.exs".to_owned(),
        language_id: "elixir".to_owned(),
        text: package_file_fixture("mix.exs"),
        workspace_root: None,
    };

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
    let input = DocumentInput {
        uri: "file:///rebar.config".to_owned(),
        language_id: "erlang".to_owned(),
        text: package_file_fixture("rebar.config"),
        workspace_root: None,
    };

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
    let input = DocumentInput {
        uri: "file:///gleam.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("gleam.toml"),
        workspace_root: None,
    };

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
    let input = DocumentInput {
        uri: "file:///demo.opam".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("demo.opam"),
        workspace_root: None,
    };

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
    let input = DocumentInput {
        uri: "file:///dune-project".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("dune-project"),
        workspace_root: None,
    };

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
    let input = DocumentInput {
        uri: "file:///demo.cabal".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("demo.cabal"),
        workspace_root: None,
    };

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

    let project_dependencies = session.dependencies(&DocumentInput {
        uri: "file:///Project.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("Project.toml"),
        workspace_root: None,
    });
    assert_eq!(project_dependencies.len(), 2);
    assert_eq!(project_dependencies[0].name, "Demo");
    assert_eq!(project_dependencies[0].group, "version");
    assert_eq!(project_dependencies[1].name, "Example");
    assert_eq!(project_dependencies[1].group, "compat");

    let manifest_dependencies = session.dependencies(&DocumentInput {
        uri: "file:///Manifest-v1.11.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("Manifest-v1.11.toml"),
        workspace_root: None,
    });
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

    let description_dependencies = session.dependencies(&DocumentInput {
        uri: "file:///DESCRIPTION".to_owned(),
        language_id: "plaintext".to_owned(),
        text: package_file_fixture("DESCRIPTION"),
        workspace_root: None,
    });
    assert_eq!(description_dependencies.len(), 2);
    assert_eq!(description_dependencies[1].name, "dplyr");
    assert_eq!(description_dependencies[1].group, "Imports");

    let renv_dependencies = session.dependencies(&DocumentInput {
        uri: "file:///renv.lock".to_owned(),
        language_id: "json".to_owned(),
        text: package_file_fixture("renv.lock"),
        workspace_root: None,
    });
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
    let input = DocumentInput {
        uri: "file:///example.gemspec".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("example.gemspec"),
        workspace_root: None,
    };

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

    let output = session.analyze_document(DocumentInput {
        uri: "file:///workspace/example.gemspec".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("development.example.gemspec"),
        workspace_root: None,
    });

    assert!(output.is_supported_manifest);
    assert_eq!(output.dependencies.len(), 1);
    assert_eq!(output.dependencies[0].ecosystem, "ruby");
    assert_eq!(output.dependencies[0].group, "add_development_dependency");
    assert_eq!(output.dependencies[0].name, "rspec");
    assert_eq!(output.dependencies[0].requirement, "~> 3.13");
}
