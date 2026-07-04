pub(super) fn go_registry_url(name: &str) -> String {
    format!("https://proxy.golang.org/{}/@v/list", go_base_module(name))
}

pub(super) fn go_registry_url_with_base(base_url: &str, name: &str) -> String {
    base_url.replacen("{base-module}", &go_base_module(name), 1)
}

pub(super) fn go_base_module(name: &str) -> String {
    name.to_lowercase()
}
