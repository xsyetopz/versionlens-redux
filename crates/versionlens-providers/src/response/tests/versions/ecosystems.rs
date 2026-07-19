#[test]
fn reads_latest_versions_from_normalized_github_commit_arrays() {
    for ecosystem in [Npm, Ruby] {
        assert_latest(
            ecosystem,
            "owner/string-commit",
            r#"["abcdef1234567890","1234567890abcdef"]"#,
            "abcdef1",
        );
    }
}

#[test]
fn github_tag_responses_ignore_non_semver_names() {
    for ecosystem in [Npm, Ruby] {
        assert_latest(
            ecosystem,
            "owner/repo",
            r#"[{"name":"release-5.6.8"},{"name":"v2.0.0"},{"name":"build-9.0.0"}]"#,
            "v2.0.0",
        );
    }
}

#[test]
fn cpp_github_tag_responses_use_json_before_xmake_text_fallback() {
    assert_latest(
        Cpp,
        "fmtlib/fmt",
        r#"[{"name":"11.1.4"},{"name":"11.2.0-rc.1"},{"name":"10.2.1"}]"#,
        "11.1.4",
    );
    assert_latest(
        Cpp,
        "openssl",
        r#"add_versions("3.0.0", "sha256") add_versions("3.1.0-beta", "sha256")"#,
        "3.0.0",
    );
}

#[test]
fn reads_ruby_versions_from_normalized_string_arrays() {
    assert_latest(Ruby, "rails", r#"["1.0.0","1.1.0","2.0.0-alpha"]"#, "1.1.0");
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Ruby,
            "rails",
            r#"["1.0.0","1.1.0","2.0.0-alpha"]"#,
            true,
        ),
        Some("2.0.0-alpha".to_owned())
    );
}

#[test]
fn reads_deno_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(
            Deno,
            "@std/assert",
            r#"["0.215.0","0.212.0","1.0.6","0.198.0","0.196.0","1.1.0-rc.2"]"#,
        ),
        Some("1.0.6".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Deno,
            "@std/assert",
            r#"["0.215.0","0.212.0","1.0.6","0.198.0","0.196.0","1.1.0-rc.2"]"#,
            true,
        ),
        Some("1.1.0-rc.2".to_owned())
    );
}

#[test]
fn reads_cargo_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(
            Cargo,
            "serde",
            r#"{"versions":["1.0.20","1.0.19","1.1.0-beta.1"]}"#,
        ),
        Some("1.0.20".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Cargo,
            "serde",
            r#"["1.0.20","1.0.19","1.1.0-beta.1"]"#,
            true,
        ),
        Some("1.1.0-beta.1".to_owned())
    );
}

#[test]
fn reads_hex_versions_from_package_releases() {
    assert_eq!(
        latest_version_from_response(
            Hex,
            "plug",
            r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
        ),
        Some("1.20.2".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Hex,
            "plug",
            r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
            true,
        ),
        Some("1.21.0-rc.1".to_owned())
    );
}

#[test]
fn ignores_malformed_hex_package_responses() {
    assert_eq!(
        latest_version_from_response(Hex, "plug", r#"{"releases": ["#),
        None
    );
    assert_eq!(
        latest_version_from_response(Hex, "plug", r#"{"package":"plug"}"#),
        None
    );
    assert!(
        release_versions_from_response_for_package(Hex, "plug", r#"{"releases": ["#).is_empty()
    );
}

#[test]
fn extracts_hex_release_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Hex,
            "plug",
            r#"{"releases":[{"version":"1.20.2"},{"version":"1.21.0-rc.1"},{"version":"1.19.4"}]}"#,
        ),
        [
            "1.19.4".to_owned(),
            "1.20.2".to_owned(),
            "1.21.0-rc.1".to_owned()
        ]
    );
}

#[test]
fn reads_latest_opam_version_from_package_page() {
    assert_eq!(
        latest_version_from_response(Opam, "lwt", r#"<h2>lwt version</h2><p>6.1.2 (latest)</p>"#,),
        Some("6.1.2".to_owned())
    );
}

#[test]
fn reads_latest_hackage_version_from_package_versions() {
    assert_eq!(
        latest_version_from_response(
            Hackage,
            "aeson",
            r#"{"2.1.2.1":"normal","2.2.3.0":"normal","2.2.4.0":"deprecated","2.3.0.0-rc1":"normal"}"#,
        ),
        Some("2.2.3.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Hackage,
            "aeson",
            r#"{"2.1.2.1":"normal","2.2.3.0":"normal","2.3.0.0-rc1":"normal"}"#,
            true,
        ),
        Some("2.3.0.0-rc1".to_owned())
    );
}

#[test]
fn reads_latest_stackage_lts_snapshot_from_snapshot_index() {
    assert_eq!(
        latest_version_from_response(
            Hackage,
            "stackage-lts",
            r#"{"snapshots":[[["nightly-2026-07-03","Stackage Nightly 2026-07-03 (ghc-9.12.4)","2 days ago"]],[["lts-24.48","LTS Haskell 24.48 (ghc-9.10.3)","5 days ago"],["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#,
        ),
        Some("24.49".to_owned())
    );
}

#[test]
fn reads_latest_stackage_nightly_snapshot_from_snapshot_index() {
    assert_eq!(
        latest_version_from_response(
            Hackage,
            "stackage-nightly",
            r#"{"snapshots":[[["nightly-2026-07-02","Stackage Nightly 2026-07-02 (ghc-9.12.4)","3 days ago"]],[["nightly-2026-07-03","Stackage Nightly 2026-07-03 (ghc-9.12.4)","2 days ago"]],[["lts-24.49","LTS Haskell 24.49 (ghc-9.10.3)","a day ago"]]],"totalCount":3792}"#,
        ),
        Some("2026-07-03".to_owned())
    );
}

#[test]
fn reads_latest_conan_version_from_search_results() {
    assert_eq!(
        latest_version_from_response(
            Conan,
            "zlib",
            r#"{"results":["zlib/1.2.13","zlib/1.3.1","zlib/1.3.0#recipe"]}"#,
        ),
        Some("1.3.1".to_owned())
    );
}

#[test]
fn reads_latest_vcpkg_version_from_versions_database_entry() {
    assert_eq!(
        latest_version_from_response(
            Vcpkg,
            "fmt",
            r#"{"versions":[{"version":"11.1.4","git-tree":"a"},{"version":"11.2.0-rc.1","git-tree":"b"},{"version":"10.2.1#1","git-tree":"c"}]}"#,
        ),
        Some("11.1.4".to_owned())
    );
}

#[test]
fn reads_latest_swift_versions_from_registry_and_github_responses() {
    assert_eq!(
        latest_version_from_response(
            Swift,
            "mona.LinkedList",
            r#"{"releases":{"1.1.0":{"url":"https://packages.example.com/mona/LinkedList/1.1.0"},"1.2.0-beta.1":{"url":"https://packages.example.com/mona/LinkedList/1.2.0-beta.1"},"1.0.0":{"problem":{"status":410}}}}"#,
        ),
        Some("1.1.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Swift,
            "apple/swift-nio",
            r#"[{"name":"2.66.0"},{"name":"2.67.0-alpha.1"},{"name":"2.65.0"}]"#,
        ),
        Some("2.66.0".to_owned())
    );
}

#[test]
fn reads_latest_zig_version_from_github_tags() {
    assert_eq!(
        latest_version_from_response(
            Zig,
            "ziglibs/known-folders",
            r#"[{"name":"0.8.0"},{"name":"0.9.0-dev.1"},{"name":"0.7.0"}]"#,
        ),
        Some("0.8.0".to_owned())
    );
}

#[test]
fn reads_latest_nim_version_from_package_list_or_github_tags() {
    assert_eq!(
        latest_version_from_response(
            Nim,
            "jester",
            r#"[{"name":"jester","url":"https://github.com/dom96/jester","versions":["0.5.0","0.6.0-rc.1","0.4.3"]},{"name":"other","versions":["9.0.0"]}]"#,
        ),
        Some("0.5.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Nim,
            "user/pkg",
            r#"[{"name":"2.1.0"},{"name":"2.2.0-beta.1"},{"name":"2.0.0"}]"#,
        ),
        Some("2.1.0".to_owned())
    );
}

#[test]
fn reads_latest_luarocks_versions_from_manifest() {
    assert_eq!(
        latest_version_from_response(
            LuaRocks,
            "luasocket",
            r#"repository = {
   ["luasocket"] = {
      ["3.0.0-1"] = { { arch = "rockspec" } },
      ["3.1.0-1"] = { { arch = "src" } },
      ["3.2.0-rc1"] = { { arch = "src" } }
   },
   ["other"] = {
      ["9.0.0-1"] = { { arch = "src" } }
   }
}"#,
        ),
        Some("3.1.0-1".to_owned())
    );
}

#[test]
fn reads_latest_cpan_version_from_metacpan_download_url() {
    assert_eq!(
        latest_version_from_response(
            Cpan,
            "Plack",
            r#"{"status":"latest","version":"1.0054","download_url":"https://cpan.metacpan.org/authors/id/M/MI/MIYAGAWA/Plack-1.0054.tar.gz"}"#,
        ),
        Some("1.0054".to_owned())
    );
}

#[test]
fn extracts_cran_releases_only_for_the_requested_package() {
    let body = "Package: cli\nVersion: 3.6.2\n\nPackage: dplyr\nVersion: 1.1.3\nDepends: R (>= 3.5.0)\n\nPackage: dplyr\nVersion: 1.1.4\n";
    assert_eq!(
        latest_version_from_response(Cran, "dplyr", body),
        Some("1.1.4".to_owned())
    );
    assert_eq!(
        release_versions_from_response_for_package(Cran, "dplyr", body),
        vec!["1.1.3".to_owned(), "1.1.4".to_owned()]
    );
    assert_eq!(
        release_versions_from_response_for_package(Cran, "cli", body),
        vec!["3.6.2".to_owned()]
    );
    assert!(release_versions_from_response(Cran, body).is_empty());
}

#[test]
fn release_versions_compatibility_api_does_not_require_a_package() {
    assert_eq!(
        release_versions_from_response(
            Hex,
            r#"{"releases":[{"version":"1.20.2"},{"version":"1.19.4"}]}"#,
        ),
        ["1.19.4".to_owned(), "1.20.2".to_owned()]
    );
}

#[test]
fn reads_latest_julia_version_from_registry_versions_toml() {
    assert_eq!(
        latest_version_from_response(
            Julia,
            "Example",
            r#"[0.5.3]
git-tree-sha1 = "b4d4"

[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0-rc1]
git-tree-sha1 = "d6f6"
"#,
        ),
        Some("0.5.4".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Julia,
            "Example",
            r#"[0.5.4]
git-tree-sha1 = "c5e5"

[0.6.0-rc1]
git-tree-sha1 = "d6f6"
"#,
            true,
        ),
        Some("0.6.0-rc1".to_owned())
    );
}
