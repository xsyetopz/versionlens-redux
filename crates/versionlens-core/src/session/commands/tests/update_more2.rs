use versionlens_parsers::Ecosystem::{Conan, Cran, Hackage, Hex, Julia, Opam, Pub, Ruby, Swift, Vcpkg};
#[test]
fn apply_command_updates_deno_jsr_import_aliases_by_specifier_package() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deno.json".to_owned(),
            language_id: "jsonc".to_owned(),
            text: package_file_fixture("apply-command-updates-deno-jsr-import-aliases-by-specifier-package.json"),
            workspace_root: None,
        },
        None,
        Some("luca"),
        &[RegistryResponseInput {
            package: "@luca/cases".to_owned(),
            ecosystem: Deno,
            body: r#"{"versions":{"1.1.0":{},"1.0.0":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:@luca/cases@1.1.0");
}

#[test]
fn apply_command_updates_import_map_directory_specifier_preserving_slashes() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///import_map.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-import-map-directory-specifier-preserving-slashes.json"),
            workspace_root: None,
        },
        None,
        Some("@std/async/"),
        &[RegistryResponseInput {
            package: "@std/async".to_owned(),
            ecosystem: Deno,
            body: r#"{"versions":{"2.0.0":{},"1.0.0":{}}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "jsr:/@std/async@^2.0.0/");
}

#[test]
fn apply_command_updates_conanfile_txt_dependency_preserving_revision_suffix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///conanfile.txt".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-conanfile-txt-dependency-preserving-revision-suffix.txt"),
            workspace_root: None,
        },
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput {
            package: "zlib".to_owned(),
            ecosystem: Conan,
            body: r#"{"results":["zlib/1.2.13","zlib/1.3.1"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.3.1#rev0");
}

#[test]
fn apply_command_updates_stack_resolver_from_stackage_snapshot_index() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///stack.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-updates-stack-resolver-from-stackage-snapshot-index.yaml"),
            workspace_root: None,
        },
        Some("update"),
        Some("stackage-lts"),
        &[RegistryResponseInput {
            package: "stackage-lts".to_owned(),
            ecosystem: Hackage,
            body: r#"{"snapshots":[[["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "24.49");
}

#[test]
fn apply_command_updates_gemspec_dependency_preserving_ruby_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///example.gemspec".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("apply-command-updates-gemspec-dependency-preserving-ruby-operator.gemspec"),
            workspace_root: None,
        },
        Some("update"),
        Some("rack"),
        &[RegistryResponseInput {
            package: "rack".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"number":"3.0.0"},{"number":"2.2.9"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "~> 3.0.0");
}

#[test]
fn apply_command_updates_pub_hosted_dependency_without_version_by_inserting_version() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///pubspec.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-updates-pub-hosted-dependency-without-version-by-inserting-version.yaml"),
            workspace_root: None,
        },
        Some("update"),
        Some("hosted_dep"),
        &[RegistryResponseInput {
            package: "hosted_alias".to_owned(),
            ecosystem: Pub,
            body: r#"{"latest":{"version":"2.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "\n    version: 2.0.0");
}

#[test]
fn apply_command_updates_gleam_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///gleam.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-gleam-project-version-by-requested-level.toml"),
            workspace_root: None,
        },
        Some("updateMinor"),
        Some("my_package"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "version");
    assert_eq!(output.suggestions[0].dependency.name, "my_package");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "1.3.0");
}

#[test]
fn apply_command_updates_gleam_dependency_preserving_requirement_syntax() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///gleam.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-gleam-dependency-preserving-requirement-syntax.toml"),
            workspace_root: None,
        },
        Some("update"),
        Some("gleam_stdlib"),
        &[RegistryResponseInput {
            package: "gleam_stdlib".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"2.0.0"},{"version":"0.44.0"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_updates_rebar_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///rebar.config".to_owned(),
            language_id: "erlang".to_owned(),
            text: package_file_fixture("apply-command-updates-rebar-dependency-version.config"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("cowboy"),
        selected_version: Some("2.13.0"),
        responses: &[RegistryResponseInput {
            package: "cowboy".to_owned(),
            ecosystem: Hex,
            body: r#"{"releases":[{"version":"2.13.0"},{"version":"2.12.0"}]}"#.to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.13.0");
}

#[test]
fn apply_command_updates_opam_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///demo.opam".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-opam-dependency-preserving-operator.opam"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("lwt"),
        selected_version: Some("6.1.2"),
        responses: &[RegistryResponseInput {
            package: "lwt".to_owned(),
            ecosystem: Opam,
            body: r#"<h2>lwt version</h2><p>6.1.2 (latest)</p>"#.to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, ">= \"6.1.2\"");
}

#[test]
fn apply_command_updates_dune_project_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///dune-project".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-dune-project-dependency-preserving-operator.dune-project"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("fmt"),
        selected_version: Some("0.9.0"),
        responses: &[RegistryResponseInput {
            package: "fmt".to_owned(),
            ecosystem: Opam,
            body: r#"<h2>fmt version</h2><p>0.9.0 (latest)</p>"#.to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, ">= 0.9.0");
}

#[test]
fn apply_command_updates_cabal_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///demo.cabal".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-cabal-dependency-preserving-operator.cabal"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("base"),
        selected_version: Some("4.20.0.0"),
        responses: &[RegistryResponseInput {
            package: "base".to_owned(),
            ecosystem: Hackage,
            body: r#"{"4.20.0.0":"normal","4.19.2.0":"normal"}"#.to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, ">= 4.20.0.0");
}

#[test]
fn apply_command_updates_julia_compat_dependency() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///Project.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("apply-command-updates-julia-compat-dependency.toml"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("Example"),
        selected_version: Some("0.6.0"),
        responses: &[RegistryResponseInput {
            package: "Example".to_owned(),
            ecosystem: Julia,
            body: r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0]
git-tree-sha1 = "d6f6"
"#
            .to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.6.0");
}

#[test]
fn apply_command_updates_r_description_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///DESCRIPTION".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-r-description-dependency-preserving-operatorDESCRIPTION"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("dplyr"),
        selected_version: Some("1.1.4"),
        responses: &[RegistryResponseInput {
            package: "dplyr".to_owned(),
            ecosystem: Cran,
            body: "Package: dplyr\nVersion: 1.1.4\n".to_owned(),
        }],
    });
    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, ">= 1.1.4");
}

#[test]
fn apply_command_updates_paket_dependencies_nuget_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput {
            uri: "file:///paket.dependencies".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-updates-paket-dependencies-nuget-version.dependencies"),
            workspace_root: None,
        },
        command: Some("update"),
        dependency_name: Some("Newtonsoft.Json"),
        selected_version: Some("13.0.3"),
        responses: &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["13.0.1","13.0.3"]}"#.to_owned(),
        }],
    });

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].dependency.group, "paket.dependencies");
    assert_eq!(output.suggestions[0].dependency.requirement, "13.0.1");
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "13.0.3");
}

#[test]
fn apply_command_does_not_update_paket_references_without_versions() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///paket.references".to_owned(),
            language_id: "plaintext".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-paket-references-without-versions.references"),
            workspace_root: None,
        },
        Some("update"),
        Some("Newtonsoft.Json"),
        &[RegistryResponseInput {
            package: "Newtonsoft.Json".to_owned(),
            ecosystem: Dotnet,
            body: r#"{"versions":["13.0.3"]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_update_dockerfile_digest_pinned_image() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Dockerfile".to_owned(),
            language_id: "dockerfile".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-dockerfile-digest-pinned-imageDockerfile"),
            workspace_root: None,
        },
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput {
            package: "ubuntu".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_update_compose_digest_pinned_image() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///compose.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-compose-digest-pinned-image.yaml"),
            workspace_root: None,
        },
        Some("update"),
        Some("ubuntu"),
        &[RegistryResponseInput {
            package: "ubuntu".to_owned(),
            ecosystem: Docker,
            body: r#"{"results":[{"name":"24.04","tag_status":"active","digest":"sha256-new"}]}"#
                .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_vcpkg_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///vcpkg.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-vcpkg-version-constraint.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("fmt"),
        &[RegistryResponseInput {
            package: "fmt".to_owned(),
            ecosystem: Vcpkg,
            body: r#"{"versions":[{"version":"11.1.4"},{"version":"10.1.1#1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "11.1.4");
}

#[test]
fn apply_command_does_not_update_vcpkg_baseline_dependency_without_version_constraint() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///vcpkg.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-vcpkg-baseline-dependency-without-version-constraint.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput {
            package: "zlib".to_owned(),
            ecosystem: Vcpkg,
            body: r#"{"versions":[{"version":"1.3.1"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_swift_package_github_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Package.swift".to_owned(),
            language_id: "swift".to_owned(),
            text: package_file_fixture("apply-command-updates-swift-package-github-dependency.swift"),
            workspace_root: None,
        },
        Some("update"),
        Some("swift-nio"),
        &[RegistryResponseInput {
            package: "apple/swift-nio".to_owned(),
            ecosystem: Swift,
            body: r#"[{"name":"2.66.0"},{"name":"2.65.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.66.0");
}
