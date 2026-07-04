pub(in crate::json_manifest::npm::github) fn is_commit_sha(value: &str) -> bool {
    matches!(value.len(), 7 | 40) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}
