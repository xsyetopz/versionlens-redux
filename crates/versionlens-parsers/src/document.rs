use crate::classify::classify_document;
use crate::model::{Dependency, DocumentInput, ManifestKind};

mod parsers;
#[cfg(test)]
pub(crate) mod test_support;

pub fn parse_document(input: &DocumentInput) -> Vec<Dependency> {
    parse_document_with_dependency_paths::<&str>(input, &[])
}

pub fn parse_document_with_dependency_paths<P: AsRef<str>>(
    input: &DocumentInput,
    dependency_paths: &[P],
) -> Vec<Dependency> {
    parse_document_as_manifest_with_dependency_paths(
        input,
        classify_document(input),
        dependency_paths,
    )
}

pub fn parse_document_as_manifest_with_dependency_paths<P: AsRef<str>>(
    input: &DocumentInput,
    kind: ManifestKind,
    dependency_paths: &[P],
) -> Vec<Dependency> {
    let paths = dependency_paths
        .iter()
        .map(AsRef::as_ref)
        .collect::<Vec<_>>();

    parsers::parse_manifest_kind(kind, &input.text, &paths)
}

#[cfg(test)]
mod tests;
