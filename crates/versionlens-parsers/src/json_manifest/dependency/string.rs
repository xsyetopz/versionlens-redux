use jsonc_parser::ast::{ObjectProp, StringLit};

use crate::model::{Dependency, Ecosystem};

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
    if source.ecosystem == Ecosystem::Npm {
        return npm_string_json_manifest_dependency(source, prop, lit);
    }

    Some(string_literal_json_manifest_dependency(source, prop, lit))
}
