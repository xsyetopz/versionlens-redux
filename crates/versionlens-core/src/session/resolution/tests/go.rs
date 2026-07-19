use super::{DocumentInput, RegistryResponseInput, standard_session};
use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use versionlens_model::Ecosystem::Go;

#[test]
fn go_mod_exclude_versions_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///go.mod".to_owned(),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture(
                "go-mod-exclude-versions-are-fixed-without-registry-updates.mod",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/bad".to_owned(),
            ecosystem: Go,
            body: "v1.0.0\nv1.1.0\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("excluded version")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn go_mod_uses_workspace_go_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goproxy-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "GOPROXY=https://proxy.example.test/,direct|https://fallback.example.test|off\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-mod-uses-workspace-go-proxy-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://proxy.example.test/!go.uber.org/!zap/@v/list",
            "https://proxy.example.test/!go.uber.org/!zap/@latest",
            "https://fallback.example.test/!go.uber.org/!zap/@v/list",
            "https://fallback.example.test/!go.uber.org/!zap/@latest",
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_goproxy_off_disables_default_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goproxy-off-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".env"), "GOPROXY=off\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-mod-goproxy-off-disables-default-proxy-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_goproxy_direct_disables_default_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goproxy-direct-{}", id()));
    create_dir_all(&root).unwrap();
    write(root.join(".env"), "GOPROXY=direct\n").unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-mod-goproxy-direct-disables-default-proxy-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_goprivate_dependencies_do_not_use_module_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goprivate-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=corp.example.com\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture(
            "go-mod-goprivate-dependencies-do-not-use-module-proxy-urls.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_goprivate_path_match_patterns_disable_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goprivate-pattern-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "GOPROXY=https://proxy.golang.org,direct\nGOPRIVATE=[a-z].corp.example.com\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-mod-goprivate-path-match-patterns-disable-proxy-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn go_mod_goproxy_off_in_list_disables_later_proxy_urls() {
    let root = temp_dir().join(format!("versionlens-goproxy-off-list-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "GOPROXY=off,https://proxy.example.test\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("go.mod").display()),
        language_id: "go.mod".to_owned(),
        text: package_file_fixture("go-mod-goproxy-off-in-list-disables-later-proxy-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/go")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
