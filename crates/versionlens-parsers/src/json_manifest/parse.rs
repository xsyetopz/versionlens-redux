use crate::support;
use jsonc_parser::errors::ParseError as JsonParseError;
use versionlens_model::Ecosystem;

use versionlens_model::Dependency;

use super::collect::{JsonManifestContext, collect_json_path};

pub(super) fn parse_json_manifest(
    text: &str,
    dependency_paths: &[&str],
    ecosystem: Ecosystem,
) -> Result<Vec<Dependency>, JsonParseError> {
    Ok(support::try_with_json_object(text, |root| {
        let mut dependencies = vec![];
        let context = JsonManifestContext {
            text,
            root,
            ecosystem,
        };
        for path in dependency_paths {
            collect_json_path(&context, path, &mut dependencies);
        }
        dependencies
    })?
    .unwrap_or_default())
}
