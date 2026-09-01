use super::*;
use versionlens_model::{Position, Range};

fn edit(start: u32, end: u32) -> TextEdit {
    TextEdit {
        range: Range {
            start: Position {
                line: 0,
                character: start,
            },
            end: Position {
                line: 0,
                character: end,
            },
        },
        new_text: String::new(),
    }
}

fn plan(edits: Vec<TextEdit>) -> WorkspaceEditPlan {
    WorkspaceEditPlan {
        documents: vec![DocumentEditPlan {
            document: DocumentSnapshot {
                uri: "file:///a".into(),
                version: Some(1),
                text_hash: "hash".into(),
            },
            edits,
        }],
    }
}

#[test]
fn validator_sorts_and_allows_adjacent_edits() {
    assert!(
        validate_workspace_edit_plan(
            &plan(vec![edit(2, 3), edit(0, 1)]),
            &[("file:///a".into(), Some(1), "hash".into())],
        )
        .is_ok()
    );
}

#[test]
fn validator_rejects_overlap_duplicate_and_stale_documents() {
    assert!(
        validate_workspace_edit_plan(
            &plan(vec![edit(0, 2), edit(1, 3)]),
            &[("file:///a".into(), Some(1), "hash".into())],
        )
        .is_err()
    );
    let mut duplicate = plan(vec![]);
    duplicate.documents.push(duplicate.documents[0].clone());
    assert!(
        validate_workspace_edit_plan(&duplicate, &[("file:///a".into(), Some(1), "hash".into())])
            .is_err()
    );
    assert!(
        validate_workspace_edit_plan(
            &plan(vec![]),
            &[("file:///a".into(), Some(2), "hash".into())]
        )
        .is_err()
    );
}
