use super::{
    build_versions_from_response, docker_tag_exists, latest_version_for_requirement,
    latest_version_from_response, latest_version_from_response_with_prereleases,
};
use versionlens_model::Ecosystem::Docker;

#[test]
fn ignores_docker_hub_tags_without_digests() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"24","tag_status":"active"},{"name":"22","tag_status":"active","digest":"sha256-22"}]}"#,
        ),
        Some("22".to_owned())
    );
}

#[test]
fn ignores_docker_hub_results_without_active_tag_status() {
    let body = r#"{"results":[{"name":"2.0.0","digest":"sha256-2"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#;

    assert_eq!(
        latest_version_from_response(Docker, "node", body),
        Some("1.0.0".to_owned())
    );
    assert_eq!(docker_tag_exists(body, "2.0.0"), Some(false));
}

#[test]
fn detects_docker_tag_existence() {
    let body = r#"{"results":[{"name":"2.0.0","tag_status":"active","digest":"sha256-2"},{"name":"old","tag_status":"inactive","digest":"sha256-old"},{"name":"broken","tag_status":"active"}]}"#;

    assert_eq!(docker_tag_exists(body, "2.0.0"), Some(true));
    assert_eq!(docker_tag_exists(body, "3"), Some(false));
    assert_eq!(docker_tag_exists(body, "old"), Some(false));
    assert_eq!(docker_tag_exists(body, "broken"), Some(false));
}

#[test]
fn reads_docker_build_versions_from_same_digest_aliases() {
    assert_eq!(
        build_versions_from_response(
            Docker,
            r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"}]}"#,
            "23.11.0",
        ),
        [
            "latest".to_owned(),
            "23".to_owned(),
            "23-bookworm".to_owned(),
            "23.11".to_owned(),
            "23.11-bookworm".to_owned(),
            "23.11.0".to_owned(),
            "23.11.0-bookworm".to_owned(),
            "bookworm".to_owned(),
            "current".to_owned(),
            "current-bookworm".to_owned(),
        ]
    );
}

#[test]
fn reads_docker_build_versions_for_empty_requirement_from_latest_alias() {
    assert_eq!(
        build_versions_from_response(
            Docker,
            r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"current-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"current","tag_status":"active","digest":"sha256-23"},{"name":"bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"23.11-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23.11","tag_status":"active","digest":"sha256-23"},{"name":"23-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"22.4.3","tag_status":"active","digest":"sha256-22"}]}"#,
            "",
        ),
        [
            "latest".to_owned(),
            "23".to_owned(),
            "23-bookworm".to_owned(),
            "23.11".to_owned(),
            "23.11-bookworm".to_owned(),
            "23.11.0".to_owned(),
            "23.11.0-bookworm".to_owned(),
            "bookworm".to_owned(),
            "current".to_owned(),
            "current-bookworm".to_owned(),
        ]
    );
}

#[test]
fn reads_docker_latest_build_family_aliases() {
    assert_eq!(
        build_versions_from_response(
            Docker,
            r#"{"results":[{"name":"2022-RTM-CU2-ubuntu-20.04","tag_status":"active","digest":"sha256-a"},{"name":"2022-RTM-GDR1-ubuntu-20.04","tag_status":"active","digest":"sha256-b"},{"name":"2022-RTM-ubuntu-20.04","tag_status":"active","digest":"sha256-c"},{"name":"2022-latest","tag_status":"active","digest":"sha256-latest"},{"name":"2022-preview-ubuntu-22.04","tag_status":"active","digest":"sha256-d"},{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"latest-ubuntu","tag_status":"active","digest":"sha256-e"}]}"#,
            "latest",
        ),
        [
            "latest".to_owned(),
            "2022-RTM-CU2-ubuntu-20.04".to_owned(),
            "2022-RTM-GDR1-ubuntu-20.04".to_owned(),
            "2022-RTM-ubuntu-20.04".to_owned(),
            "2022-latest".to_owned(),
            "2022-preview-ubuntu-22.04".to_owned(),
        ]
    );
}

#[test]
fn does_not_expand_docker_latest_builds_to_older_numeric_family_tags() {
    assert_eq!(
        build_versions_from_response(
            Docker,
            r#"{"results":[{"name":"23.10.0","tag_status":"active","digest":"sha256-old"},{"name":"23.11.0","tag_status":"active","digest":"sha256-latest"},{"name":"latest","tag_status":"active","digest":"sha256-latest"}]}"#,
            "latest",
        ),
        ["latest".to_owned(), "23.11.0".to_owned()]
    );
}

#[test]
fn orphan_docker_hub_latest_tag_falls_back_to_highest_versioned_tag() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-22"},{"name":"24","tag_status":"active","digest":"sha256-24"}]}"#,
        ),
        Some("24".to_owned())
    );
}

#[test]
fn docker_latest_alias_without_version_digest_uses_highest_versioned_tag() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-latest"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#
        ),
        Some("1.0.0".to_owned())
    );
}

#[test]
fn docker_ignores_five_digit_numeric_tags_like_upstream() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"20240501","tag_status":"active","digest":"sha256-date"},{"name":"1.0.0","tag_status":"active","digest":"sha256-1"}]}"#
        ),
        Some("1.0.0".to_owned())
    );
}

#[test]
fn ignores_docker_four_segment_numeric_tags_like_upstream() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"1.2.3.4","tag_status":"active","digest":"sha256-four"},{"name":"1.2.3","tag_status":"active","digest":"sha256-three"}]}"#
        ),
        Some("1.2.3".to_owned())
    );
}

#[test]
fn normalizes_docker_numeric_tags_with_leading_zeroes_like_upstream() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"01.02.003","tag_status":"active","digest":"sha256-leading"}]}"#
        ),
        Some("1.2.3".to_owned())
    );
}

#[test]
fn reads_docker_numeric_range_alias_builds_across_digests() {
    assert_eq!(
        build_versions_from_response(
            Docker,
            r#"{"results":[{"name":"1","tag_status":"active","digest":"sha256-major"},{"name":"1.0","tag_status":"active","digest":"sha256-minor"},{"name":"1.0.0","tag_status":"active","digest":"sha256-fixed"}]}"#,
            "1.0.0",
        ),
        ["1".to_owned(), "1.0".to_owned(), "1.0.0".to_owned()]
    );
}

#[test]
fn resolves_docker_latest_tag_to_semver_alias() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"latest","tag_status":"active","digest":"sha256-23"},{"name":"23","tag_status":"active","digest":"sha256-23"},{"name":"23.11.0","tag_status":"active","digest":"sha256-23"},{"name":"22","tag_status":"active","digest":"sha256-22"}]}"#
        ),
        Some("23.11.0".to_owned())
    );
}

#[test]
fn reads_docker_build_suffix_tags() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "node",
            r#"{"results":[{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"},{"name":"24.1.0-bookworm","tag_status":"active","digest":"sha256-24"}]}"#,
        ),
        Some("24.1.0-bookworm".to_owned())
    );
}

#[test]
fn keeps_docker_build_suffix_when_matching_tag_exists() {
    assert_eq!(
        latest_version_for_requirement(
            Docker,
            "node",
            "20.19.1-bookworm",
            r#"{"results":[{"name":"25.0.0-alpine","tag_status":"active","digest":"sha256-25"},{"name":"24.1.0-bookworm","tag_status":"active","digest":"sha256-24"},{"name":"23.11.0-bookworm","tag_status":"active","digest":"sha256-23"}]}"#,
        ),
        Some("24.1.0-bookworm".to_owned())
    );
}

#[test]
fn filters_docker_dotted_prerelease_tags() {
    let body = r#"{"results":[{"name":"24.1.0","tag_status":"active","digest":"sha256-24"},{"name":"25.0.0-rc.1","tag_status":"active","digest":"sha256-25"}]}"#;

    assert_eq!(
        latest_version_from_response(Docker, "node", body),
        Some("24.1.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(Docker, "node", body, true),
        Some("25.0.0-rc.1".to_owned())
    );
}

#[test]
fn reads_microsoft_container_registry_tags() {
    assert_eq!(
        latest_version_from_response(
            Docker,
            "mcr.microsoft.com/dotnet/sdk",
            r#"[{"name":"8.0","digest":"sha256-8"},{"name":"9.0-preview","digest":"sha256-9"},{"name":"10.0","digest":"sha256-10"}]"#
        ),
        Some("10.0".to_owned())
    );
}

#[test]
fn reads_docker_registry_v2_tag_lists() {
    let body = r#"{"name":"org/app","tags":["1.0.0","1.4.0","2.0.0-rc.1"]}"#;

    assert_eq!(
        latest_version_from_response(Docker, "ghcr.io/org/app", body),
        Some("1.4.0".to_owned())
    );
    assert_eq!(docker_tag_exists(body, "1.4.0"), Some(true));
    assert_eq!(docker_tag_exists(body, "3.0.0"), Some(false));
}
