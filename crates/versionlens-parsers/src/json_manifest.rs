mod collect;
mod deno;
mod dependency;
mod detect;
mod entry;
mod npm;
mod parse;
mod paths;

pub(crate) use entry::{
    looks_like_package_json, parse_composer_json_with_paths, parse_deno_json_with_paths,
    parse_dotnet_project_json_with_paths, parse_dub_json_with_paths, parse_package_json_with_paths,
};

#[cfg(test)]
mod tests;
