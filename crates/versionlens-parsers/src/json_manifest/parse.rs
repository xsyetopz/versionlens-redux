use jsonc_parser::ast::Value;
use jsonc_parser::{CollectOptions, ParseOptions, parse_to_ast};

use crate::model::{Dependency, Ecosystem};

use super::collect::{JsonManifestContext, collect_json_path};

pub(super) fn parse_json_manifest(
    text: &str,
    dependency_paths: &[&str],
    ecosystem: Ecosystem,
) -> Result<Vec<Dependency>, jsonc_parser::errors::ParseError> {
    let parse_result = parse_to_ast(text, &CollectOptions::default(), &ParseOptions::default())?;
    let Some(Value::Object(root)) = parse_result.value else {
        return Ok(Vec::new());
    };

    let mut dependencies = Vec::new();
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
