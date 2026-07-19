use jsonc_parser::ast::Object;
use versionlens_model::Ecosystem;

use versionlens_model::Dependency;

mod path;
mod path_target;
mod wildcard;

use path::value_at_path;
use path_target::{JsonPathTargetContext, collect_json_path_target};
use wildcard::{collect_json_wildcard_path, collect_terminal_wildcard_path};

pub(super) struct JsonManifestContext<'a> {
    pub(super) text: &'a str,
    pub(super) root: &'a Object<'a>,
    pub(super) ecosystem: Ecosystem,
}

pub(super) fn collect_json_path(
    context: &JsonManifestContext<'_>,
    path: &str,
    out: &mut Vec<Dependency>,
) {
    let segments = path.split('.').collect::<Vec<_>>();
    let Some((last, parents)) = segments.split_last() else {
        return;
    };

    if *last == "*" {
        collect_terminal_wildcard_path(context, parents, out);
        return;
    }

    if let Some(star) = segments.iter().position(|segment| *segment == "*") {
        collect_json_wildcard_path(context, &segments, star, out);
        return;
    }

    let Some(target) = value_at_path(context.root, &segments) else {
        return;
    };

    collect_json_path_target(
        &JsonPathTargetContext {
            manifest: context,
            path,
            target,
            parents,
            last,
        },
        out,
    );
}
