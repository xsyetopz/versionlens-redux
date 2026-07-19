use jsonc_parser::ast::{ObjectProp, StringLit};

use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Npm;

mod literal;
mod npm;

use literal::string_literal_json_manifest_dependency;
use npm::npm_string_json_manifest_dependency;

use super::JsonDependencySource;

pub(super) fn string_json_manifest_dependency(
    source: &JsonDependencySource<'_>,
    prop: &ObjectProp<'_>,
    lit: &StringLit<'_>,
) -> Option<Dependency> {
    if source.ecosystem == Npm {
        return npm_string_json_manifest_dependency(source, prop, lit);
    }

    Some(string_literal_json_manifest_dependency(source, prop, lit))
}
