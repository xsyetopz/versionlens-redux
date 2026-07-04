pub(super) fn contains_four_segment_nuget_version(requirement: &str) -> bool {
    requirement
        .split(|char: char| !(char.is_ascii_digit() || matches!(char, '.' | '*')))
        .any(is_four_segment_nuget_version)
}

fn is_four_segment_nuget_version(value: &str) -> bool {
    let mut parts = value.split('.');
    matches!(
        (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next()
        ),
        (Some(major), Some(minor), Some(patch), Some(revision), None)
            if digits(major)
                && digits(minor)
                && digits(patch)
                && (revision == "*" || digits(revision))
    )
}

fn digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}
