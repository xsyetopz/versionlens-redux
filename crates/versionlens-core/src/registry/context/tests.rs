use super::*;
use versionlens_parsers::classify_document;

impl RegistryContext {
    pub(crate) fn from_document(input: &DocumentInput) -> Self {
        let kind = classify_document(input);
        Self::from_document_kind(input, kind)
    }
}
