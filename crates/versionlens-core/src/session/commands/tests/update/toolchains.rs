#[test]
fn apply_command_does_not_update_swift_local_package_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///Package.swift".to_owned(), "swift".to_owned(), package_file_fixture(
                "command-does-not-update-swift-local-package-dependency.swift",
            ), None),
        Some("update"),
        Some("LocalPackage"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_zig_github_url_tag_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///build.zig.zon".to_owned(), "zig".to_owned(), package_file_fixture(
                "command-updates-zig-github-url-tag-dependency.zig.zon",
            ), None),
        Some("update"),
        Some("known_folders"),
        &[RegistryResponseInput::new("ziglibs/known-folders".to_owned(), Zig, r#"[{"name":"0.8.0"},{"name":"0.7.0"}]"#.to_owned())],
    );

    assert_single_edit(&output, "0.8.0");
}

#[test]
fn apply_command_does_not_update_zig_path_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///build.zig.zon".to_owned(), "zig".to_owned(), package_file_fixture("command-does-not-update-zig-path-dependency.zig.zon"), None),
        Some("update"),
        Some("local_dep"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_nimble_github_url_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///demo.nimble".to_owned(), "nim".to_owned(), package_file_fixture("command-updates-nimble-github-url-dependency.nimble"), None),
        Some("update"),
        Some("pkg"),
        &[RegistryResponseInput::new("user/pkg".to_owned(), Nim, r#"[{"name":"2.1.0"},{"name":"2.0.0"}]"#.to_owned())],
    );

    assert_single_edit(&output, "== 2.1.0");
}

#[test]
fn apply_command_does_not_update_nimble_head_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///demo.nimble".to_owned(), "nim".to_owned(), package_file_fixture(
                "command-does-not-update-nimble-head-dependency.nimble",
            ), None),
        Some("update"),
        Some("foobar"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_luarocks_rockspec_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///demo-1.0.0-1.rockspec".to_owned(), "lua".to_owned(), package_file_fixture(
                "command-updates-luarocks-rockspec-dependency.0.0-1.rockspec",
            ), None),
        Some("update"),
        Some("luasocket"),
        &[RegistryResponseInput::new("luasocket".to_owned(), LuaRocks, r#"repository = {
   ["luasocket"] = {
      ["3.0.0-1"] = { { arch = "rockspec" } },
      ["3.1.0-1"] = { { arch = "src" } }
   }
}"#
            .to_owned())],
    );

    assert_single_edit(&output, "== 3.1.0-1");
}

#[test]
fn apply_command_does_not_update_luarocks_lua_runtime_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///demo-1.0.0-1.rockspec".to_owned(), "lua".to_owned(), package_file_fixture(
                "command-does-not-update-luarocks-lua-runtime-dependency.0.0-1.rockspec",
            ), None),
        Some("update"),
        Some("lua"),
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_updates_cpanfile_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/cpanfile".to_owned(), "perl".to_owned(), package_file_fixture("command-updates-cpanfile-dependencycpanfile"), None),
        Some("update"),
        Some("Plack"),
        &[RegistryResponseInput::new("Plack".to_owned(), Cpan, r#"{"status":"latest","version":"2.0.0"}"#.to_owned())],
    );

    assert_single_edit(&output, "2.0.0");
}

#[test]
fn apply_command_updates_haxelib_json_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/haxelib.json".to_owned(), "json".to_owned(), package_file_fixture("command-updates-haxelib-json-dependency.json"), None),
        Some("update"),
        Some("tink_core"),
        &[RegistryResponseInput::new("tink_core".to_owned(), Haxelib, r#"<code>haxelib install tink_core 2.0.0</code><code>haxelib install tink_core 1.0.0</code>"#.to_owned())],
    );

    assert_single_edit(&output, "2.0.0");
}

#[test]
fn apply_command_does_not_update_haxelib_latest_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput::new("file:///work/haxelib.json".to_owned(), "json".to_owned(), package_file_fixture(
                "command-does-not-update-haxelib-latest-dependency.json",
            ), None),
        Some("update"),
        Some("tink_macro"),
        &[RegistryResponseInput::new("tink_macro".to_owned(), Haxelib, r#"<code>haxelib install tink_macro 2.0.0</code>"#.to_owned())],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}
