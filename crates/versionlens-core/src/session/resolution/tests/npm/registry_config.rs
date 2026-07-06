use super::{DocumentInput, package_file_fixture, standard_session};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;
use versionlens_parsers::Ecosystem::Npm;
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

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-registry-urls-use-document-npmrc-scope-registry.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://scope.example.test/npm/@scope%2fpkg"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://registry.example.test/left-pad"]
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

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "package-npmrc-registry-takes-precedence-over-workspace-npmrc.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://package-registry.example.test/left-pad"]
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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-auth-headers-use-most-specific-document-npmrc-token.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);

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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-http-config-uses-document-npmrc-proxy-and-strict-ssl.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
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

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "npm-env-file-without-npmrc-does-not-override-registry-or-http-defaults.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Npm),
    );

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
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

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-env-file-without-npmrc-does-not-select-userconfig.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
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

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture(
            "npm-env-file-without-npmrc-does-not-select-home-userconfig.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("package-json-uses-workspace-yarnrc-registry-and-token.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://scope.example.test/npm/@scope%2fpkg"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://registry.example.test/left-pad"]
    );

    let headers = context.auth_headers_for_url(Npm, "https://scope.example.test/npm/@scope%2fpkg");
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, "Bearer scoped-secret");

    remove_dir_all(root).unwrap();
}

#[test]
fn yarnrc_document_ignores_unsaved_registry_text() {
    let input = DocumentInput {
        uri: "file:///work/.yarnrc.yml".to_owned(),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("yarnrc-document-ignores-unsaved-registry-text.yarnrc.yml"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("npm-basic-auth-headers-use-document-npmrc-auth.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let headers = context.auth_headers_for_url(Npm, "https://registry.example.test/left-pad");

    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: package_file_fixture("yarnrc-auth-ident-headers-use-workspace-yarnrc-auth.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let headers = context.auth_headers_for_url(Npm, "https://registry.example.test/left-pad");

    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

    remove_dir_all(root).unwrap();
}
