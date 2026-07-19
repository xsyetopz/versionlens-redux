pub(super) fn go_registry_url(name: &str) -> String {
    format!("https://proxy.golang.org/{}/@v/list", go_base_module(name))
}

pub(super) fn go_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!(
        "{}/{}/@v/list",
        super::trim_end_slash(base_url),
        go_base_module(name)
    )
}

pub(super) fn go_base_module(name: &str) -> String {
    let mut encoded: String = crate::default();
    for ch in name.chars() {
        if ch.is_ascii_uppercase() {
            encoded.push('!');
            encoded.push(ch.to_ascii_lowercase());
        } else {
            encoded.push(ch);
        }
    }
    encoded
}
