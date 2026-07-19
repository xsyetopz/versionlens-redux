use jsonc_parser::ast::Value::Object as JsonValueObject;
use jsonc_parser::errors::ParseError as JsonParseError;
use jsonc_parser::parse_to_ast;
use versionlens_model::Ecosystem;

use versionlens_model::Dependency;

use super::collect::{JsonManifestContext, collect_json_path};

pub(super) fn parse_json_manifest(
    text: &str,
    dependency_paths: &[&str],
    ecosystem: Ecosystem,
) -> Result<Vec<Dependency>, JsonParseError> {
    let parse_result = parse_to_ast(text, &crate::default(), &crate::default())?;
    let Some(JsonValueObject(root)) = parse_result.value else {
        return Ok(vec![]);
    };

    let mut dependencies = vec![];
    let context = JsonManifestContext {
        text,
        root: &root,
        ecosystem,
    };
    for path in dependency_paths {
        collect_json_path(&context, path, &mut dependencies);
    }
    Ok(dependencies)
}
