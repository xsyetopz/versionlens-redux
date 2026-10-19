use super::*;
use versionlens_model::{Dependency, Ecosystem, ManifestKind};

use crate::workspace::tests::support::{TestWorkspace, dependency};

const LOCAL_CONSUMER_TEXT: &str = r#"{
  "name": "consumer",
  "version": "1.0.0",
  "dependencies": {
    "local-package": "^1.0.0"
  }
}"#;

fn session() -> VersionLensSession {
    VersionLensSession::new(crate::support::tests::session_config(
        crate::default(),
        false,
    ))
}

fn cached_graphs(session: &VersionLensSession) -> usize {
    session
        .storage_state
        .workspace
        .lock()
        .unwrap_or_else(crate::recover_poison)
        .graphs
        .len()
}

fn assert_cached_graphs(session: &VersionLensSession, expected: usize) {
    assert_eq!(cached_graphs(session), expected);
}

fn registry_url(context: &RegistryContext, name: &str) -> String {
    context
        .registry_endpoints(&dependency(name, "1.0.0"))
        .into_iter()
        .next()
        .expect("registry endpoint")
        .url
}

fn npm_input(workspace: &TestWorkspace, relative: &str) -> DocumentInput {
    workspace.document(
        relative,
        r#"{"name":"app","dependencies":{"demo":"1.0.0"}}"#,
        1,
    )
}

fn npm_document(uri: String, workspace_root: String) -> DocumentInput {
    DocumentInput::new(
        uri,
        "json",
        r#"{"dependencies":{"demo":"1.0.0"}}"#,
        Some(workspace_root),
    )
}

fn rootless(mut input: DocumentInput) -> DocumentInput {
    let path = workspace_path(&input.uri).expect("rootless document path");
    input.uri = crate::workspace_file_uri(&path).expect("rootless document URI");
    input.workspace_root = None;
    input
}

fn npm_context(session: &VersionLensSession, input: &DocumentInput) -> RegistryContext {
    session.registry_context(input, ManifestKind::NpmPackageJson)
}

fn npm_context_with_documents(
    input: &DocumentInput,
    documents: Vec<DocumentInput>,
) -> RegistryContext {
    let session = session();
    assert!(session.set_workspace_documents(documents));
    npm_context(&session, input)
}

fn registered_member(
    text: &str,
    version: u64,
) -> (TestWorkspace, VersionLensSession, DocumentInput) {
    let workspace = TestWorkspace::new("session-workspace");
    let root = workspace.document("package.json", r#"{"workspaces":["packages/*"]}"#, 1);
    let member = workspace.document("packages/a/package.json", text, version);
    let session = session();
    session.set_workspace_documents(vec![root, member.clone()]);
    (workspace, session, member)
}

fn local_member_workspace(name: &str) -> (TestWorkspace, DocumentInput) {
    let workspace = TestWorkspace::new(name);
    workspace.write_all(&[
        ("package.json", r#"{"workspaces":["packages/*"]}"#),
        (
            "packages/local/package.json",
            r#"{"name":"local-package","version":"1.0.0"}"#,
        ),
        ("packages/consumer/package.json", LOCAL_CONSUMER_TEXT),
    ]);
    let mut consumer = workspace.document("packages/consumer/package.json", LOCAL_CONSUMER_TEXT, 1);
    consumer.uri =
        crate::workspace_file_uri(&workspace.root.join("packages/consumer/package.json")).unwrap();
    (workspace, consumer)
}

fn local_dependency(session: &VersionLensSession, consumer: &DocumentInput) -> Dependency {
    session
        .dependencies(consumer)
        .into_iter()
        .find(|dependency| dependency.name == "local-package")
        .unwrap()
}

fn assert_workspace_version(
    graph: &crate::workspace::WorkspaceGraph,
    name: &str,
    requirement: &str,
    expected: &str,
) {
    assert_eq!(
        graph
            .resolve(&dependency(name, requirement))
            .unwrap()
            .version,
        expected
    );
}

fn registry_configuration(
    workspace: &TestWorkspace,
    host: &str,
    version: u64,
    rootless_config: bool,
) -> DocumentInput {
    let configuration = workspace.document(
        ".npmrc",
        &format!("registry=https://{host}/\n//{host}/:_authToken={version}"),
        version,
    );
    if rootless_config {
        rootless(configuration)
    } else {
        configuration
    }
}

fn assert_registry_overlay_replacement(
    workspace: &TestWorkspace,
    input: &DocumentInput,
    hosts: [&str; 2],
    rootless_config: bool,
) {
    let session = session();
    let first = registry_configuration(workspace, hosts[0], 1, rootless_config);
    session.set_workspace_documents(vec![first]);
    let first_context = npm_context(&session, input);
    let first_scope = session.document_cache_scope(&first_context, input);

    let second = registry_configuration(workspace, hosts[1], 2, rootless_config);
    assert!(session.set_workspace_documents(vec![second]));
    let second_context = npm_context(&session, input);
    let second_scope = session.document_cache_scope(&second_context, input);

    assert!(registry_url(&first_context, "demo").starts_with(&format!("https://{}/", hosts[0])));
    assert!(registry_url(&second_context, "demo").starts_with(&format!("https://{}/", hosts[1])));
    assert_ne!(first_scope, second_scope);
    assert_ne!(
        first_context.auth_headers_for_url(Ecosystem::Npm, &format!("https://{}/demo", hosts[0]),),
        second_context.auth_headers_for_url(Ecosystem::Npm, &format!("https://{}/demo", hosts[1]),)
    );
}

fn assert_registry_size_failure(session: &VersionLensSession, input: &DocumentInput) {
    assert!(
        npm_context(session, input)
            .failure_message()
            .is_some_and(|message| message.contains("the limit is"))
    );
}

#[path = "tests/graphs.rs"]
mod graphs;

#[path = "tests/registry.rs"]
mod registry;
