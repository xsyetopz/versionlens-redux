use super::{
    DocumentInput, Ecosystem, RegistryResponseInput, session_without_vulnerabilities,
    standard_session,
};
use crate::registry::RegistryContext;

mod dist_tags;
mod http;

#[test]
fn resolves_update_from_registry_response_body() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"1.1.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn resolves_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"aliased":"npm:typescript@6.0.3"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "typescript".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"6.0.4"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "typescript");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:typescript@6.0.4");
}

#[test]
fn resolves_ranged_npm_alias_dependencies_preserving_range_prefix() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"aliased":"npm:@types/react@^19.0.0"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "@types/react".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"20.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "@types/react");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:@types/react@^20.0.0");
}

#[test]
fn resolves_unversioned_npm_alias_dependencies_against_target_package() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"aliased":"npm:types-react"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "types-react".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"19.2.7"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].dependency.name, "types-react");
    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "npm:types-react@19.2.7");
}

#[test]
fn resolves_invalid_empty_ranges_as_invalid_range_with_latest_update() {
    let session = session_without_vulnerabilities();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///package.json".to_owned(),
            language_id: "json".to_owned(),
            text: r#"{"dependencies":{"left-pad":">1 <1"}}"#.to_owned(),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{"dist-tags":{"latest":"5.0.0"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "invalidRange");
    assert_eq!(output.edits[0].new_text, "5.0.0");
}

#[test]
fn missing_fixed_npm_registry_version_resolves_no_match_with_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"0.5.0"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "dist-tags": { "latest": "1.0.0" },
              "versions": {
                "0.5.1": {},
                "0.6.0": {},
                "1.0.0": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "noMatch");
    assert_eq!(output.suggestions[0].latest, None);
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "⚪ no match",
            "↑  latest 1.0.0",
            "↑  minor 0.6.0",
            "↑  patch 0.5.1"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "1.0.0"],
            vec!["updateMinor", "0.6.0"],
            vec!["updatePatch", "0.5.1"]
        ]
    );
}

#[test]
fn fixed_npm_prerelease_resolves_fixed_with_prerelease_update_choice() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0-beta.1"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "versions": {
                "1.0.0-beta.1": {},
                "1.0.0-beta.2": {},
                "1.0.0-beta.3": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("1.0.0-beta.1")
    );
    assert!(output.edits.is_empty());
    assert_eq!(titles, ["🟡 fixed 1.0.0-beta.1", "↑  beta 1.0.0-beta.3"]);
    assert_eq!(arguments, [vec!["update", "1.0.0-beta.3"]]);
}

#[test]
fn fixed_npm_release_resolves_fixed_with_release_update_choices() {
    let session = session_without_vulnerabilities();
    let input = DocumentInput {
        uri: "file:///package.json".to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.1.1"}}"#.to_owned(),
        workspace_root: None,
    };

    let output = session.resolve_document_with_responses(
        input.clone(),
        &[RegistryResponseInput {
            package: "left-pad".to_owned(),
            ecosystem: Ecosystem::Npm,
            body: r#"{
              "versions": {
                "1.1.0": {},
                "1.1.1": {},
                "1.1.2": {},
                "1.2.0": {},
                "1.2.2": {},
                "2.0.0": {},
                "2.2.2": {}
              }
            }"#
            .to_owned(),
        }],
    );

    let analysis = session.analyze_document(input);
    let titles = analysis
        .code_lenses
        .iter()
        .map(|lens| lens.title.as_str())
        .collect::<Vec<_>>();
    let arguments = analysis
        .code_lenses
        .iter()
        .skip(1)
        .map(|lens| {
            lens.arguments
                .iter()
                .skip(2)
                .map(String::as_str)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
    assert_eq!(
        titles,
        [
            "🟡 fixed 1.1.1",
            "↑  latest 2.2.2",
            "↑  minor 1.2.2",
            "↑  patch 1.1.2"
        ]
    );
    assert_eq!(
        arguments,
        [
            vec!["update", "2.2.2"],
            vec!["updateMinor", "1.2.2"],
            vec!["updatePatch", "1.1.2"]
        ]
    );
}

#[test]
fn npm_registry_urls_use_document_npmrc_scope_registry() {
    let root = std::env::temp_dir().join(format!("versionlens-npmrc-{}", std::process::id()));
    let package_dir = root.join("package");
    std::fs::create_dir_all(&package_dir).unwrap();
    std::fs::write(
        package_dir.join(".npmrc"),
        "registry=${DEFAULT_REGISTRY}\n@scope:registry=${SCOPE_REGISTRY}\n",
    )
    .unwrap();
    std::fs::write(
        package_dir.join(".env"),
        "DEFAULT_REGISTRY=https://registry.example.test/\nSCOPE_REGISTRY=https://scope.example.test/npm\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"@scope/pkg":"1.0.0","left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
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

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_npmrc_registry_takes_precedence_over_workspace_npmrc() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-precedence-{}",
        std::process::id()
    ));
    let package_dir = root.join("package");
    std::fs::create_dir_all(&package_dir).unwrap();
    std::fs::write(
        package_dir.join(".npmrc"),
        "registry=https://package-registry.example.test/\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://workspace-registry.example.test/\n//package-registry.example.test/:_authToken=workspace-secret\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://package-registry.example.test/left-pad"]
    );
    let headers = context.auth_headers_for_url(
        Ecosystem::Npm,
        "https://package-registry.example.test/left-pad",
    );
    assert!(headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_auth_headers_use_most_specific_document_npmrc_token() {
    let root = std::env::temp_dir().join(format!("versionlens-npmrc-auth-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "//registry.example.test/:_authToken=${DEFAULT_TOKEN}\n//registry.example.test/npm/:_authToken=${SCOPED_TOKEN}\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "DEFAULT_TOKEN=default-secret\nSCOPED_TOKEN=scoped-secret\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);

    let default_headers =
        context.auth_headers_for_url(Ecosystem::Npm, "https://registry.example.test/left-pad");
    let scoped_headers =
        context.auth_headers_for_url(Ecosystem::Npm, "https://registry.example.test/npm/left-pad");
    let other_headers =
        context.auth_headers_for_url(Ecosystem::Npm, "https://other.example.test/left-pad");

    assert_eq!(default_headers.len(), 1);
    assert_eq!(default_headers[0].name, "authorization");
    assert_eq!(default_headers[0].value, "Bearer default-secret");
    assert_eq!(scoped_headers.len(), 1);
    assert_eq!(scoped_headers[0].value, "Bearer scoped-secret");
    assert!(other_headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_http_config_uses_document_npmrc_proxy_and_strict_ssl() {
    let root = std::env::temp_dir().join(format!("versionlens-npmrc-http-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "strict-ssl=false\nhttps-proxy=${NPM_PROXY}\ncafile=/tmp/npm-ca.pem\n",
    )
    .unwrap();
    std::fs::write(
        root.join(".env"),
        "NPM_PROXY=http://proxy.example.test:8080\n",
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

    assert!(!http.strict_ssl);
    assert_eq!(
        http.proxy.as_deref(),
        Some("http://proxy.example.test:8080")
    );
    assert_eq!(http.ca_file.as_deref(), Some("/tmp/npm-ca.pem"));

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_override_registry_or_http_defaults() {
    let root = std::env::temp_dir().join(format!("versionlens-npm-env-{}", std::process::id()));
    let package_dir = root.join("package");
    std::fs::create_dir_all(&package_dir).unwrap();
    std::fs::write(
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
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();
    let http = context.http_config_for_request(
        Ecosystem::Npm,
        "https://registry.npmjs.org/left-pad",
        session.http_config(Ecosystem::Npm),
    );

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );
    assert!(http.strict_ssl);
    assert_eq!(http.proxy, None);
    assert_eq!(http.ca_file, None);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_select_userconfig() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-userconfig-{}",
        std::process::id()
    ));
    let package_dir = root.join("package");
    std::fs::create_dir_all(&package_dir).unwrap();
    let userconfig_path = root.join("user.npmrc");
    std::fs::write(
        &userconfig_path,
        "registry=https://user-registry.example.test/\n//user-registry.example.test/:_authToken=user-secret\n",
    )
    .unwrap();
    std::fs::write(
        package_dir.join(".env"),
        format!("NPM_CONFIG_USERCONFIG={}\n", userconfig_path.display()),
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Npm,
        "https://user-registry.example.test/left-pad",
    );
    assert!(headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn npm_env_file_without_npmrc_does_not_select_home_userconfig() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-npmrc-home-userconfig-{}",
        std::process::id()
    ));
    let package_dir = root.join("package");
    let home_dir = root.join("home");
    std::fs::create_dir_all(&package_dir).unwrap();
    std::fs::create_dir_all(&home_dir).unwrap();
    std::fs::write(
        home_dir.join(".npmrc"),
        "registry=https://home-registry.example.test/\n//home-registry.example.test/:_authToken=home-secret\n",
    )
    .unwrap();
    std::fs::write(
        package_dir.join(".env"),
        format!("NPM_CONFIG_USERCONFIG=\nHOME={}\n", home_dir.display()),
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", package_dir.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Npm,
        "https://home-registry.example.test/left-pad",
    );
    assert!(headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn package_json_ignores_workspace_yarnrc_registry_and_token() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-yarnrc-registry-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmScopes:\n  scope:\n    npmRegistryServer: https://scope.example.test/npm\n    npmAuthToken: ${SCOPE_TOKEN}\n",
    )
    .unwrap();
    std::fs::write(root.join(".env"), "SCOPE_TOKEN=scoped-secret\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"@scope/pkg":"1.0.0","left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.npmjs.org/@scope%2fpkg"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://registry.npmjs.org/left-pad"]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Npm,
        "https://scope.example.test/npm/@scope%2fpkg",
    );
    assert!(headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn yarnrc_document_ignores_unsaved_registry_text() {
    let input = DocumentInput {
        uri: "file:///work/.yarnrc.yml".to_owned(),
        language_id: "yaml".to_owned(),
        text: "npmRegistryServer: https://registry.example.test\ncatalog:\n  left-pad: ^1.0.0\n"
            .to_owned(),
        workspace_root: None,
    };
    let context = RegistryContext::from_document(&input);
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
    let root = std::env::temp_dir().join(format!("versionlens-npmrc-basic-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "//registry.example.test/:_auth=${BASIC_TOKEN}\n",
    )
    .unwrap();
    std::fs::write(root.join(".env"), "BASIC_TOKEN=dXNlcjpwYXNz\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let headers =
        context.auth_headers_for_url(Ecosystem::Npm, "https://registry.example.test/left-pad");

    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].name, "authorization");
    assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn yarnrc_auth_ident_headers_are_ignored() {
    let root =
        std::env::temp_dir().join(format!("versionlens-yarnrc-basic-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".yarnrc.yml"),
        "npmRegistryServer: https://registry.example.test\nnpmAuthIdent: ${YARN_IDENT}\n",
    )
    .unwrap();
    std::fs::write(root.join(".env"), "YARN_IDENT=user:pass\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("package.json").display()),
        language_id: "json".to_owned(),
        text: r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let headers =
        context.auth_headers_for_url(Ecosystem::Npm, "https://registry.example.test/left-pad");

    assert!(headers.is_empty());

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn deno_npm_imports_use_document_npmrc_registry() {
    let root = std::env::temp_dir().join(format!("versionlens-deno-npmrc-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "registry=https://registry.example.test/\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("deno.json").display()),
        language_id: "jsonc".to_owned(),
        text: r#"{"imports":{"chalk":"npm:chalk@5.3.0"}}"#.to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.example.test/chalk"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn pnpm_yaml_dependencies_use_document_npmrc_registry() {
    let root = std::env::temp_dir().join(format!("versionlens-pnpm-npmrc-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".npmrc"),
        "@scope:registry=https://scope.example.test/npm\nregistry=https://registry.example.test/\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("pnpm-workspace.yaml").display()),
        language_id: "yaml".to_owned(),
        text: "catalog:\n  '@scope/pkg': ^1.0.0\n  left-pad: ^1.0.0\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = RegistryContext::from_document(&input);
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

    std::fs::remove_dir_all(root).unwrap();
}
