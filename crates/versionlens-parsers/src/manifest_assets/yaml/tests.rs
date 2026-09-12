use super::scalar_range;

#[test]
fn quoted_scalar_ranges_cover_the_encoded_source() {
    for (encoded, decoded, contents) in [
        (r#""v\u0031.2.0""#, "v1.2.0", r#"v\u0031.2.0"#),
        (r#"'it''s v1'"#, "it's v1", "it''s v1"),
        (r#""a\"b""#, "a\"b", r#"a\"b"#),
        (r#""""#, "", ""),
        ("'🦀 v1'", "🦀 v1", "🦀 v1"),
    ] {
        let text = format!("name: '🦀'\nversion: {encoded} # retained\n");
        crate::support::with_yaml_mapping(&text, |root| {
            let scalar = root.get_scalar("version").unwrap();
            assert_eq!(scalar.as_str(), decoded);
            let range = scalar_range(&text, scalar).unwrap();
            assert_eq!(&text[range], contents);
        })
        .unwrap();
    }
}
