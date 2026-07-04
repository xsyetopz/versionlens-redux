use napi_derive::napi;
use versionlens_parsers::DocumentInput;

#[napi(object)]
pub struct NativeDocumentInput {
    pub uri: String,
    pub language_id: String,
    pub text: String,
    pub workspace_root: Option<String>,
}

#[napi(object)]
pub struct NativeApplyCommandInput {
    pub document: NativeDocumentInput,
    pub command: Option<String>,
    pub dependency_name: Option<String>,
    pub selected_version: Option<String>,
}

impl NativeDocumentInput {
    pub(crate) fn into_core(self) -> DocumentInput {
        DocumentInput {
            uri: self.uri,
            language_id: self.language_id,
            text: self.text,
            workspace_root: self.workspace_root,
        }
    }
}

impl NativeApplyCommandInput {
    pub fn into_parts(
        self,
    ) -> (
        DocumentInput,
        Option<String>,
        Option<String>,
        Option<String>,
    ) {
        (
            self.document.into_core(),
            self.command,
            self.dependency_name,
            self.selected_version,
        )
    }
}

#[cfg(test)]
mod tests;
