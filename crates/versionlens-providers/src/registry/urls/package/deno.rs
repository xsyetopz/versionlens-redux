pub(in crate::registry::urls) fn deno_registry_url(name: &str) -> String {
    format!("https://jsr.io/{name}/meta.json")
}

pub(in crate::registry::urls) fn deno_registry_url_with_base(
    _base_url: &str,
    name: &str,
) -> String {
    deno_registry_url(name)
}
