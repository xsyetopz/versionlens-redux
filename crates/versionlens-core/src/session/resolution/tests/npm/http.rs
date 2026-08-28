use super::*;

#[test]
fn npm_http_config_uses_document_npmrc_fetch_timeout() {
    let root = temp_dir().join(format!("versionlens-npmrc-timeout-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), "fetch-timeout=45000\n").unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-http-config-uses-document-npmrc-fetch-timeout.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, "https://registry.npmjs.org/left-pad");

    assert_eq!(http.timeout_ms, 45_000);

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_registry_http_config_uses_npm_registry_fetch_default_timeout() {
    let input = DocumentInput::new(
        "file:///workspace/package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("registry-http-config-uses-npm-registry-fetch-default-timeout.json"),
        Some("/workspace".to_owned()),
    );
    let http = http_config(&input, "https://registry.npmjs.org/left-pad");

    assert_eq!(http.timeout_ms, 300_000);
}

#[test]
fn npm_github_http_config_keeps_extension_timeout() {
    let input = DocumentInput::new(
        "file:///workspace/package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("github-http-config-keeps-extension-timeout.json"),
        Some("/workspace".to_owned()),
    );
    let http = http_config(&input, "https://api.github.com/repos/owner/repo/tags");

    assert_eq!(http.timeout_ms, 10_000);
}

#[test]
fn npm_registry_http_config_maps_zero_fetch_timeout_to_npm_registry_fetch_fallback() {
    let root = temp_dir().join(format!("versionlens-npmrc-zero-timeout-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), "fetch-timeout=0\n").unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "npm-registry-http-config-maps-zero-fetch-timeout-to-npm-registry-fetch-fallback.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, "https://registry.npmjs.org/left-pad");

    assert_eq!(http.timeout_ms, 30_000);

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_bypasses_proxy_for_noproxy_host() {
    let http = http_config_from_npmrc(
        "versionlens-npmrc-noproxy",
        "https-proxy=http://proxy.example.test:8080\nnoproxy=registry.npmjs.org\n",
        None,
        "npm-http-config-bypasses-proxy-for-noproxy-host.txt",
        "https://registry.npmjs.org/left-pad",
    );
    assert_eq!(http.proxy, None);
}

#[test]
fn npm_http_config_uses_generic_https_proxy_from_env_when_npm_proxy_absent() {
    let http = http_config_from_npmrc(
        "versionlens-npmrc-env-proxy",
        "registry=https://registry.npmjs.org/\n",
        Some("HTTPS_PROXY=http://generic-proxy.example.test:8080\n"),
        "npm-http-config-uses-generic-https-proxy-from-env-when-npm-proxy-absent.txt",
        "https://registry.npmjs.org/left-pad",
    );

    assert_eq!(
        http.proxy.as_deref(),
        Some("http://generic-proxy.example.test:8080")
    );
}

#[test]
fn npm_https_registry_http_config_ignores_generic_http_proxy_without_https_proxy() {
    let http = http_config_from_npmrc(
        "versionlens-npmrc-env-https-http-proxy",
        "registry=https://registry.npmjs.org/\n",
        Some(
            "HTTP_PROXY=http://http-proxy.example.test:8080\nPROXY=http://plain-proxy.example.test:8080\n",
        ),
        "npm-https-registry-http-config-ignores-generic-http-proxy-without-https-proxy.txt",
        "https://registry.npmjs.org/left-pad",
    );

    assert_eq!(http.proxy, None);
}

#[test]
fn npm_https_registry_http_config_ignores_generic_plain_proxy_without_https_proxy() {
    let http = http_config_from_npmrc(
        "versionlens-npmrc-env-https-plain-proxy",
        "registry=https://registry.npmjs.org/\n",
        Some("PROXY=http://plain-proxy.example.test:8080\n"),
        "npm-https-registry-http-config-ignores-generic-plain-proxy-without-https-proxy.txt",
        "https://registry.npmjs.org/left-pad",
    );
    assert_eq!(http.proxy, None);
}

#[test]
fn npm_http_registry_http_config_uses_generic_http_proxy_when_https_proxy_absent() {
    let root = temp_dir().join(format!("versionlens-npmrc-env-http-proxy-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "registry=http://registry.example.test/
",
    )
    .unwrap();
    write(
        root.join(".env"),
        "HTTP_PROXY=http://http-proxy.example.test:8080
PROXY=http://plain-proxy.example.test:8080
",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "npm-http-registry-http-config-uses-generic-http-proxy-when-https-proxy-absent.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, "http://registry.example.test/left-pad");

    assert_eq!(
        http.proxy.as_deref(),
        Some("http://http-proxy.example.test:8080")
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_direct_tls_pem_options() {
    let root = temp_dir().join(format!("versionlens-npmrc-direct-tls-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "ca=direct-ca\ncert=direct-cert\nkey=direct-key\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-http-config-uses-direct-tls-pem-options.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, "https://registry.example.test/left-pad");

    assert_eq!(http.ca.as_deref(), Some("direct-ca"));
    assert_eq!(http.cert.as_deref(), Some("direct-cert"));
    assert_eq!(http.key.as_deref(), Some("direct-key"));

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_registry_scoped_client_cert_files_override_direct_cert_and_key() {
    let root = temp_dir().join(format!("versionlens-npmrc-mtls-override-{}", id()));
    create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    let key_file = root.join("client-key.pem");
    write_client_cert_config(&root, &cert_file, Some(&key_file), true);

    let input = http_fixture_input(
        &root,
        "npm-registry-scoped-client-cert-files-override-direct-cert-and-key.txt",
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    let session = standard_session();
    let (matching, other) = scoped_request_configs(&context, &session);

    assert_overridden_client_cert_config(&matching, &other, &cert_file, &key_file);

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_partial_registry_scoped_client_cert_files_do_not_override_direct_cert_and_key() {
    let root = temp_dir().join(format!("versionlens-npmrc-mtls-partial-{}", id()));
    create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    write(
        root.join(".npmrc"),
        format!(
            "cert=direct-cert\nkey=direct-key\n//registry.example.test/:certfile={}\n",
            cert_file.display()
        ),
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "npm-partial-registry-scoped-client-cert-files-do-not-override-direct-cert-and-key.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, "https://registry.example.test/left-pad");

    assert_eq!(http.cert.as_deref(), Some("direct-cert"));
    assert_eq!(http.key.as_deref(), Some("direct-key"));
    assert_eq!(http.cert_file, None);
    assert_eq!(http.key_file, None);

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_registry_scoped_client_cert_files() {
    let root = temp_dir().join(format!("versionlens-npmrc-mtls-{}", id()));
    create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    let key_file = root.join("client-key.pem");
    write(
        root.join(".npmrc"),
        format!(
            "//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
            cert_file.display(),
            key_file.display()
        ),
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-http-config-uses-registry-scoped-client-cert-files.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    let session = standard_session();
    let matching = context.http_config_for_request(
        Npm,
        "https://registry.example.test/left-pad",
        session.http_config(Npm),
    );
    let other = context.http_config_for_request(
        Npm,
        "https://other.example.test/left-pad",
        session.http_config(Npm),
    );

    assert_regular_client_cert_config(&matching, &other, &cert_file, &key_file);

    remove_dir_all(root).unwrap();
}

#[test]
fn npmrc_proxy_false_disables_extension_proxy_for_npm_registry_fetches() {
    let root = temp_dir().join(format!("versionlens-npmrc-proxy-false-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "proxy=false
",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "npmrc-proxy-false-disables-extension-proxy-for-npm-registry-fetches.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    let mut base = standard_session().http_config(Npm);
    base.proxy = Some("http://extension-proxy.example.test:8080".to_owned());
    let http = context.http_config_for_request(Npm, "https://registry.npmjs.org/left-pad", base);

    assert_eq!(http.proxy, None);

    remove_dir_all(root).unwrap();
}

fn http_config(input: &DocumentInput, url: &str) -> versionlens_http::HttpConfig {
    let context = crate::registry::RegistryContext::from_document(input);
    let session = standard_session();
    context.http_config_for_request(Npm, url, session.http_config(Npm))
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/npm/http", name)
}

fn http_fixture_input(root: &std::path::Path, fixture: &str) -> DocumentInput {
    DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(fixture),
        Some(root.to_string_lossy().into_owned()),
    )
}

fn write_client_cert_config(
    root: &std::path::Path,
    cert_file: &std::path::Path,
    key_file: Option<&std::path::Path>,
    direct: bool,
) {
    let direct = if direct {
        "cert=direct-cert\nkey=direct-key\n"
    } else {
        ""
    };
    let scoped = match key_file {
        Some(key) => format!(
            "//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
            cert_file.display(),
            key.display()
        ),
        None => format!(
            "//registry.example.test/:certfile={}\n",
            cert_file.display()
        ),
    };
    write(root.join(".npmrc"), format!("{direct}{scoped}")).unwrap();
}

fn scoped_request_configs(
    context: &crate::registry::RegistryContext,
    session: &crate::VersionLensSession,
) -> (versionlens_http::HttpConfig, versionlens_http::HttpConfig) {
    (
        context.http_config_for_request(
            Npm,
            "https://registry.example.test/left-pad",
            session.http_config(Npm),
        ),
        context.http_config_for_request(
            Npm,
            "https://other.example.test/left-pad",
            session.http_config(Npm),
        ),
    )
}

fn assert_overridden_client_cert_config(
    matching: &versionlens_http::HttpConfig,
    other: &versionlens_http::HttpConfig,
    cert_file: &std::path::Path,
    key_file: &std::path::Path,
) {
    assert_eq!(matching.cert, None);
    assert_eq!(matching.key, None);
    assert_client_cert_files(matching, cert_file, key_file);
    assert_eq!(other.cert.as_deref(), Some("direct-cert"));
    assert_eq!(other.key.as_deref(), Some("direct-key"));
}

fn assert_regular_client_cert_config(
    matching: &versionlens_http::HttpConfig,
    other: &versionlens_http::HttpConfig,
    cert_file: &std::path::Path,
    key_file: &std::path::Path,
) {
    assert_client_cert_files(matching, cert_file, key_file);
    assert_eq!(other.cert_file, None);
    assert_eq!(other.key_file, None);
}

fn assert_client_cert_files(
    config: &versionlens_http::HttpConfig,
    cert_file: &std::path::Path,
    key_file: &std::path::Path,
) {
    assert_eq!(
        config.cert_file.as_deref(),
        Some(cert_file.to_string_lossy().as_ref())
    );
    assert_eq!(
        config.key_file.as_deref(),
        Some(key_file.to_string_lossy().as_ref())
    );
}

fn http_config_from_npmrc(
    root_name: &str,
    npmrc: &str,
    env: Option<&str>,
    fixture: &str,
    url: &str,
) -> versionlens_http::HttpConfig {
    let root = temp_dir().join(format!("{root_name}-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".npmrc"), npmrc).unwrap();
    if let Some(env) = env {
        write(root.join(".env"), env).unwrap();
    }
    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(fixture),
        Some(root.to_string_lossy().into_owned()),
    );
    let http = http_config(&input, url);
    remove_dir_all(root).unwrap();
    http
}
