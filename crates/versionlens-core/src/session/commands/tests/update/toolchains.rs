use versionlens_model::Ecosystem::{Cpan, Haxelib, LuaRocks, Nim, Zig};
#[test]
fn apply_command_does_not_update_swift_local_package_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Package.swift".to_owned(),
            language_id: "swift".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-swift-local-package-dependency.swift",
            ),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///build.zig.zon".to_owned(),
            language_id: "zig".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-zig-github-url-tag-dependency.zig.zon",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("known_folders"),
        &[RegistryResponseInput {
            package: "ziglibs/known-folders".to_owned(),
            ecosystem: Zig,
            body: r#"[{"name":"0.8.0"},{"name":"0.7.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "0.8.0");
}

#[test]
fn apply_command_does_not_update_zig_path_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///build.zig.zon".to_owned(),
            language_id: "zig".to_owned(),
            text: package_file_fixture("apply-command-does-not-update-zig-path-dependency.zig.zon"),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///demo.nimble".to_owned(),
            language_id: "nim".to_owned(),
            text: package_file_fixture("apply-command-updates-nimble-github-url-dependency.nimble"),
            workspace_root: None,
        },
        Some("update"),
        Some("pkg"),
        &[RegistryResponseInput {
            package: "user/pkg".to_owned(),
            ecosystem: Nim,
            body: r#"[{"name":"2.1.0"},{"name":"2.0.0"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "== 2.1.0");
}

#[test]
fn apply_command_does_not_update_nimble_head_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///demo.nimble".to_owned(),
            language_id: "nim".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-nimble-head-dependency.nimble",
            ),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///demo-1.0.0-1.rockspec".to_owned(),
            language_id: "lua".to_owned(),
            text: package_file_fixture(
                "apply-command-updates-luarocks-rockspec-dependency.0.0-1.rockspec",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("luasocket"),
        &[RegistryResponseInput {
            package: "luasocket".to_owned(),
            ecosystem: LuaRocks,
            body: r#"repository = {
   ["luasocket"] = {
      ["3.0.0-1"] = { { arch = "rockspec" } },
      ["3.1.0-1"] = { { arch = "src" } }
   }
}"#
            .to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "== 3.1.0-1");
}

#[test]
fn apply_command_does_not_update_luarocks_lua_runtime_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///demo-1.0.0-1.rockspec".to_owned(),
            language_id: "lua".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-luarocks-lua-runtime-dependency.0.0-1.rockspec",
            ),
            workspace_root: None,
        },
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
        DocumentInput {
            uri: "file:///work/cpanfile".to_owned(),
            language_id: "perl".to_owned(),
            text: package_file_fixture("apply-command-updates-cpanfile-dependencycpanfile"),
            workspace_root: None,
        },
        Some("update"),
        Some("Plack"),
        &[RegistryResponseInput {
            package: "Plack".to_owned(),
            ecosystem: Cpan,
            body: r#"{"status":"latest","version":"2.0.0"}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_updates_haxelib_json_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/haxelib.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("apply-command-updates-haxelib-json-dependency.json"),
            workspace_root: None,
        },
        Some("update"),
        Some("tink_core"),
        &[RegistryResponseInput {
            package: "tink_core".to_owned(),
            ecosystem: Haxelib,
            body: r#"<code>haxelib install tink_core 2.0.0</code><code>haxelib install tink_core 1.0.0</code>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.edits.len(), 1);
    assert_eq!(output.edits[0].new_text, "2.0.0");
}

#[test]
fn apply_command_does_not_update_haxelib_latest_dependency() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/haxelib.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "apply-command-does-not-update-haxelib-latest-dependency.json",
            ),
            workspace_root: None,
        },
        Some("update"),
        Some("tink_macro"),
        &[RegistryResponseInput {
            package: "tink_macro".to_owned(),
            ecosystem: Haxelib,
            body: r#"<code>haxelib install tink_macro 2.0.0</code>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert!(output.edits.is_empty());
}
