pub(crate) fn split_python_requirement(raw: &str) -> Option<(&str, &str, usize)> {
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
        ));
    }

    let requirement_part = value.split(';').next().unwrap_or("").trim_end();
    let split = requirement_part
        .find(['=', '<', '>', '~', '!'])
        .unwrap_or(requirement_part.len());
    let name = requirement_part[..split]
        .trim()
        .split('[')
        .next()
        .unwrap_or("")
        .trim();
    if name.is_empty() {
        return None;
    }
    let requirement = requirement_part[split..].trim_end();
    Some((name, requirement, split))
}

pub(super) fn split_requirements_txt_requirement(raw: &str) -> Option<(&str, &str, usize)> {
    let value = strip_comment(raw).trim_end();
    let requirement_part = value.split(';').next().unwrap_or("").trim_end();
    let name_end = requirement_part
        .bytes()
        .position(|byte| !valid_upstream_requirement_name_byte(byte))
        .unwrap_or(requirement_part.len());
    let name = &requirement_part[..name_end];
    if name.is_empty() {
        return None;
    }

    let after_name = &requirement_part[name_end..];
    let operator_offset = after_name
        .find(['=', '<', '>', '~', '!'])
        .filter(|offset| after_name[..*offset].trim().is_empty());
    let Some(operator_offset) = operator_offset else {
        let version_start = name_end + leading_space_len(after_name);
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

    let split = name_end + operator_offset;
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

fn strip_comment(input: &str) -> &str {
    input
        .char_indices()
        .find_map(|(index, char)| {
            (char == '#' && input[..index].ends_with(char::is_whitespace))
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
    value.bytes().take_while(u8::is_ascii_whitespace).count()
}

fn valid_upstream_requirement_version_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'*' | b'+' | b'-')
}
