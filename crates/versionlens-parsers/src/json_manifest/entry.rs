mod deno;
mod generic;
mod package;

pub(crate) use deno::parse_deno_json_with_paths;
pub(crate) use generic::{
    parse_composer_json_with_paths, parse_dotnet_project_json_with_paths, parse_dub_json_with_paths,
};
pub(crate) use package::{looks_like_package_json, parse_package_json_with_paths};
