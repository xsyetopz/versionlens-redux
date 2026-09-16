use lsp_types::{TextDocumentSyncCapability, TextDocumentSyncKind, Uri, WorkspaceFolder};

use super::{DISPLAY_CODE_LENS_COMMAND, VersionLensLspState, VersionLensTextDocument};

type FileTestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn capabilities_match_implemented_document_and_command_protocol() -> Result<(), &'static str> {
    let capabilities = VersionLensLspState::server_capabilities();
    let Some(TextDocumentSyncCapability::Options(sync)) = capabilities.text_document_sync else {
        return Err("expected explicit text document sync options");
    };
    assert_eq!(sync.open_close, Some(true));
    assert_eq!(sync.change, Some(TextDocumentSyncKind::FULL));
    let Some(code_lens) = capabilities.code_lens_provider else {
        return Err("expected CodeLens capabilities");
    };
    assert_eq!(code_lens.resolve_provider, Some(false));
    let Some(execute_command) = capabilities.execute_command_provider else {
        return Err("expected execute command capabilities");
    };
    assert_eq!(
        execute_command.commands,
        [
            DISPLAY_CODE_LENS_COMMAND,
            crate::state::UPDATE_DEPENDENCY_COMMAND
        ]
    );
    Ok(())
}

#[test]
fn selects_the_deepest_workspace_folder_and_decodes_its_path() -> Result<(), String> {
    let root_uri = uri("file:///workspace")?;
    let workspace_folders = vec![
        folder("file:///workspace/project", "project")?,
        folder("file:///workspace/project/nested%20folder", "nested")?,
    ];
    let mut state = VersionLensLspState::with_workspace(Some(root_uri), workspace_folders);
    let document_uri = "file:///workspace/project/nested%20folder/package.json";

    state.open_document(document(document_uri, None));

    assert_eq!(
        state
            .documents
            .get(document_uri)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("file:///workspace/project/nested%20folder")
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn workspace_roots_match_canonical_document_uris_and_new_files() -> FileTestResult {
    use versionlens_model::{
        DocumentEditPlan, DocumentSnapshot, WorkspaceEditPlan, document_text_hash,
    };

    let directory =
        versionlens_test_support::temporary_directory("versionlens-lsp-roots")?.canonicalize()?;
    let real = directory.join("real");
    let alias = directory.join("alias");
    std::fs::create_dir(&real)?;
    std::os::unix::fs::symlink(&real, &alias)?;
    let root_uri = format!("file://{}", alias.display());
    let mut state = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), vec![]);
    let document_uri = format!("file://{}/package.json", alias.display());
    let canonical_uri = format!("file://{}/package.json", real.display());
    state.open_document(document(&document_uri, None));
    let work = state.document_work(&document_uri).unwrap();
    assert_eq!(
        work.input.workspace_root.as_deref(),
        versionlens_core::workspace_file_uri(&real).as_deref()
    );
    assert_eq!(work.input.uri, canonical_uri);
    assert_eq!(
        state.document_work(&canonical_uri).unwrap().uri,
        document_uri
    );
    let edit = state.workspace_edit(&WorkspaceEditPlan {
        documents: vec![DocumentEditPlan {
            document: DocumentSnapshot {
                uri: canonical_uri,
                version: None,
                text_hash: document_text_hash("{}"),
            },
            edits: vec![],
        }],
    })?;
    assert_eq!(
        edit["documentChanges"][0]["textDocument"]["uri"],
        document_uri
    );
    std::fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn root_uri_is_used_only_for_documents_inside_it() -> Result<(), String> {
    let mut state =
        VersionLensLspState::with_workspace(Some(uri("file:///workspace")?), Vec::new());
    let inside = "file:///workspace/package.json";
    let outside = "file:///workspace-other/package.json";

    state.open_document(document(inside, None));
    state.open_document(document(outside, None));

    assert_eq!(
        state
            .documents
            .get(inside)
            .and_then(|document| document.workspace_root.as_deref()),
        Some("file:///workspace")
    );
    assert_eq!(
        state
            .documents
            .get(outside)
            .and_then(|document| document.workspace_root.as_deref()),
        None
    );
    Ok(())
}

#[test]
fn existing_file_workspace_root_uses_its_parent_directory() -> FileTestResult {
    let directory = versionlens_test_support::temporary_directory("versionlens-lsp-file-root")?
        .canonicalize()?;
    let manifest = directory.join("package.json");
    std::fs::write(&manifest, "{}")?;
    let manifest_uri = versionlens_core::workspace_file_uri(&manifest).ok_or("missing file URI")?;
    let directory_uri =
        versionlens_core::workspace_file_uri(&directory).ok_or("missing directory URI")?;
    let mut state = VersionLensLspState::with_workspace(Some(uri(&manifest_uri)?), Vec::new());

    state.open_document(document(&manifest_uri, None));

    assert_eq!(
        state
            .document_work(&manifest_uri)
            .unwrap()
            .input
            .workspace_root,
        Some(directory_uri)
    );
    assert_eq!(
        state.workspace_checking_options().discovery.roots,
        [directory]
    );
    Ok(())
}

#[test]
fn workspace_folder_changes_rebind_open_documents_and_invalidate_work() -> Result<(), String> {
    let parent = folder("file:///workspace", "parent")?;
    let nested = folder("file:///workspace/nested%20folder", "nested")?;
    let mut state =
        VersionLensLspState::with_workspace(Some(parent.uri.clone()), vec![parent.clone()]);
    let document_uri = "file:///workspace/nested%20folder/package.json";
    state.open_document(document(document_uri, None));
    state.set_document_version(document_uri, 7);
    for (added, removed, expected) in [
        (
            vec![nested.clone()],
            vec![],
            Some("file:///workspace/nested%20folder"),
        ),
        (vec![], vec![nested], Some("file:///workspace")),
        (vec![], vec![parent], None),
    ] {
        let before = state
            .document_work(document_uri)
            .ok_or("missing open document")?;
        state.change_workspace_folders(lsp_types::WorkspaceFoldersChangeEvent { added, removed });
        let after = state
            .document_work(document_uri)
            .ok_or("missing updated document")?;
        assert!(!state.work_is_current(&before));
        assert_eq!(after.input.workspace_root.as_deref(), expected);
        assert_eq!(after.version, Some(7));
    }
    Ok(())
}

#[test]
fn code_lenses_bind_updates_to_the_document_and_close_with_it() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    state.open_document(VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: r#"{"version":"1.0.0"}"#.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });

    let lenses = state.code_lenses(uri);
    crate::test_support::assert_update_code_lenses(&lenses);

    state.close_document(uri);
    assert!(state.code_lenses(uri).is_empty());
    assert!(!state.documents.contains_key(uri));
}

#[tokio::test]
async fn dependency_lenses_share_the_meaningful_source_range() {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    let text = r#"{"dependencies":{"example":"1.0.0"}}"#;
    state.open_document(VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: Some("/workspace".to_owned()),
    });
    let work = state.document_work(uri).unwrap();
    state
        .session
        .resolve_document_with_responses(
            work.input,
            &[versionlens_core::RegistryResponseInput::new(
                "example",
                versionlens_model::Ecosystem::Npm,
                r#"{"dist-tags":{"latest":"2.0.0"},"versions":{"1.0.0":{},"2.0.0":{}}}"#,
            )],
        )
        .await;

    let lenses = state.code_lenses(uri);

    assert!(lenses.len() >= 2);
    assert!(lenses.iter().all(|lens| {
        lens.range
            == lsp_types::Range::new(
                lsp_types::Position::new(0, 18),
                lsp_types::Position::new(0, 33),
            )
    }));
}

#[test]
fn changes_unknown_document_without_diagnostics() {
    let mut state = VersionLensLspState::standard();
    assert!(
        state
            .change_document("file:///missing/package.json", "{}".to_owned())
            .is_empty()
    );
}

fn uri(value: &str) -> Result<Uri, String> {
    value.parse::<Uri>().map_err(|error| error.to_string())
}

fn folder(value: &str, name: &str) -> Result<WorkspaceFolder, String> {
    Ok(WorkspaceFolder {
        uri: uri(value)?,
        name: name.to_owned(),
    })
}

fn document(uri: &str, workspace_root: Option<String>) -> VersionLensTextDocument {
    VersionLensTextDocument {
        uri: uri.to_owned(),
        language_id: "json".to_owned(),
        text: "{}".to_owned(),
        workspace_root,
    }
}

#[test]
fn configuration_changes_invalidate_document_work_and_preserve_versions() -> Result<(), String> {
    let mut state = VersionLensLspState::standard();
    let uri = "file:///workspace/package.json";
    state.open_document(document(uri, None));
    state.set_document_version(uri, 9);
    let work = state.document_work(uri).ok_or("missing document")?;
    let config = super::session_configuration(
        serde_json::json!({"showVulnerabilities":false,"http":{"timeoutMs":250}}),
    )?;
    assert!(!config.show_vulnerabilities);
    assert_eq!(config.http.timeout_ms, 250);
    assert!(config.http.strict_ssl);
    assert_eq!(
        super::session_configuration(serde_json::json!({"cacheDurationMinutes": 2.0}))?
            .cache_ttl_ms,
        120_000
    );
    state
        .replace_configuration(config)
        .map_err(|error| error.to_string())?;
    assert!(!state.work_is_current(&work));
    assert_eq!(
        state.document_work(uri).ok_or("missing document")?.version,
        Some(9)
    );
    assert!(!state.accepts_document_version(uri, 9));
    assert!(state.accepts_document_version(uri, 10));
    assert!(
        super::session_configuration(serde_json::json!({"http":{"timeoutMs":"invalid"}})).is_err()
    );
    Ok(())
}

#[test]
fn workspace_edits_validate_every_open_document_before_application() {
    use versionlens_model::{
        DocumentEditPlan, DocumentSnapshot, WorkspaceEditPlan, document_text_hash,
    };
    let mut state = VersionLensLspState::standard();
    let uris = [
        "file:///workspace/a/package.json",
        "file:///workspace/b/package.json",
    ];
    for uri in uris {
        state.open_document(document(uri, None));
        state.set_document_version(uri, 1);
    }
    let plan = WorkspaceEditPlan {
        documents: uris
            .into_iter()
            .map(|uri| DocumentEditPlan {
                document: DocumentSnapshot {
                    uri: uri.to_owned(),
                    version: Some(1),
                    text_hash: document_text_hash("{}"),
                },
                edits: vec![],
            })
            .collect(),
    };
    assert!(state.workspace_edit(&plan).is_ok());
    state.change_document(uris[1], "{\"version\":\"2.0.0\"}".to_owned());
    state.set_document_version(uris[1], 2);
    assert!(state.workspace_edit(&plan).is_err());
}

#[test]
fn workspace_snapshot_changes_invalidate_other_open_document_work() {
    let root = std::env::temp_dir().canonicalize().unwrap();
    let mut state = VersionLensLspState::standard();
    let paths = [
        "versionlens-lsp-a/package.json",
        "versionlens-lsp-b/package.json",
    ]
    .map(|name| root.join(name).to_string_lossy().into_owned());
    for path in &paths {
        state.open_document(document(path, Some(root.to_string_lossy().into_owned())));
    }
    let work = state.document_work(&paths[0]).unwrap();
    state.change_document(&paths[1], "{\"version\":\"2.0.0\"}".to_owned());
    assert!(!state.work_is_current(&work));
    let work = state.document_work(&paths[0]).unwrap();
    state.close_document(&paths[1]);
    assert!(!state.work_is_current(&work));
    let work = state.document_work(&paths[0]).unwrap();
    state.invalidate_workspace();
    assert!(!state.work_is_current(&work));
}

#[test]
fn editor_exclusions_expand_braces_and_filter_watched_files() -> Result<(), String> {
    let root = std::env::temp_dir()
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let root_uri = versionlens_core::workspace_file_uri(&root).ok_or("missing root URI")?;
    let state = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_workspace_exclusions(super::workspace_exclusions(&serde_json::json!({
            "files": {"exclude": {"**/{generated,cache}/**": true}}
        })));
    let included = root.join("src/package.json");
    let cargo = root.join("src/Cargo.toml");
    let configured = root.join("src/deps.custom");
    let unrelated = root.join("src/readme.md");
    let generated = root.join("generated/package.json");
    let cache = root.join("cache/package.json");
    let file_uri = |path: &std::path::Path| {
        versionlens_core::workspace_file_uri(path).ok_or_else(|| "missing file URI".to_owned())
    };

    assert!(state.watched_file_is_relevant(&file_uri(&included)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&unrelated)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&generated)?));
    assert!(!state.watched_file_is_relevant(&file_uri(&cache)?));
    let options = state.workspace_checking_options();
    assert_eq!(options.discovery.provider_exclusions.len(), 1);
    assert_eq!(
        options.discovery.provider_exclusions[0].ecosystem,
        versionlens_model::Ecosystem::Dotnet
    );
    assert_eq!(
        options.discovery.provider_exclusions[0].patterns,
        ["**/obj/**"]
    );

    let cargo_only = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_config(super::session_configuration(serde_json::json!({
            "enabledProviders": ["cargo"]
        }))?);
    assert!(cargo_only.watched_file_is_relevant(&file_uri(&cargo)?));
    assert!(!cargo_only.watched_file_is_relevant(&file_uri(&included)?));

    let configured_npm = VersionLensLspState::with_workspace(Some(uri(&root_uri)?), Vec::new())
        .with_config(super::session_configuration(serde_json::json!({
            "enabledProviders": ["npm"],
            "providers": {
                "filePatterns": [{"ecosystem": "npm", "pattern": "**/deps.custom"}]
            }
        }))?);
    assert!(configured_npm.watched_file_is_relevant(&file_uri(&configured)?));
    Ok(())
}
