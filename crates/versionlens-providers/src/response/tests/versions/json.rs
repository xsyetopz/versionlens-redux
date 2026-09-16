#[test]
#[expect(
    clippy::too_many_lines,
    reason = "table-driven manifest coverage stays readable as one scenario"
)]
fn reads_latest_versions_from_json_registry_responses() {
    for (ecosystem, package, body, expected) in [
        (
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"1.0.228"}}"#,
            "1.0.228",
        ),
        (
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"1.0.229"},"versions":[{"num":"1.0.229","yanked":true},{"num":"1.0.228","yanked":false}]}"#,
            "1.0.228",
        ),
        (
            Npm,
            "typescript",
            r#"{"dist-tags":{"latest":"6.0.3"}}"#,
            "6.0.3",
        ),
        (
            Npm,
            "octokit/core.js",
            r#"[{"name":"v2.5.0"},{"name":"v2.0.0"}]"#,
            "v2.5.0",
        ),
        (
            Npm,
            "owner/commit",
            r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
            "abcdef1",
        ),
        (Npm, "owner/short-commit", r#"[{"sha":"123"}]"#, "123"),
        (
            Deno,
            "@std/assert",
            r#"{"versions":{"1.0.0":{"yanked":true},"1.1.0":{},"1.2.0-rc.1":{},"1.0.3":{}}}"#,
            "1.1.0",
        ),
        (
            Deno,
            "@std/assert",
            r#"{"latest":"1.3.0","versions":{"1.2.0":{},"1.3.0":{}}}"#,
            "1.3.0",
        ),
        (
            Dotnet,
            "Newtonsoft.Json",
            r#"{"versions":["13.0.1","14.0.0-beta.1","13.0.3"]}"#,
            "13.0.3",
        ),
        (
            Docker,
            "node",
            r#"{"results":[{"name":"20","tag_status":"active","digest":"sha256-20"},{"name":"22-beta","tag_status":"active","digest":"sha256-22"},{"name":"18","tag_status":"inactive","digest":"sha256-18"}]}"#,
            "20",
        ),
        (Python, "flask", r#"{"info":{"version":"3.0.0"}}"#, "3.0.0"),
        (
            Python,
            "flask",
            r#"{"info":{"version":"3.1.0"},"releases":{"3.0.0":[{"yanked":false}],"3.1.0":[{"yanked":true}]}}"#,
            "3.0.0",
        ),
        (
            Pub,
            "http",
            r#"{"versions":[{"version":"1.0.0"},{"version":"1.2.0","retracted":true},{"version":"1.1.0"},{"version":"2.0.0-dev.1"}]}"#,
            "1.1.0",
        ),
        (
            Ruby,
            "rails",
            r#"[{"number":"8.0.0"},{"number":"8.1.0.beta1"},{"number":"8.0.4"}]"#,
            "8.0.4",
        ),
        (
            Ruby,
            "rspec/rspec-rails",
            r#"[{"name":"v6.1.0"},{"name":"v6.0.1"}]"#,
            "v6.1.0",
        ),
        (
            Ruby,
            "rspec/rspec-core",
            r#"[{"sha":"abcdef1234567890"},{"sha":"1234567890abcdef"}]"#,
            "abcdef1",
        ),
        (
            Vcpkg,
            "fmt",
            r#"{"versions":[{"version":"11.1.4","git-tree":"a"},{"version":"11.2.0-rc.1","git-tree":"b"},{"version":"10.2.1#1","git-tree":"c"}]}"#,
            "11.1.4",
        ),
        (
            Terraform,
            "hashicorp/aws",
            r#"{"versions":[{"version":"5.0.0"},{"version":"5.1.0-beta.1"},{"version":"4.67.0"}]}"#,
            "5.0.0",
        ),
        (
            AnsibleGalaxy,
            "community.general",
            r#"{"data":[{"version":"8.0.0-beta.1"},{"version":"7.5.0"}]}"#,
            "7.5.0",
        ),
        (
            Bazel,
            "rules_cc",
            r#"{"versions":["0.0.9","0.0.10-rc1","0.0.10"],"yanked_versions":{"0.0.10":"bad release"}}"#,
            "0.0.9",
        ),
        (
            Nix,
            "NixOS/nixpkgs",
            r#"[{"name":"nixos-24.05"},{"name":"nixos-unstable"},{"name":"23.11"}]"#,
            "24.05",
        ),
        (
            CocoaPods,
            "AFNetworking",
            r#"{"versions":[{"name":"4.0.1"},{"name":"4.0.0-beta.1"},{"name":"3.2.1"}]}"#,
            "4.0.1",
        ),
    ] {
        assert_latest(ecosystem, package, body, expected);
    }
}

#[test]
fn reads_full_crates_io_response_without_deserializing_unneeded_fields() {
    assert_latest(
        Cargo,
        "serde",
        r#"{
            "crate": {
                "id": "serde",
                "name": "serde",
                "downloads": 900000000,
                "max_version": "2.0.0-alpha.1",
                "max_stable_version": "1.0.228",
                "description": "serialization",
                "repository": "https://github.com/serde-rs/serde"
            },
            "versions": [
                {"id": 1, "crate": "serde", "num": "1.0.229", "downloads": 1, "yanked": true, "features": {}},
                {"id": 2, "crate": "serde", "num": "1.0.228", "downloads": 1, "yanked": false, "features": {}}
            ],
            "keywords": [{"id": "serialization"}],
            "categories": [{"id": "encoding"}]
        }"#,
        "1.0.228",
    );
}

#[test]
fn reads_abbreviated_npm_metadata_without_materializing_version_payloads() {
    assert_latest(
        Npm,
        "react",
        r#"{
            "dist-tags":{"latest":"19.1.1"},
            "versions":{
                "18.3.1":{"dependencies":{"loose-envify":"^1.1.0"}},
                "19.1.1":{"dist":{"tarball":"https://registry.example/react.tgz"}}
            },
            "readme":"a decoy \"dist-tags\" and \"versions\""
        }"#,
        "19.1.1",
    );
}

#[test]
fn rejects_malformed_cargo_responses() {
    for body in [
        "not json",
        r#"{"versions":"1.0.0"}"#,
        r#"{"versions":[{"num":1,"yanked":false}]}"#,
    ] {
        assert_eq!(latest_version_from_response(Cargo, "serde", body), None);
    }
}

#[test]
fn reads_only_a_valid_included_crates_io_default_version() {
    assert_latest(
        Cargo,
        "serde",
        r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[{"num":"1.0.228","yanked":false}]}"#,
        "1.0.228",
    );
    for body in [
        r#"{"crate":{"default_version":"2.0.0-alpha.1","max_version":"0.0.0"},"versions":[{"num":"2.0.0-alpha.1","yanked":false}]}"#,
        r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[{"num":"1.0.228","yanked":true}]}"#,
        r#"{"crate":{"default_version":null,"max_version":"0.0.0","yanked":true},"versions":[]}"#,
        r#"{"crate":{"default_version":"1.0.228","max_version":"0.0.0"},"versions":[]}"#,
    ] {
        assert_eq!(latest_version_from_response(Cargo, "serde", body), None);
    }
}

#[test]
fn reads_cargo_prerelease_pages_and_metadata() {
    let body = r#"{
        "versions":[
            {"num":"2.0.0-rc.1","yanked":true},
            {"num":"2.0.0-beta.2","yanked":false},
            {"num":"1.9.0","yanked":false}
        ],
        "meta":{"next_page":"?per_page=100&seek=next"}
    }"#;
    assert_eq!(
        latest_version_with_tags(Cargo, "serde", body, &["beta".to_owned()]),
        Some("2.0.0-beta.2".to_owned())
    );
    assert_eq!(
        latest_version_with_tags(Cargo, "serde", body, &["rc".to_owned()]),
        Some("1.9.0".to_owned())
    );
}
