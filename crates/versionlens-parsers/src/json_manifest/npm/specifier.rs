use crate::support;
pub(in crate::json_manifest) fn alias_dependency(
    value: &str,
    value_start: usize,
) -> Option<(&str, &str, usize)> {
    let spec = value.strip_prefix("npm:")?;
    let Some(at) = spec.rfind('@').filter(|index| *index > 0) else {
        return support::valid_package_alias_name(spec).then_some((
            spec,
            "",
            value_start + value.len(),
        ));
    };
    let name = &spec[..at];
    let requirement = &spec[at + 1..];
    if requirement.is_empty() {
        return None;
    }
    Some((
        name,
        requirement,
        value_start + value.len() - requirement.len(),
    ))
}

pub(in crate::json_manifest) fn parse_package_manager(input: &str) -> Option<(&str, &str)> {
    let (name, requirement) = input.split_once('@')?;
    if name.is_empty()
        || requirement.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return None;
    }
    Some((name, requirement))
}

pub(in crate::json_manifest) fn string_requirement(value: &str) -> String {
    if let Some(path) = value.strip_prefix("link:") {
        return format!("file:{path}/package.json");
    }
    value.to_owned()
}
