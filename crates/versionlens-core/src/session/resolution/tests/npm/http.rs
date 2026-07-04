use super::{DocumentInput, Ecosystem, standard_session};
use crate::registry::RegistryContext;

#[test]
fn npm_http_config_uses_document_npmrc_fetch_timeout() {
    let root =
        std::env::temp_dir().join(format!("versionlens-npmrc-timeout-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(".npmrc"), "fetch-timeout=45000\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.timeout_ms, 45_000);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_registry_http_config_uses_npm_registry_fetch_default_timeout() {
    let input = DocumentInput {
        uri: "file:///workspace/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.timeout_ms, 300_000);
}

#[test]
fn npm_github_http_config_keeps_extension_timeout() {
    let input = DocumentInput {
        uri: "file:///workspace/package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"github:owner/repo#1.0.0"}}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://api.github.com/repos/owner/repo/tags",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.timeout_ms, 10_000);
}

#[test]
fn npm_registry_http_config_maps_zero_fetch_timeout_to_npm_registry_fetch_fallback() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-zero-timeout-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(".npmrc"), "fetch-timeout=0\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.timeout_ms, 30_000);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_bypasses_proxy_for_noproxy_host() {
    let root =
        std::env::temp_dir().join(format!("versionlens-npmrc-noproxy-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "https-proxy=http://proxy.example.test:8080
noproxy=registry.npmjs.org
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.proxy, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_generic_https_proxy_from_env_when_npm_proxy_absent() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-env-proxy-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://registry.npmjs.org/
",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "HTTPS_PROXY=http://generic-proxy.example.test:8080
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(
        http.proxy.as_deref(),
        Some("http://generic-proxy.example.test:8080")
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_https_registry_http_config_ignores_generic_http_proxy_without_https_proxy() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-env-https-http-proxy-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://registry.npmjs.org/
",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "HTTP_PROXY=http://http-proxy.example.test:8080
PROXY=http://plain-proxy.example.test:8080
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.proxy, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_https_registry_http_config_ignores_generic_plain_proxy_without_https_proxy() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-env-https-plain-proxy-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://registry.npmjs.org/
",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "PROXY=http://plain-proxy.example.test:8080
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.proxy, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_registry_http_config_uses_generic_http_proxy_when_https_proxy_absent() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-env-http-proxy-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=http://registry.example.test/
",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "HTTP_PROXY=http://http-proxy.example.test:8080
PROXY=http://plain-proxy.example.test:8080
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "http://registry.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(
        http.proxy.as_deref(),
        Some("http://http-proxy.example.test:8080")
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_direct_tls_pem_options() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-direct-tls-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "ca=direct-ca\ncert=direct-cert\nkey=direct-key\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.ca.as_deref(), Some("direct-ca"));
    assert_eq!(http.cert.as_deref(), Some("direct-cert"));
    assert_eq!(http.key.as_deref(), Some("direct-key"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_registry_scoped_client_cert_files_override_direct_cert_and_key() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-mtls-override-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    let key_file = root.join("client-key.pem");
    std::fs::write(
        root.join(".npmrc"),
        format!(
            "cert=direct-cert\nkey=direct-key\n//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
            cert_file.display(),
            key_file.display()
        ),
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let matching = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );
    let other = context.http_config_for_request(
        Ecosystem::Npm,
        "https://other.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(matching.cert, None);
    assert_eq!(matching.key, None);
    assert_eq!(
        matching.cert_file.as_deref(),
        Some(cert_file.to_string_lossy().as_ref())
    );
    assert_eq!(
        matching.key_file.as_deref(),
        Some(key_file.to_string_lossy().as_ref())
    );
    assert_eq!(other.cert.as_deref(), Some("direct-cert"));
    assert_eq!(other.key.as_deref(), Some("direct-key"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_partial_registry_scoped_client_cert_files_do_not_override_direct_cert_and_key() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-mtls-partial-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    std::fs::write(
        root.join(".npmrc"),
        format!(
            "cert=direct-cert\nkey=direct-key\n//registry.example.test/:certfile={}\n",
            cert_file.display()
        ),
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(http.cert.as_deref(), Some("direct-cert"));
    assert_eq!(http.key.as_deref(), Some("direct-key"));
    assert_eq!(http.cert_file, None);
    assert_eq!(http.key_file, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_registry_scoped_client_cert_files() {
    let root = std::env::temp_dir().join(format!("versionlens-npmrc-mtls-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let cert_file = root.join("client-cert.pem");
    let key_file = root.join("client-key.pem");
    std::fs::write(
        root.join(".npmrc"),
        format!(
            "//registry.example.test/:certfile={}\n//registry.example.test/:keyfile={}\n",
            cert_file.display(),
            key_file.display()
        ),
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let session = standard_session();
    let matching = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );
    let other = context.http_config_for_request(
        Ecosystem::Npm,
        "https://other.example.test/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(
        matching.cert_file.as_deref(),
        Some(cert_file.to_string_lossy().as_ref())
    );
    assert_eq!(
        matching.key_file.as_deref(),
        Some(key_file.to_string_lossy().as_ref())
    );
    assert_eq!(other.cert_file, None);
    assert_eq!(other.key_file, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npmrc_proxy_false_disables_extension_proxy_for_npm_registry_fetches() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-proxy-false-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "proxy=false
",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let mut base = standard_session().http_config(Ecosystem::Npm);
    base.proxy = Some("http://extension-proxy.example.test:8080".to_owned());
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        base,
    );

    assert_eq!(http.proxy, None);

    std::fs::remove_dir_all(root).unwrap();
}
