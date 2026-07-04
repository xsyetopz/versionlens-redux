use super::super::encoding::encode_component;
use super::super::trim_end_slash;

pub(in crate::registry::urls) fn dub_registry_url(name: &str) -> String {
    format!(
        "https://code.dlang.org/api/packages/{}/info?minimize=true",
        encode_component(name)
    )
}

pub(in crate::registry::urls) fn dub_registry_url_with_base(base_url: &str, name: &str) -> String {
    format!(
        "{}/{}/info?minimize=true",
        trim_end_slash(base_url),
        encode_component(name)
    )
}
