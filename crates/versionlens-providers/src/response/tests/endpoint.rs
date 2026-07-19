use crate::{
    LatestVersionRequest, RegistryEndpoint, RegistryResponseKind,
    latest_version_from_response_for_endpoint, release_versions_from_response_for_endpoint,
};
use versionlens_model::Ecosystem::{Go, Python};

fn latest(endpoint: &RegistryEndpoint, package: &str, body: &str) -> Option<String> {
    latest_version_from_response_for_endpoint(
        endpoint,
        LatestVersionRequest {
            ecosystem: match endpoint.response_kind {
                RegistryResponseKind::GoModuleList | RegistryResponseKind::GoModuleLatest => Go,
                _ => Python,
            },
            package,
            requirement: "",
            body,
            include_prereleases: false,
            prerelease_tags: &[],
        },
    )
}

#[test]
fn python_simple_html_responses_use_project_distribution_files() {
    let endpoint = RegistryEndpoint::new(
        "https://packages.test/simple/my-package/".to_owned(),
        RegistryResponseKind::PythonSimple,
    );
    let body = r#"<!doctype html><html><body>
      <a href="../../files/My_Package-1.0.0.tar.gz">My_Package-1.0.0.tar.gz</a>
      <a href="../../files/my_package-1.1.0-py3-none-any.whl">my_package-1.1.0-py3-none-any.whl</a>
      <a data-yanked="broken" href="../../files/my_package-9.0.0.tar.gz">my_package-9.0.0.tar.gz</a>
      <a href="../../files/unrelated-8.0.0.tar.gz">unrelated-8.0.0.tar.gz</a>
    </body></html>"#;

    assert_eq!(
        latest(&endpoint, "My.Package", body),
        Some("1.1.0".to_owned())
    );
    assert_eq!(
        release_versions_from_response_for_endpoint(&endpoint, Python, "My.Package", body),
        ["1.0.0".to_owned(), "1.1.0".to_owned()]
    );
}

#[test]
fn python_simple_json_responses_filter_yanked_files() {
    let endpoint = RegistryEndpoint::new(
        "https://packages.test/simple/demo/".to_owned(),
        RegistryResponseKind::PythonSimple,
    );
    let body = r#"{"meta":{"api-version":"1.4"},"name":"demo","files":[{"filename":"demo-1.0.0.tar.gz","yanked":false},{"filename":"demo-1.2.0-py3-none-any.whl"},{"filename":"demo-3.0.0.tar.gz","yanked":"broken"}]}"#;

    assert_eq!(latest(&endpoint, "demo", body), Some("1.2.0".to_owned()));
}

#[test]
fn endpoint_parsers_reject_incompatible_response_shapes() {
    let go_list = RegistryEndpoint::new(
        "https://proxy.test/example.com/mod/@v/list".to_owned(),
        RegistryResponseKind::GoModuleList,
    );
    let go_latest = RegistryEndpoint::new(
        "https://proxy.test/example.com/mod/@latest".to_owned(),
        RegistryResponseKind::GoModuleLatest,
    );
    let python_simple = RegistryEndpoint::new(
        "https://packages.test/simple/demo/".to_owned(),
        RegistryResponseKind::PythonSimple,
    );
    let python_rss = RegistryEndpoint::new(
        "https://pypi.org/rss/project/demo/releases.xml".to_owned(),
        RegistryResponseKind::PythonRss,
    );

    assert_eq!(
        latest(&go_list, "example.com/mod", r#"{"Version":"v1.2.0"}"#),
        None
    );
    assert_eq!(
        latest(&go_latest, "example.com/mod", "v1.1.0\nv1.2.0\n"),
        None
    );
    assert_eq!(
        latest(
            &python_simple,
            "demo",
            r#"{"info":{"version":"9.0.0"},"releases":{}}"#,
        ),
        None
    );
    assert_eq!(
        latest(
            &python_rss,
            "demo",
            "<!doctype html><html><a href='demo-1.0.0.tar.gz'>demo-1.0.0.tar.gz</a></html>",
        ),
        None
    );
}
