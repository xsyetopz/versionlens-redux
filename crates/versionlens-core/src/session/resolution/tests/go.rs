use super::*;

#[test]
fn go_mod_exclude_versions_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput::new(
            "file:///go.mod".to_owned(),
            "go.mod".to_owned(),
            package_file_fixture("go-mod-exclude-versions-are-fixed-without-registry-updates.mod"),
            None,
        ),
        &[RegistryResponseInput::new(
            "example.test/bad".to_owned(),
            Go,
            "v1.0.0\nv1.1.0\n".to_owned(),
        )],
    );

    crate::support::tests::assert_fixed_suggestion(&output, "excluded version");
}

fn assert_go_proxy_configuration(env: &str, fixture: &str, expected_urls: &[&str]) {
    let root = temp_dir().join(format!(
        "versionlens-go-proxy-{}-{}",
        id(),
        fixture.replace('.', "-")
    ));
    create_dir_all(&root).unwrap();
    write(root.join(".env"), env).unwrap();
    let input = DocumentInput::new(
        format!("file://{}", root.join("go.mod").display()),
        "go.mod".to_owned(),
        package_file_fixture(fixture),
        Some(root.to_string_lossy().into_owned()),
    );
    crate::session::resolution::tests::registry_case_with_expected_urls(&input, &[expected_urls]);
    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_uses_workspace_go_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=https://proxy.example.test/,direct|https://fallback.example.test|off\n",
        "go-mod-uses-workspace-go-proxy-urls.txt",
        &[
            "https://proxy.example.test/!go.uber.org/!zap/@v/list",
            "https://proxy.example.test/!go.uber.org/!zap/@latest",
            "https://fallback.example.test/!go.uber.org/!zap/@v/list",
            "https://fallback.example.test/!go.uber.org/!zap/@latest",
        ],
    );
}

#[test]
fn go_mod_goproxy_off_disables_default_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=off\n",
        "go-mod-goproxy-off-disables-default-proxy-urls.txt",
        &[],
    );
}

#[test]
fn go_mod_goproxy_direct_disables_default_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=direct\n",
        "go-mod-goproxy-direct-disables-default-proxy-urls.txt",
        &[],
    );
}

#[test]
fn go_mod_goprivate_dependencies_do_not_use_module_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=corp.example.com\n",
        "go-mod-goprivate-dependencies-do-not-use-module-proxy-urls.txt",
        &[],
    );
}

#[test]
fn go_mod_goprivate_path_match_patterns_disable_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=[a-z].corp.example.com\n",
        "go-mod-goprivate-path-match-patterns-disable-proxy-urls.txt",
        &[],
    );
}

#[test]
fn go_mod_goproxy_off_in_list_disables_later_proxy_urls() {
    assert_go_proxy_configuration(
        "GOPROXY=off,https://proxy.example.test\n",
        "go-mod-goproxy-off-in-list-disables-later-proxy-urls.txt",
        &[],
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/go", name)
}
