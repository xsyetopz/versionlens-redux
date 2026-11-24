use super::*;

#[test]
fn finds_the_uri_and_level_in_any_order() {
    let (uri, level) = targets(&[json!("major"), json!("file:///p/package.json")]);
    assert_eq!(uri.unwrap().as_str(), "file:///p/package.json");
    assert_eq!(level, Some("major"));
    assert_eq!(targets(&[json!("file:///p/Cargo.toml")]).1, None);
    assert!(targets(&[]).0.is_none());
}
