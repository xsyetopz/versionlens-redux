use std::fs;

use serde_json::{Value, json};
use versionlens_model::{WorkspaceEditPlan, document_text_hash};

use super::VersionLensLspState;

impl VersionLensLspState {
    pub(crate) fn workspace_edit(&self, plan: &WorkspaceEditPlan) -> Result<Value, String> {
        let mut current = Vec::with_capacity(plan.documents.len());
        let mut changes = Vec::with_capacity(plan.documents.len());
        for document in &plan.documents {
            let uri = &document.document.uri;
            let (edit_uri, version, model_version, text) =
                if let Some(work) = self.document_work(uri) {
                    (work.uri, work.version, work.input.version, work.input.text)
                } else {
                    let path =
                        versionlens_core::workspace_path(uri).ok_or("unsupported document URI")?;
                    let text = fs::read_to_string(path)
                        .map_err(|error| format!("cannot validate {uri}: {error}"))?;
                    (uri.clone(), None, None, text)
                };
            current.push((uri.clone(), model_version, document_text_hash(&text)));
            changes.push(
                json!({"textDocument":{"uri":edit_uri,"version":version},"edits":document.edits}),
            );
        }
        versionlens_core::validate_workspace_edit_plan(plan, &current).map_err(str::to_owned)?;
        Ok(json!({"documentChanges":changes}))
    }
}
