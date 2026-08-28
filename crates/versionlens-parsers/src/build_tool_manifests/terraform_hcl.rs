use crate::positions::offset_range;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::Terraform;

pub(crate) fn parse_terraform_hcl(text: &str) -> Vec<Dependency> {
    let mut dependencies = vec![];
    let Some(required_start) = text.find("required_providers") else {
        return dependencies;
    };
    let Some(open) = text[required_start..]
        .find('{')
        .map(|index| required_start + index)
    else {
        return dependencies;
    };
    let Some(close) = matching_brace(text, open) else {
        return dependencies;
    };
    let body_start = open + 1;
    let body = &text[body_start..close];
    let mut offset = 0usize;
    while offset < body.len() {
        let Some((key, key_start, value_start)) = next_assignment(body, offset) else {
            break;
        };
        let absolute_value_start = body_start + value_start;
        let value = body[value_start..].trim_start();
        let leading = body[value_start..].len() - value.len();
        let absolute_value_start = absolute_value_start + leading;
        if value.starts_with('{') {
            let block_open = absolute_value_start;
            if let Some(block_close) = matching_brace(text, block_open) {
                if let Some(dependency) = object_provider_dependency(
                    text,
                    key,
                    body_start + key_start,
                    block_open,
                    block_close + 1,
                ) {
                    dependencies.push(dependency);
                }
                offset = block_close.saturating_sub(body_start) + 1;
            } else {
                break;
            }
        } else if value.starts_with('"') {
            if let Some((requirement, value_inner_start, value_inner_end)) =
                quoted_value(text, absolute_value_start)
            {
                dependencies.push(provider_dependency(TerraformProviderDependency {
                    text,
                    name: &format!("hashicorp/{key}"),
                    hosted_name: Some(key),
                    requirement,
                    range_start: body_start + key_start,
                    value_start: value_inner_start,
                    value_end: value_inner_end,
                    hosted_url: None,
                }));
                offset = value_inner_end.saturating_sub(body_start) + 1;
            } else {
                break;
            }
        } else {
            offset = value_start + 1;
        }
    }

    dependencies
}

fn object_provider_dependency(
    text: &str,
    local_name: &str,
    key_start: usize,
    block_start: usize,
    block_end: usize,
) -> Option<Dependency> {
    let block = text.get(block_start..block_end)?;
    let source = attribute_string(block, block_start, "source");
    let version = attribute_string(block, block_start, "version");
    let source_value = source
        .as_ref()
        .map(|value| value.value.as_str())
        .unwrap_or_else(|| local_name);
    let package_name = terraform_provider_source(local_name, source_value);
    let hosted_url =
        (package_name == "terraform.io/builtin/terraform").then(|| "builtin".to_owned());
    if let Some(version) = version {
        return Some(provider_dependency(TerraformProviderDependency {
            text,
            name: &package_name,
            hosted_name: Some(local_name),
            requirement: &version.value,
            range_start: key_start,
            value_start: version.value_start,
            value_end: version.value_end,
            hosted_url,
        }));
    }
    Some(Dependency {
        name: package_name,
        requirement: "latest".to_owned(),
        ecosystem: Terraform,
        group: "required_providers".to_owned(),
        hosted_url,
        hosted_name: Some(local_name.to_owned()),
        range: offset_range(text, key_start, block_end),
        requirement_range: offset_range(text, block_end, block_end),
        requirement_prefix: " version = \"".to_owned(),
        requirement_suffix: "\"".to_owned(),
    })
}

struct TerraformProviderDependency<'a> {
    text: &'a str,
    name: &'a str,
    hosted_name: Option<&'a str>,
    requirement: &'a str,
    range_start: usize,
    value_start: usize,
    value_end: usize,
    hosted_url: Option<String>,
}

fn provider_dependency(input: TerraformProviderDependency<'_>) -> Dependency {
    let (requirement, requirement_prefix, version_start) =
        terraform_requirement_parts(input.requirement, input.value_start);
    Dependency {
        name: input.name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Terraform,
        group: "required_providers".to_owned(),
        hosted_url: input.hosted_url,
        hosted_name: input.hosted_name.map(|value| value.to_owned()),
        range: offset_range(input.text, input.range_start, input.value_end + 1),
        requirement_range: offset_range(input.text, version_start, input.value_end),
        requirement_prefix,
        requirement_suffix: "".to_owned(),
    }
}

fn terraform_provider_source(local_name: &str, source: &str) -> String {
    let source = source.trim();
    if source.contains('/') {
        source.to_owned()
    } else {
        format!("hashicorp/{local_name}")
    }
}

fn terraform_requirement_parts(requirement: &str, value_start: usize) -> (&str, String, usize) {
    let trimmed = requirement.trim_start();
    let leading = requirement.len() - trimmed.len();
    for operator in ["~>", ">=", "<=", "!=", "=", ">", "<"] {
        if let Some(rest) = trimmed.strip_prefix(operator) {
            let rest_trimmed = rest.trim_start();
            let whitespace = rest.len() - rest_trimmed.len();
            let version_start = value_start + leading + operator.len() + whitespace;
            return (trimmed, "".to_owned(), version_start);
        }
    }
    (trimmed, "".to_owned(), value_start + leading)
}

struct StringAttribute {
    value: String,
    value_start: usize,
    value_end: usize,
}

fn attribute_string(block: &str, block_offset: usize, name: &str) -> Option<StringAttribute> {
    let assignment = name.to_owned();
    let mut search = 0usize;
    while let Some(index) = block[search..].find(&assignment) {
        let attr_start = search + index;
        let after_name = attr_start + assignment.len();
        let after_ws = skip_ws(block, after_name);
        if block.as_bytes().get(after_ws) != Some(&b'=') {
            search = after_name;
            continue;
        }
        let value_start = skip_ws(block, after_ws + 1);
        let absolute = block_offset + value_start;
        let (value, inner_start, inner_end) =
            quoted_value(&block[value_start..], 0).map(|(value, start, end)| {
                (
                    value.to_owned(),
                    block_offset + value_start + start,
                    block_offset + value_start + end,
                )
            })?;
        return Some(StringAttribute {
            value,
            value_start: inner_start.max(absolute + 1),
            value_end: inner_end,
        });
    }
    None
}

fn next_assignment(source: &str, start: usize) -> Option<(&str, usize, usize)> {
    let mut offset = start;
    while offset < source.len() {
        offset = skip_ws(source, offset);
        let key_start = offset;
        while offset < source.len() {
            let byte = source.as_bytes()[offset];
            if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
                offset += 1;
            } else {
                break;
            }
        }
        if key_start == offset {
            offset += 1;
            continue;
        }
        let after_key = skip_ws(source, offset);
        if source.as_bytes().get(after_key) == Some(&b'=') {
            return source
                .get(key_start..offset)
                .map(|key| (key, key_start, after_key + 1));
        }
        offset = after_key + 1;
    }
    None
}

fn quoted_value(text: &str, quote_offset: usize) -> Option<(&str, usize, usize)> {
    if text.as_bytes().get(quote_offset) != Some(&b'"') {
        return None;
    }
    let start = quote_offset + 1;
    let end = start + text[start..].find('"')?;
    text.get(start..end).map(|value| (value, start, end))
}

fn matching_brace(text: &str, open: usize) -> Option<usize> {
    if text.as_bytes().get(open) != Some(&b'{') {
        return None;
    }
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;
    for (relative, byte) in text[open..].bytes().enumerate() {
        let offset = open + relative;
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn skip_ws(source: &str, mut offset: usize) -> usize {
    while offset < source.len() && source.as_bytes()[offset].is_ascii_whitespace() {
        offset += 1;
    }
    offset
}
