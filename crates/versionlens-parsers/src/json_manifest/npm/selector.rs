pub(in crate::json_manifest) fn trim_selector(name: &str) -> &str {
    selector_trim_index(name).map_or(name, |index| &name[..index])
}

fn selector_trim_index(name: &str) -> Option<usize> {
    let index = name.find('@')?;
    (index > 0).then_some(index)
}
