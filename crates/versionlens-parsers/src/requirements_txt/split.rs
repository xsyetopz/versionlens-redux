pub(crate) fn split_python_requirement(raw: &str) -> Option<(&str, &str, usize, &str)> {
    let value = strip_comment(raw).trim_end();
    if let Some((raw_name, requirement)) = value.split_once(" @ ") {
        let name = raw_name.trim().split('[').next().unwrap_or("").trim();
        if name.is_empty() {
            return None;
        }
        return Some((
            name,
            requirement.split(';').next().unwrap_or("").trim_end(),
            raw_name.len() + 3,
            "",
        ));
    }

    let requirement_part = value.split(';').next().unwrap_or("").trim_end();
    let split = requirement_part
        .find(['=', '<', '>', '~', '!'])
        .unwrap_or(requirement_part.len());
    let raw_name = requirement_part[..split].trim();
    let parenthesized = raw_name.ends_with('(') && requirement_part.ends_with(')');
    let name = raw_name
        .strip_suffix('(')
        .unwrap_or(raw_name)
        .trim_end()
        .split('[')
        .next()
        .unwrap_or("")
        .trim();
    if name.is_empty() {
        return None;
    }
    let requirement = requirement_part[split..]
        .strip_suffix(')')
        .filter(|_| parenthesized)
        .unwrap_or(&requirement_part[split..])
        .trim_end();
    Some((
        name,
        requirement,
        split,
        if parenthesized { ")" } else { "" },
    ))
}

pub(super) fn split_requirements_txt_requirement(raw: &str) -> Option<(&str, &str, usize)> {
    let value = strip_comment(raw).trim_end();
    let requirement_part = value.split(';').next().unwrap_or("").trim_end();
    if is_bare_direct_reference(requirement_part) {
        return None;
    }
    let name_end = requirement_part
        .bytes()
        .position(|byte| !valid_upstream_requirement_name_byte(byte))
        .unwrap_or(requirement_part.len());
    let name = &requirement_part[..name_end];
    if name.is_empty() {
        return None;
    }

    let after_name = &requirement_part[name_end..];
    let extras_len = if after_name.starts_with('[') {
        after_name.find(']').map_or(0, |end| end + 1)
    } else {
        0
    };
    let after_qualifier = &after_name[extras_len..];
    let leading_space = leading_space_len(after_qualifier);
    if let Some(at_offset) = after_qualifier[leading_space..].find('@') {
        let before_at = &after_qualifier[leading_space..leading_space + at_offset];
        if before_at.trim().is_empty() {
            let raw_requirement = &after_qualifier[leading_space + at_offset + 1..];
            let requirement = raw_requirement.trim();
            if !requirement.is_empty() {
                let trim_start = raw_requirement.len() - raw_requirement.trim_start().len();
                let requirement_start =
                    name_end + extras_len + leading_space + at_offset + 1 + trim_start;
                return Some((name, requirement, requirement_start));
            }
        }
    }
    if let Some(parenthesized) = after_qualifier[leading_space..].strip_prefix('(')
        && let Some(close) = parenthesized.find(')')
    {
        let requirement_start = name_end + extras_len + leading_space + 1;
        let requirement = parenthesized[..close].trim();
        let trimmed_start =
            parenthesized[..close].len() - parenthesized[..close].trim_start().len();
        return Some((name, requirement, requirement_start + trimmed_start));
    }
    let operator_offset = after_qualifier
        .find(['=', '<', '>', '~', '!'])
        .filter(|offset| after_qualifier[..*offset].trim().is_empty());
    let Some(operator_offset) = operator_offset else {
        let version_start = name_end + extras_len + leading_space_len(after_qualifier);
        let version_len = requirement_part[version_start..]
            .bytes()
            .take_while(|byte| valid_upstream_requirement_version_byte(*byte))
            .count();
        if version_len == 0 {
            return Some((name, "", name_end));
        }
        let requirement_end = version_start + version_len;
        return Some((
            name,
            &requirement_part[version_start..requirement_end],
            version_start,
        ));
    };

    let split = name_end + extras_len + operator_offset;
    let operator_end = split + requirement_operator_len(&requirement_part[split..]);
    let version_start = operator_end + leading_space_len(&requirement_part[operator_end..]);
    let version_len = requirement_part[version_start..]
        .bytes()
        .take_while(|byte| valid_upstream_requirement_version_byte(*byte))
        .count();
    if version_len == 0 {
        return Some((name, "", name_end));
    }

    let requirement_end = version_start + version_len;
    Some((name, &requirement_part[split..requirement_end], split))
}

fn is_bare_direct_reference(value: &str) -> bool {
    [
        "http://", "https://", "ftp://", "ssh://", "git://", "git+", "git@", "svn://", "svn+",
        "hg://", "hg+", "bzr://", "bzr+", "file:",
    ]
    .iter()
    .any(|prefix| value.starts_with(prefix))
}

fn strip_comment(input: &str) -> &str {
    input
        .char_indices()
        .find_map(|(index, char)| {
            (char == '#' && input[..index].ends_with(crate::is_whitespace))
                .then_some(&input[..index])
        })
        .unwrap_or(input)
}

fn valid_upstream_requirement_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
}

fn requirement_operator_len(value: &str) -> usize {
    ["===", "==", "~=", ">=", "<=", "!=", ">", "<"]
        .iter()
        .find_map(|operator| value.starts_with(operator).then_some(operator.len()))
        .unwrap_or(0)
}

fn leading_space_len(value: &str) -> usize {
    value
        .bytes()
        .take_while(|byte| byte.is_ascii_whitespace())
        .count()
}

fn valid_upstream_requirement_version_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'*' | b'+' | b'-')
}
