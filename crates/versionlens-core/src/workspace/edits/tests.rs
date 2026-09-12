use super::*;

#[test]
fn edits_only_structural_package_fields() {
    let text = r#"{"metadata":{"version":"1.0.0"},"scripts":{"a":"1.0.0"},"version":"1.0.0","dependencies":{"a":"^1.0.0"}}"#;
    let edits = package_edits(text, "a", "1.0.0", "2.0.0", true);
    assert_eq!(edits.len(), 2);
    for edit in edits {
        let start = usize::try_from(edit.range.start.character).unwrap();
        let end = usize::try_from(edit.range.end.character).unwrap();
        assert!(start > text.find("scripts").unwrap());
        assert_eq!(text[start..end].replace("1.0.0", "2.0.0"), edit.new_text);
    }
}

#[test]
fn version_tokens_preserve_other_releases_in_compound_ranges() {
    assert_eq!(
        replace_version("^1.0.0 || ^11.0.0 || v1.0.0", "1.0.0", "2.0.0"),
        Some("^2.0.0 || ^11.0.0 || v2.0.0".into())
    );
    assert_eq!(replace_version("1.0.0-beta", "1.0.0", "2.0.0"), None);
}
