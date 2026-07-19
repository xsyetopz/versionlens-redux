use versionlens_model::Ecosystem::{Dub, Haxelib, Helm};
#[test]
fn reads_npm_build_versions_in_upstream_compare_build_order() {
    assert_eq!(
        npm_build_versions(
            r#"{"versions":{"1.0.0+build.10":{},"1.0.0+build.2":{},"1.0.0":{},"1.1.0+build.1":{}}}"#,
            "1.0.0+build.2",
        ),
        [
            "1.0.0".to_owned(),
            "1.0.0+build.2".to_owned(),
            "1.0.0+build.10".to_owned()
        ]
    );
}

#[test]
fn pub_latest_field_is_used_when_version_list_has_no_allowed_release() {
    assert_eq!(
        latest_version_from_response(
            Pub,
            "http",
            r#"{"latest":{"version":"1.2.3"},"versions":[]}"#,
        ),
        Some("1.2.3".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Pub,
            "http",
            r#"{"latest":{"version":"1.2.3"},"versions":[{"version":"2.0.0-dev.1"}]}"#,
        ),
        Some("1.2.3".to_owned())
    );
}

#[test]
fn reads_pub_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(
            Pub,
            "http",
            r#"{"versions":["0.1.0","0.1.1","0.1.2","1.0.0-dev.1"]}"#,
        ),
        Some("0.1.2".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Pub,
            "http",
            r#"{"versions":["0.1.0","0.1.1","0.1.2","1.0.0-dev.1"]}"#,
            true,
        ),
        Some("1.0.0-dev.1".to_owned())
    );
}

#[test]
fn cargo_crate_metadata_prefers_max_stable_version() {
    assert_eq!(
        latest_version_from_response(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"}}"#,
        ),
        Some("1.0.228".to_owned())
    );
}

#[test]
fn cargo_crate_metadata_is_used_when_version_list_has_no_allowed_release() {
    assert_eq!(
        latest_version_from_response(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.1","yanked":false},{"num":"1.0.229","yanked":true}]}"#,
        ),
        Some("1.0.228".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.2","yanked":true}]}"#,
            true,
        ),
        Some("2.0.0-alpha.1".to_owned())
    );
}

#[test]
fn cargo_crate_metadata_uses_max_version_when_prereleases_are_visible() {
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"2.0.0-alpha.1","max_stable_version":"1.0.228"}}"#,
            true,
        ),
        Some("2.0.0-alpha.1".to_owned())
    );
}

#[test]
fn deno_latest_field_falls_back_when_yanked_or_prerelease() {
    assert_eq!(
        latest_version_from_response(
            Deno,
            "@std/assert",
            r#"{"latest":"1.3.0","versions":{"1.2.0":{},"1.3.0":{"yanked":true}}}"#,
        ),
        Some("1.2.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Deno,
            "@std/assert",
            r#"{"latest":"1.3.0-rc.1","versions":{"1.2.0":{},"1.3.0-rc.1":{}}}"#,
        ),
        Some("1.2.0".to_owned())
    );
}

#[test]
fn reads_latest_versions_from_dub_registry_responses() {
    assert_latest(
        Dub,
        "vibe-d",
        r#"{"versions":[{"version":"~master"},{"version":"1.0.0"},{"version":"1.1.0-beta.1"},{"version":"1.0.2"}]}"#,
        "1.0.2",
    );
    assert_eq!(
        latest_version_for_requirement(
            Dub,
            "vibe-d",
            "~master",
            r#"{"versions":[{"version":"~master"},{"version":"0.9.0"},{"version":"0.8.0"}]}"#,
        )
        .as_deref(),
        Some("~master")
    );
    assert_latest(
        Dub,
        "vibe-d",
        r#"["~master","1.0.0","1.1.0-beta.1","1.0.2"]"#,
        "1.0.2",
    );
    assert_eq!(
        latest_version_for_requirement(Dub, "vibe-d", "~master", r#"["~master","0.9.0","0.8.0"]"#,)
            .as_deref(),
        Some("~master")
    );
    assert_latest(
        Dub,
        "vibe-d",
        r#"{"versions":[{"version":"~main"}]}"#,
        "~main",
    );
}

#[test]
fn reads_latest_versions_from_text_registry_responses() {
    assert_latest(Go, "go.uber.org/zap", "v1.0.0\nv1.2.0\n", "v1.2.0");
    assert_latest(
        Go,
        "go.uber.org/zap",
        "\nv0.32.3\nv0.19.10\n\nv0.26.0\n",
        "v0.32.3",
    );
    assert_latest(
        Go,
        "github.com/docker/cli",
        "v26.1.3+incompatible\nv27.0.0+incompatible\n",
        "v27.0.0",
    );
    assert_latest(
        Maven,
        "org.springframework:spring-core",
        r#"<metadata><versioning><versions><version>5.0.0</version><version>6.0.0-M1</version><version>5.3.1</version></versions></versioning></metadata>"#,
        "5.3.1",
    );
    assert_latest(
        Helm,
        "apache",
        "apiVersion: v1\nentries:\n  apache:\n    - version: 11.4.1\n    - version: 12.0.0-beta.1\n    - version: 11.3.0\n  mysql:\n    - version: 9.0.0\n",
        "11.4.1",
    );
    assert_eq!(
        latest_version_from_response(
            Maven,
            "org.springframework:spring-core",
            r#"<metadata><versioning><release>6.1.1</release><latest>6.2.0-M1</latest></versioning></metadata>"#,
        ),
        None
    );
    assert_eq!(
        latest_version_from_response(
            Maven,
            "org.springframework:spring-core",
            r#"<metadata><versioning><latest>6.1.2</latest></versioning></metadata>"#,
        ),
        None
    );
    assert_latest(
        Python,
        "flask",
        r#"<rss><channel><item><title>Flask 2.3.3</title></item><item><title>Flask 3.0.0</title></item></channel></rss>"#,
        "3.0.0",
    );
    assert_latest(
        Python,
        "pip",
        r#"<rss><channel><item><title>25.0.1</title></item><item><title>24.3.1</title></item></channel></rss>"#,
        "25.0.1",
    );
    assert_latest(
        Haxelib,
        "tink_core",
        r#"<main><code>haxelib install tink_core 2.0.0</code><code>haxelib install tink_core 2.1.0-rc.1</code><code>haxelib install tink_core 1.9.0</code></main>"#,
        "2.0.0",
    );
}

#[test]
fn python_rss_two_segment_latest_is_coerced_to_three_segments() {
    assert_eq!(
        latest_version_from_response(
            Python,
            "pip",
            r#"<rss><channel><item><title>25.0</title></item><item><title>24.3</title></item></channel></rss>"#,
        ),
        Some("25.0.0".to_owned())
    );
}

#[test]
fn ignores_invalid_python_rss_responses() {
    assert_eq!(latest_version_from_response(Python, "pip", "<rss>"), None);
}

#[test]
fn falls_back_when_registry_latest_tag_is_prerelease() {
    assert_eq!(
        latest_version_from_response(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"2.0.0-alpha.1"},"versions":[{"num":"1.0.228","yanked":false},{"num":"2.0.0-alpha.1","yanked":false}]}"#,
        ),
        Some("1.0.228".to_owned())
    );
    assert_eq!(
        latest_version_from_response(
            Python,
            "flask",
            r#"{"info":{"version":"4.0.0rc1"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#,
        ),
        Some("3.0.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response(Pub, "http", r#"{"latest":{"version":"2.0.0-dev.1"}}"#,),
        None
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Pub,
            "http",
            r#"{"latest":{"version":"2.0.0-dev.1"}}"#,
            true,
        ),
        Some("2.0.0-dev.1".to_owned())
    );
}

#[test]
fn npm_latest_dist_tag_uses_prerelease_tagged_version() {
    assert_eq!(
        latest_version_from_response(
            Npm,
            "typescript",
            r#"{"dist-tags":{"latest":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
        ),
        Some("7.0.0-beta.1".to_owned())
    );
}

#[test]
fn can_pick_prerelease_versions() {
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Cargo,
            "serde",
            r#"{"crate":{"max_version":"1.0.228"},"versions":[{"num":"2.0.0-alpha.1","yanked":true},{"num":"1.1.0-beta.1","yanked":false},{"num":"1.0.228","yanked":false}]}"#,
            true,
        ),
        Some("1.1.0-beta.1".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Dotnet,
            "Newtonsoft.Json",
            r#"{"versions":["13.0.1","14.0.0-beta.1","13.0.3"]}"#,
            true,
        ),
        Some("14.0.0-beta.1".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Npm,
            "typescript",
            r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
            true,
        ),
        Some("6.0.3".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Python,
            "flask",
            r#"{"info":{"version":"3.0.0"},"releases":{"3.0.0":[],"4.0.0rc1":[]}}"#,
            true,
        ),
        Some("4.0.0rc1".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Ruby,
            "rails",
            r#"[{"number":"8.0.4"},{"number":"8.1.0.beta1"}]"#,
            true,
        ),
        Some("8.1.0.beta1".to_owned())
    );
}

#[test]
fn filters_prerelease_tags_from_registry_responses() {
    assert_eq!(
        latest_version_with_tags(
            Npm,
            "typescript",
            r#"{"dist-tags":{"latest":"6.0.3"},"versions":{"6.0.3":{},"7.0.0-beta.1":{},"8.0.0-rc.1":{}}}"#,
            &["beta".to_owned()],
        ),
        Some("6.0.3".to_owned())
    );
    assert_eq!(
        latest_version_with_tags(
            Deno,
            "@std/assert",
            r#"{"versions":{"1.0.0":{},"1.1.0-alpha.1":{},"1.1.0-beta.1":{}}}"#,
            &["alpha".to_owned()],
        ),
        Some("1.1.0-alpha.1".to_owned())
    );
    assert_eq!(
        latest_version_with_tags(
            Docker,
            "node",
            r#"{"results":[{"name":"24.1.0","tag_status":"active","digest":"sha256-24"},{"name":"25.0.0-beta.1","tag_status":"active","digest":"sha256-beta"},{"name":"25.0.0-rc.1","tag_status":"active","digest":"sha256-rc"}]}"#,
            &["beta".to_owned()],
        ),
        Some("25.0.0-beta.1".to_owned())
    );
}
