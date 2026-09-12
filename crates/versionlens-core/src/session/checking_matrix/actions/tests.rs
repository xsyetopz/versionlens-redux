use super::{FIXTURE_ROOT, runtime_responses};
use crate::ApplyCommandRequest;
use crate::session::checking_matrix::{WorkspaceDiscoveryOptions, apply_edits, test_session};
use crate::support::tests::fixture;
use crate::workspace::tests::support::TestWorkspace;

#[test]
fn github_action_runtime_pins_complete_the_workspace_checking_pipeline() {
    let pinned = fixture(FIXTURE_ROOT, "runtime-pins.yml");
    let workspace = TestWorkspace::new("actions-matrix");
    workspace.write(".github/workflows/ci.yml", &pinned);
    let session = test_session();
    let inputs = session
        .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
            workspace.path().to_owned(),
        ]))
        .collect::<Result<Vec<_>, _>>()
        .expect("the workflow must be discovered without failures");
    assert_eq!(inputs.len(), 1);
    let input = &inputs[0];
    assert_eq!(input.text, pinned);

    let dependencies = session.dependencies(input);
    for (name, requirement, group) in [
        ("node", "20.0.0", "with.node-version"),
        ("bun", "1.0.0", "with.bun-version"),
        ("rust", "1.80.0", "with.toolchain"),
    ] {
        assert!(
            dependencies.iter().any(|dependency| {
                dependency.name == name
                    && dependency.requirement == requirement
                    && dependency.group == group
            }),
            "missing {name} {requirement} from {dependencies:?}"
        );
    }

    let responses = runtime_responses();
    let resolved = session.resolve_document_with_responses(input.clone(), &responses);
    for name in ["node", "bun", "rust"] {
        let suggestion = resolved
            .suggestions
            .iter()
            .find(|suggestion| suggestion.dependency.name == name)
            .unwrap_or_else(|| panic!("missing resolved {name}: {resolved:?}"));
        assert_eq!(suggestion.status, "updateAvailable", "{name}");
    }
    assert_eq!(
        apply_edits(&pinned, &resolved.edits),
        fixture(FIXTURE_ROOT, "runtime-pins-updated.yml")
    );
    assert!(session.document_is_fresh(input));

    let cached = session.analyze_document(input.clone());
    assert!(cached.is_supported_manifest);
    assert!(cached.status.visible);
    assert!(cached.status.update_count >= 3);
    assert!(
        cached
            .code_lenses
            .iter()
            .filter(|lens| !lens.command.is_empty())
            .count()
            >= 3
    );
}

#[test]
fn github_action_runtime_constraints_never_raise_the_declared_minimum() {
    let constrained = fixture(FIXTURE_ROOT, "runtime-constraints.yml");
    let workspace = TestWorkspace::new("actions-constraints");
    workspace.write(".github/workflows/constraints.yml", &constrained);
    let session = test_session();
    let input = session
        .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
            workspace.path().to_owned(),
        ]))
        .next()
        .expect("the workflow must be discovered")
        .expect("the workflow must be readable");
    let responses = runtime_responses();
    let resolved = session.resolve_document_with_responses(input.clone(), &responses);

    assert!(resolved.edits.is_empty(), "{resolved:?}");
    for (name, minimum) in [("node", ">=20"), ("bun", "^1.0"), ("rust", ">=1.80")] {
        let suggestion = resolved
            .suggestions
            .iter()
            .find(|suggestion| {
                suggestion.dependency.name == name && suggestion.dependency.requirement == minimum
            })
            .unwrap_or_else(|| panic!("missing constrained {name}: {resolved:?}"));
        assert_eq!(suggestion.status, "satisfiesLatest", "{name}");
        let forced = session.apply_command_with_selected_version(ApplyCommandRequest {
            input: input.clone(),
            command: Some("update"),
            dependency_name: Some(name),
            selected_version: Some("99.0.0"),
            responses: &responses,
        });
        assert!(forced.edits.is_empty(), "{name}: {forced:?}");
    }
    assert!(session.document_is_fresh(&input));
    let cached = session.analyze_document(input);
    assert!(cached.status.visible);
    assert!(
        cached
            .code_lenses
            .iter()
            .all(|lens| lens.command.is_empty())
    );
}
