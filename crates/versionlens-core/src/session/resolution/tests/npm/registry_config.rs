use super::*;
#[test]
fn npm_registry_urls_use_document_npmrc_scope_registry() {
    let root = temp_dir().join(format!("versionlens-npmrc-{}", id()));
    let package_dir = root.join("package");
    create_dir_all(&package_dir).unwrap();
    write(
        package_dir.join(".npmrc"),
        "registry=${DEFAULT_REGISTRY}\n@scope:registry=${SCOPE_REGISTRY}\n",
    )
    .unwrap();
    write(
        package_dir.join(".env"),
        "DEFAULT_REGISTRY=https://registry.example.test/\nSCOPE_REGISTRY=https://scope.example.test/npm\n",
    )
    .unwrap();

    assert_registry_case(
        document_input(
            &package_dir.join("package.json"),
            &root,
            "npm-registry-urls-use-document-npmrc-scope-registry.txt",
        ),
        &[
            &["https://scope.example.test/npm/@scope%2fpkg"],
            &["https://registry.example.test/left-pad"],
        ],
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn package_npmrc_registry_takes_precedence_over_workspace_npmrc() {
    let root = temp_dir().join(format!("versionlens-npmrc-precedence-{}", id()));
    let package_dir = root.join("package");
    create_dir_all(&package_dir).unwrap();
    write(
        package_dir.join(".npmrc"),
        "registry=https://package-registry.example.test/\n",
    )
    .unwrap();
    write(
        root.join(".npmrc"),
        "registry=https://workspace-registry.example.test/\n//package-registry.example.test/:_authToken=workspace-secret\n",
    )
    .unwrap();

    let input = document_input(
        &package_dir.join("package.json"),
        &root,
        "package-npmrc-registry-takes-precedence-over-workspace-npmrc.txt",
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_registry_urls(
        &session,
        &context,
        &dependencies,
        &[&["https://package-registry.example.test/left-pad"]],
    );
    let headers =
        context.auth_headers_for_url(Npm, "https://package-registry.example.test/left-pad");
    assert!(headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_auth_headers_use_most_specific_document_npmrc_token() {
    let root = temp_dir().join(format!("versionlens-npmrc-auth-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "//registry.example.test/:_authToken=${DEFAULT_TOKEN}\n//registry.example.test/npm/:_authToken=${SCOPED_TOKEN}\n",
    )
    .unwrap();
    write(
        root.join(".env"),
        "DEFAULT_TOKEN=default-secret\nSCOPED_TOKEN=scoped-secret\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-auth-headers-use-most-specific-document-npmrc-token.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let context = crate::registry::RegistryContext::from_document(&input);

    let default_headers =
        context.auth_headers_for_url(Npm, "https://registry.example.test/left-pad");
    let scoped_headers =
        context.auth_headers_for_url(Npm, "https://registry.example.test/npm/left-pad");
    let other_headers = context.auth_headers_for_url(Npm, "https://other.example.test/left-pad");

    assert_eq!(default_headers.len(), 1);
    assert_eq!(default_headers[0].name, "authorization");
    assert_eq!(default_headers[0].value, "Bearer default-secret");
    assert_eq!(scoped_headers.len(), 1);
    assert_eq!(scoped_headers[0].value, "Bearer scoped-secret");
    assert!(other_headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_document_npmrc_proxy_and_strict_ssl() {
    let root = temp_dir().join(format!("versionlens-npmrc-http-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "strict-ssl=false\nhttps-proxy=${NPM_PROXY}\ncafile=/tmp/npm-ca.pem\n",
    )
    .unwrap();
    write(
        root.join(".env"),
        "NPM_PROXY=http://proxy.example.test:8080\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-http-config-uses-document-npmrc-proxy-and-strict-ssl.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Npm),
    );

    assert!(!http.strict_ssl);
    assert_eq!(
        http.proxy.as_deref(),
        Some("http://proxy.example.test:8080")
    );
    assert_eq!(http.ca_file.as_deref(), Some("/tmp/npm-ca.pem"));

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_override_registry_or_http_defaults() {
    let root = temp_dir().join(format!("versionlens-npm-env-{}", id()));
    let package_dir = root.join("package");
    create_dir_all(&package_dir).unwrap();
    write(
        package_dir.join(".env"),
        format!(
            "HOME={}\nnpm_config_registry=https://env-registry.example.test/\nnpm_config_strict_ssl=false\nnpm_config_https_proxy=http://env-proxy.example.test:8080\nnpm_config_cafile=/tmp/env-ca.pem\n",
            root.join("home").display()
        ),
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", package_dir.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture(
            "npm-env-file-without-npmrc-does-not-override-registry-or-http-defaults.txt",
        ),
        Some(root.to_string_lossy().into_owned()),
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);
    let http = context.http_config_for_request(
        Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Npm),
    );

    assert_registry_urls(
        &session,
        &context,
        &dependencies,
        &[&["https://registry.npmjs.org/left-pad"]],
    );
    assert!(http.strict_ssl);
    assert_eq!(http.proxy, None);
    assert_eq!(http.ca_file, None);

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_select_userconfig() {
    let root = temp_dir().join(format!("versionlens-npmrc-userconfig-{}", id()));
    let package_dir = root.join("package");
    create_dir_all(&package_dir).unwrap();
    let userconfig_path = root.join("user.npmrc");
    write(
        &userconfig_path,
        "registry=https://user-registry.example.test/\n//user-registry.example.test/:_authToken=user-secret\n",
    )
    .unwrap();
    write(
        package_dir.join(".env"),
        format!("NPM_CONFIG_USERCONFIG={}\n", userconfig_path.display()),
    )
    .unwrap();

    let input = document_input(
        &package_dir.join("package.json"),
        &root,
        "npm-env-file-without-npmrc-does-not-select-userconfig.txt",
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_registry_urls(
        &session,
        &context,
        &dependencies,
        &[&["https://registry.npmjs.org/left-pad"]],
    );

    let headers = context.auth_headers_for_url(Npm, "https://user-registry.example.test/left-pad");
    assert!(headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_select_home_userconfig() {
    let root = temp_dir().join(format!("versionlens-npmrc-home-userconfig-{}", id()));
    let package_dir = root.join("package");
    let home_dir = root.join("home");
    create_dir_all(&package_dir).unwrap();
    create_dir_all(&home_dir).unwrap();
    write(
        home_dir.join(".npmrc"),
        "registry=https://home-registry.example.test/\n//home-registry.example.test/:_authToken=home-secret\n",
    )
    .unwrap();
    write(
        package_dir.join(".env"),
        format!("NPM_CONFIG_USERCONFIG=\nHOME={}\n", home_dir.display()),
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", package_dir.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("npm-env-file-without-npmrc-does-not-select-home-userconfig.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );

    let headers = context.auth_headers_for_url(Npm, "https://home-registry.example.test/left-pad");
    assert!(headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn package_json_uses_workspace_yarnrc_registry_and_token() {
    let root = temp_dir().join(format!("versionlens-yarnrc-registry-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmScopes:\n  scope:\n    npmRegistryServer: https://scope.example.test/npm\n    npmAuthToken: ${SCOPE_TOKEN}\n",
    )
    .unwrap();
    write(root.join(".env"), "SCOPE_TOKEN=scoped-secret\n").unwrap();

    let input = document_input(
        &root.join("package.json"),
        &root,
        "package-json-uses-workspace-yarnrc-registry-and-token.txt",
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_registry_urls(
        &session,
        &context,
        &dependencies,
        &[
            &["https://scope.example.test/npm/@scope%2fpkg"],
            &["https://registry.example.test/left-pad"],
        ],
    );

    let headers = context.auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg");
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, "Bearer scoped-secret");

    remove_dir_all(root).unwrap();
}

#[test]
fn yarnrc_document_ignores_unsaved_registry_text() {
    let input = DocumentInput::new(
        "file:///work/.yarnrc.yml".to_owned(),
        "yaml".to_owned(),
        package_file_fixture("yarnrc-document-ignores-unsaved-registry-text.yarnrc.yml"),
        None,
    );
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

    assert_eq!(dependencies[0].name, "left-pad");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );
}

#[test]
fn npm_basic_auth_headers_use_document_npmrc_auth() {
    let root = temp_dir().join(format!("versionlens-npmrc-basic-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "//registry.example.test/:_auth=${BASIC_TOKEN}\n",
    )
    .unwrap();
    write(root.join(".env"), "BASIC_TOKEN=dXNlcjpwYXNz\n").unwrap();

    let input = document_input(
        &root.join("package.json"),
        &root,
        "npm-basic-auth-headers-use-document-npmrc-auth.txt",
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    assert_single_auth_header(
        &context,
        Npm,
        "https://registry.example.test/left-pad",
        "Basic dXNlcjpwYXNz",
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn yarnrc_auth_ident_headers_use_workspace_yarnrc_auth() {
    let root = temp_dir().join(format!("versionlens-yarnrc-basic-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmAuthIdent: ${YARN_IDENT}\n",
    )
    .unwrap();
    write(root.join(".env"), "YARN_IDENT=user:pass\n").unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("package.json").display()),
        "json".to_owned(),
        package_file_fixture("yarnrc-auth-ident-headers-use-workspace-yarnrc-auth.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    let context = crate::registry::RegistryContext::from_document(&input);
    assert_single_auth_header(
        &context,
        Npm,
        "https://registry.example.test/left-pad",
        "Basic dXNlcjpwYXNz",
    );

    remove_dir_all(root).unwrap();
}

fn assert_registry_urls(
    session: &crate::VersionLensSession,
    context: &crate::registry::RegistryContext,
    dependencies: &[versionlens_model::Dependency],
    expected: &[&[&str]],
) {
    for (dependency, urls) in dependencies.iter().zip(expected) {
        crate::support::tests::assert_registry_urls(session, context, dependency, urls);
    }
}

fn assert_registry_case(input: DocumentInput, expected: &[&[&str]]) {
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);
    assert_registry_urls(&session, &context, &dependencies, expected);
}

fn document_input(
    document: &std::path::Path,
    workspace_root: &std::path::Path,
    fixture: &str,
) -> DocumentInput {
    DocumentInput::new(
        format!("file://{}", document.display()),
        "json".to_owned(),
        package_file_fixture(fixture),
        Some(workspace_root.to_string_lossy().into_owned()),
    )
}
