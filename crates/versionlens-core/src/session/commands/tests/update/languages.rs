#[test]
fn apply_command_updates_deno_jsr_import_aliases_by_specifier_package() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///deno.json".to_owned(), "jsonc".to_owned(), package_file_fixture(
                "command-updates-deno-jsr-import-aliases-by-specifier-package.json",
            ), None),
        None,
        Some("luca"),
        &[RegistryResponseInput::new("@luca/cases".to_owned(), Deno, r#"{"versions":{"1.1.0":{},"1.0.0":{}}}"#.to_owned())],
    );

    assert_single_edit(&output, "jsr:@luca/cases@1.1.0");
}

#[test]
fn apply_command_updates_import_map_directory_specifier_preserving_slashes() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///import_map.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-updates-import-map-directory-specifier-preserving-slashes.json",
            ), None),
        None,
        Some("@std/async/"),
        &[RegistryResponseInput::new("@std/async".to_owned(), Deno, r#"{"versions":{"2.0.0":{},"1.0.0":{}}}"#.to_owned())],
    );

    assert_single_edit(&output, "jsr:/@std/async@^2.0.0/");
}

#[test]
fn apply_command_updates_conanfile_txt_dependency_preserving_revision_suffix() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///conanfile.txt".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-conanfile-txt-dependency-preserving-revision-suffix.txt",
            ), None),
        Some("update"),
        Some("zlib"),
        &[RegistryResponseInput::new("zlib".to_owned(), Conan, r#"{"results":["zlib/1.2.13","zlib/1.3.1"]}"#.to_owned())],
    );

    assert_single_edit(&output, "1.3.1#rev0");
}

#[test]
fn apply_command_updates_stack_resolver_from_stackage_snapshot_index() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///stack.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-stack-resolver-from-stackage-snapshot-index.yaml"), None),
        Some("update"),
        Some("stackage-lts"),
        &[RegistryResponseInput::new("stackage-lts".to_owned(), Hackage, r#"{"snapshots":[[["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#.to_owned())],
    );

    assert_single_edit(&output, "24.49");
}

#[test]
fn apply_command_updates_gemspec_dependency_preserving_ruby_operator() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///example.gemspec".to_owned(), "ruby".to_owned(), package_file_fixture(
                "command-updates-gemspec-dependency-preserving-ruby-operator.gemspec",
            ), None),
        Some("update"),
        Some("rack"),
        &[RegistryResponseInput::new("rack".to_owned(), Ruby, r#"[{"number":"3.0.0"},{"number":"2.2.9"}]"#.to_owned())],
    );

    assert_single_edit(&output, "~> 3.0.0");
}

#[test]
fn apply_command_updates_pub_hosted_dependency_without_version_by_inserting_version() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///pubspec.yaml".to_owned(), "yaml".to_owned(), package_file_fixture("command-updates-pub-hosted-dependency-without-version-by-inserting-version.yaml"), None),
        Some("update"),
        Some("hosted_dep"),
        &[RegistryResponseInput::new("hosted_alias".to_owned(), Pub, r#"{"latest":{"version":"2.0.0"}}"#.to_owned())],
    );

    assert_single_edit(&output, "\n    version: 2.0.0");
}

#[test]
fn apply_command_updates_gleam_project_version_by_requested_level() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///gleam.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "command-updates-gleam-project-version-by-requested-level.toml",
            ), None),
        Some("updateMinor"),
        Some("my_package"),
        &[],
    );

    assert_project_update(&output, "my_package", "1.3.0");
}

#[test]
fn apply_command_updates_gleam_dependency_preserving_requirement_syntax() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///gleam.toml".to_owned(), "toml".to_owned(), package_file_fixture(
                "command-updates-gleam-dependency-preserving-requirement-syntax.toml",
            ), None),
        Some("update"),
        Some("gleam_stdlib"),
        &[RegistryResponseInput::new("gleam_stdlib".to_owned(), Hex, r#"{"releases":[{"version":"2.0.0"},{"version":"0.44.0"}]}"#.to_owned())],
    );

    assert_single_edit(&output, "2.0.0");
}

#[test]
fn apply_command_updates_rebar_dependency_version() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///rebar.config".to_owned(), "erlang".to_owned(), package_file_fixture("command-updates-rebar-dependency-version.config"), None),
        command: Some("update"),
        dependency_name: Some("cowboy"),
        selected_version: Some("2.13.0"),
        responses: &[RegistryResponseInput::new("cowboy".to_owned(), Hex, r#"{"releases":[{"version":"2.13.0"},{"version":"2.12.0"}]}"#.to_owned())],
    });
    assert_single_edit(&output, "2.13.0");
}

#[test]
fn apply_command_updates_opam_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///demo.opam".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-opam-dependency-preserving-operator.opam",
            ), None),
        command: Some("update"),
        dependency_name: Some("lwt"),
        selected_version: Some("6.1.2"),
        responses: &[RegistryResponseInput::new("lwt".to_owned(), Opam, r#"<h2>lwt version</h2><p>6.1.2 (latest)</p>"#.to_owned())],
    });
    assert_single_edit(&output, ">= \"6.1.2\"");
}

#[test]
fn apply_command_updates_dune_project_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///dune-project".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-dune-project-dependency-preserving-operator.dune-project",
            ), None),
        command: Some("update"),
        dependency_name: Some("fmt"),
        selected_version: Some("0.9.0"),
        responses: &[RegistryResponseInput::new("fmt".to_owned(), Opam, r#"<h2>fmt version</h2><p>0.9.0 (latest)</p>"#.to_owned())],
    });
    assert_single_edit(&output, ">= 0.9.0");
}

#[test]
fn apply_command_updates_cabal_dependency_preserving_operator() {
    let session = standard_session();

    let output = session.apply_command_with_selected_version(ApplyCommandRequest {
        input: DocumentInput::new("file:///demo.cabal".to_owned(), "plaintext".to_owned(), package_file_fixture(
                "command-updates-cabal-dependency-preserving-operator.cabal",
            ), None),
        command: Some("update"),
        dependency_name: Some("base"),
        selected_version: Some("4.20.0.0"),
        responses: &[RegistryResponseInput::new("base".to_owned(), Hackage, r#"{"4.20.0.0":"normal","4.19.2.0":"normal"}"#.to_owned())],
    });
    assert_single_edit(&output, ">= 4.20.0.0");
}
