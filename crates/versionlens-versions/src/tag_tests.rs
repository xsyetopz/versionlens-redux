use super::version_tag_parts;

#[test]
fn splits_plain_prefixed_and_path_qualified_version_tags() {
    assert_eq!(version_tag_parts("v7.0.1"), Some(("v", "7.0.1")));
    assert_eq!(
        version_tag_parts("release/action-v2.4.0"),
        Some(("release/action-v", "2.4.0"))
    );
    assert_eq!(
        version_tag_parts("runtime-2026-v1.2.3-beta.1"),
        Some(("runtime-2026-v", "1.2.3-beta.1"))
    );
}

#[test]
fn rejects_tags_without_a_comparable_version_suffix() {
    assert_eq!(version_tag_parts("main"), None);
    assert_eq!(version_tag_parts("release/v"), None);
    assert_eq!(version_tag_parts(""), None);
}
