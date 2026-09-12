use versionlens_model::{DocumentInput, Ecosystem, VersionableKind};

use super::{apply_edits, test_session};
use crate::RegistryResponseInput;
use crate::workspace::tests::support::TestWorkspace;

#[test]
fn nimble_equality_edit_replaces_the_complete_constraint() {
    let session = test_session();
    let text = "requires \"foo == 1.0.0\"\n";
    let input = DocumentInput::new("file:///coverage/demo.nimble", "nim", text, None);
    let response = RegistryResponseInput::new("foo", Ecosystem::Nim, r#"[{"name":"2.0.0"}]"#);
    let output = session.resolve_document_with_responses(input, &[response]);
    assert_eq!(
        apply_edits(text, &output.edits),
        "requires \"foo == 2.0.0\"\n"
    );
}

#[test]
fn luarocks_equality_edit_replaces_the_complete_constraint() {
    let session = test_session();
    let text = "package = \"demo\"\nversion = \"1.0.0-1\"\ndependencies = { \"foo == 1.0.0\" }\n";
    let input = DocumentInput::new("file:///coverage/demo.rockspec", "lua", text, None);
    let response = RegistryResponseInput::new(
        "foo",
        Ecosystem::LuaRocks,
        r#"repository = { ["foo"] = { ["1.0.0-1"] = {}, ["2.0.0-1"] = {} } }"#,
    );
    let output = session.apply_command_with_selected_version(super::super::ApplyCommandRequest {
        input,
        command: Some("update"),
        dependency_name: Some("foo"),
        selected_version: Some("2.0.0-1"),
        responses: &[response],
    });
    assert_eq!(output.edits.len(), 1);
    assert_eq!(
        apply_edits(text, &output.edits),
        "package = \"demo\"\nversion = \"1.0.0-1\"\ndependencies = { \"foo == 2.0.0-1\" }\n"
    );
}

#[test]
fn dune_equality_edits_preserve_operator_and_quotes() {
    let cases = [
        (
            "(lang dune 3.17)\n(package (name demo) (depends (foo (= 1.0.0))))\n",
            "(lang dune 3.17)\n(package (name demo) (depends (foo (= 2.0.0))))\n",
        ),
        (
            "(lang dune 3.17)\n(package (name demo) (depends (foo (= \"1.0.0\"))))\n",
            "(lang dune 3.17)\n(package (name demo) (depends (foo (= \"2.0.0\"))))\n",
        ),
    ];
    for (text, expected) in cases {
        let session = test_session();
        let input = DocumentInput::new("file:///coverage/dune-project", "plaintext", text, None);
        let response = RegistryResponseInput::new(
            "foo",
            Ecosystem::Opam,
            "<h2>foo version</h2><p>2.0.0 (latest)</p>",
        );
        let output = session.resolve_document_with_responses(input, &[response]);
        assert_eq!(output.edits.len(), 1);
        assert_eq!(apply_edits(text, &output.edits), expected);
    }
}

#[test]
fn gradle_catalog_inline_version_edit_replaces_the_value() {
    let session = test_session();
    let text = "[libraries]\nfoo = { module = \"example.test:foo\", version = \"1.0.0\" }\n";
    let input = DocumentInput::new(
        "file:///coverage/gradle/libs.versions.toml",
        "toml",
        text,
        None,
    );
    let response = RegistryResponseInput::new(
        "example.test:foo",
        Ecosystem::Maven,
        "<metadata><versioning><versions><version>1.0.0</version><version>2.0.0</version></versions></versioning></metadata>",
    );
    let output = session.apply_command_with_selected_version(super::super::ApplyCommandRequest {
        input,
        command: Some("update"),
        dependency_name: Some("example.test:foo"),
        selected_version: Some("2.0.0"),
        responses: &[response],
    });
    assert_eq!(output.edits.len(), 1);
    assert_eq!(
        apply_edits(text, &output.edits),
        "[libraries]\nfoo = { module = \"example.test:foo\", version = \"2.0.0\" }\n"
    );
}

#[test]
fn leiningen_project_version_is_local() {
    let session = test_session();
    let text = "(defproject demo \"0.1.0\" :dependencies [[example.test/foo \"1.0.0\"]])\n";
    let input = DocumentInput::new("file:///coverage/project.clj", "clojure", text, None);
    let project = session
        .dependencies(&input)
        .into_iter()
        .find(|dependency| dependency.name == "demo")
        .expect("Leiningen project version must parse");
    assert_eq!(project.versionable_kind(), VersionableKind::ProjectVersion);

    let output = session.resolve_document(input);
    let suggestion = output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "demo")
        .expect("Leiningen project version must resolve locally");
    assert_eq!(suggestion.status, "updateAvailable");
    assert_eq!(suggestion.latest.as_deref(), Some("0.1.1"));
}

#[test]
fn cargo_inherited_version_edits_only_the_workspace_root() {
    let workspace = TestWorkspace::new("cargo-inheritance");
    let root_text =
        "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nversion = \"1.0.0\"\n";
    let member_text = "[package]\nname = \"member\"\nversion.workspace = true\n";
    let input = |relative: &str, text: &str, version| {
        DocumentInput::new(
            crate::workspace_file_uri(&workspace.path().join(relative)).unwrap(),
            "toml",
            text,
            Some(workspace.path().to_string_lossy().into_owned()),
        )
        .with_version(version)
    };
    let root = input("Cargo.toml", root_text, 1);
    let member = input("member/Cargo.toml", member_text, 2);
    let session = test_session();
    assert!(session.set_workspace_documents(vec![root.clone(), member.clone()]));

    let root_dependency = session.dependencies(&root).remove(0);
    assert_eq!(
        root_dependency.versionable_kind(),
        VersionableKind::ProjectVersion
    );
    let root_output = session.resolve_document(root);
    let root_suggestion = root_output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "version")
        .expect("workspace version must resolve locally");
    assert_eq!(root_suggestion.status, "updateAvailable");
    assert_eq!(root_suggestion.latest.as_deref(), Some("1.0.1"));
    assert_eq!(root_output.edits.len(), 1);
    assert_eq!(
        apply_edits(root_text, &root_output.edits),
        "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nversion = \"1.0.1\"\n"
    );

    let member_dependency = session.dependencies(&member).remove(0);
    assert_eq!(member_dependency.requirement, "workspace:true");
    assert_eq!(
        member_dependency.versionable_kind(),
        VersionableKind::WorkspaceReference
    );
    let member_output = session.resolve_document(member);
    assert!(member_output.edits.is_empty());
    let suggestion = member_output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == "version")
        .expect("inherited version must remain visible");
    assert_eq!(suggestion.status, "fixed");
    assert_eq!(suggestion.latest.as_deref(), Some("workspace:true"));
}
