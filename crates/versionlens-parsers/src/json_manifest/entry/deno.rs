use crate::model::Dependency;

use crate::json_manifest::{
    deno::parse_deno_imports,
    paths::{DENO_DEPENDENCY_PATHS, dependency_paths},
};

pub(crate) fn parse_deno_json_with_paths(text: &str, paths: &[&str]) -> Vec<Dependency> {
    parse_deno_imports(text, dependency_paths(paths, DENO_DEPENDENCY_PATHS)).unwrap_or_default()
}
