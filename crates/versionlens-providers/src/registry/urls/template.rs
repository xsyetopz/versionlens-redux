use versionlens_parsers::Ecosystem;

use super::go::go_base_module;
use versionlens_parsers::Ecosystem::Go;

const NAME_TOKEN: &str = "{name}";
const BASE_MODULE_TOKEN: &str = "{base-module}";

pub(super) fn template_registry_url(
    ecosystem: Ecosystem,
    name: &str,
    base_url: &str,
) -> Option<String> {
    if !base_url.contains(NAME_TOKEN) && !base_url.contains(BASE_MODULE_TOKEN) {
        return None;
    }

    let base_module = base_module_name(ecosystem, name);
    Some(
        base_url
            .replace(NAME_TOKEN, name)
            .replace(BASE_MODULE_TOKEN, &base_module),
    )
}

fn base_module_name(ecosystem: Ecosystem, name: &str) -> String {
    if ecosystem == Go {
        return go_base_module(name);
    }

    name.to_lowercase()
}
