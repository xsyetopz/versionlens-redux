#[test]
fn reads_python_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(Python, "pip", r#"["25.0.1","25.0","24.3.1","24.3","24.2"]"#,),
        Some("25.0.1".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Python,
            "pip",
            r#"{"versions":["25.0.1","25.0","26.0.0rc1"]}"#,
            true,
        ),
        Some("26.0.0rc1".to_owned())
    );
}

#[test]
fn extracts_python_release_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Python,
            "pip",
            r#"{"versions":["24.3.1","25.0.0","26.0.0rc1"]}"#,
        ),
        [
            "24.3.1".to_owned(),
            "25.0.0".to_owned(),
            "26.0.0-rc.1".to_owned()
        ]
    );
    assert_eq!(
        release_versions_from_response_for_package(
            Python,
            "pip",
            r#"{"info":{"version":"3.0.0"},"releases":{"2.0.0":[{"yanked":true}],"1.0.0":[],"1.1.0":[{"yanked":false}],"1.0":[{"yanked":false}],"1.1.0rc1":[{"yanked":false}]}}"#,
        ),
        [
            "1.0.0".to_owned(),
            "1.1.0-rc.1".to_owned(),
            "1.1.0".to_owned()
        ]
    );
    assert_eq!(
        release_versions_from_response_for_package(
            Python,
            "demo",
            r#"<?xml version="1.0"?><rss><channel><item><title>Demo 1.0.0</title></item><item><title>Demo 1.1.0</title></item><item><title>Demo 1.1.0</title></item><item><title>Demo 2.0.0rc1</title></item></channel></rss>"#,
        ),
        [
            "1.0.0".to_owned(),
            "1.1.0".to_owned(),
            "2.0.0-rc.1".to_owned()
        ]
    );
}

#[test]
fn extracts_ruby_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Ruby,
            "rails",
            r#"["1.0.0","1.1.0-pre.1","1.0.1"]"#,
        ),
        [
            "1.0.0".to_owned(),
            "1.0.1".to_owned(),
            "1.1.0-pre.1".to_owned()
        ]
    );
}

#[test]
fn reads_maven_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(
            Maven,
            "junit:junit",
            r#"["4.13-rc-1","4.13-rc-2","4.13","4.13.1","4.13.2"]"#,
        ),
        Some("4.13.2".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Maven,
            "junit:junit",
            r#"["4.13-rc-1","4.13-rc-2","4.13","4.13.1","4.13.2","4.14.0-rc.1"]"#,
            true,
        ),
        Some("4.14.0-rc.1".to_owned())
    );
}

#[test]
fn reads_maven_release_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Maven,
            "junit:junit",
            r#"<metadata><versioning><versions><version>1.0.0</version><version>2.0.0-M1</version><version>1.0.1</version><version>ignored</version></versions></versioning></metadata>"#
        ),
        [
            "1.0.0".to_owned(),
            "1.0.1".to_owned(),
            "2.0.0-M1".to_owned()
        ]
    );
}

#[test]
fn reads_npm_release_versions_in_upstream_compare_build_order() {
    assert_eq!(
        release_versions_from_response_for_package(
            Npm,
            "example",
            r#"{"versions":{"2.0.0":{},"1.0.0+build.10":{},"1.0.0+build.2":{},"1.0.0":{}}}"#,
        ),
        [
            "1.0.0".to_owned(),
            "1.0.0+build.2".to_owned(),
            "1.0.0+build.10".to_owned(),
            "2.0.0".to_owned()
        ]
    );
}

#[test]
fn reads_npm_versions_from_normalized_string_arrays() {
    assert_eq!(
        latest_version_from_response(
            Npm,
            "npm-package-arg",
            r#"{"dist-tags":{"latest":"7.0.0"},"versions":["6.0.0","6.1.0","7.0.0","8.0.0","8.0.1"]}"#,
        ),
        Some("7.0.0".to_owned())
    );
    assert_eq!(
        latest_version_from_response_with_prereleases(
            Npm,
            "pacote",
            r#"{"dist-tags":{"latest":"11.1.9"},"versions":["11.1.9","12.0.0-beta.1"]}"#,
            true,
        ),
        Some("11.1.9".to_owned())
    );
}

#[test]
fn reads_npm_build_versions_for_matching_release() {
    assert_eq!(
        npm_build_versions(
            r#"{"versions":{"1.0.0":{},"1.0.0+build.1":{},"1.0.0+build.2":{},"1.1.0+build.1":{}}}"#,
            "1.0.0+build.1",
        ),
        [
            "1.0.0".to_owned(),
            "1.0.0+build.1".to_owned(),
            "1.0.0+build.2".to_owned()
        ]
    );
}

#[test]
fn reads_npm_build_versions_from_normalized_string_arrays() {
    assert_eq!(
        npm_build_versions(
            r#"{"versions":["1.0.0","1.0.0+build.1","1.0.0+build.2","1.1.0+build.1"]}"#,
            "1.0.0+build.1",
        ),
        [
            "1.0.0".to_owned(),
            "1.0.0+build.1".to_owned(),
            "1.0.0+build.2".to_owned()
        ]
    );
}
