use super::*;

fn v(text: &str) -> Version {
    Version::parse(text).unwrap()
}

#[test]
fn replaces_in_the_users_style() {
    assert_eq!(replacement("^1.2.3", &v("1.4.0"), ""), "^1.4.0");
    assert_eq!(replacement("~> 1.2", &v("2.1.0"), ""), "~> 2.1");
    assert_eq!(replacement(">=4.2, <5", &v("5.1"), ""), ">=5.1");
    assert_eq!(replacement("v4", &v("v5.0.0"), ""), "v5");
    assert_eq!(replacement("", &v("2.0"), "=="), "==2.0");
    assert_eq!(replacement("[1.0,2.0)", &v("3.0"), ""), "[3.0]");
    assert_eq!(replacement("20-alpine", &v("22.1-alpine"), ""), "22-alpine");
}

#[test]
fn positions_count_utf16_units() {
    let text = "a\n€b c\n";
    let lines = Lines::new(text);
    let at = text.find('c').unwrap();
    assert_eq!(lines.position(at), Position::new(1, 3));
    assert_eq!(lines.position(0), Position::new(0, 0));
}

#[test]
fn shortens_errors_to_their_cause() {
    assert_eq!(
        short("https://x: error sending request: dns error"),
        "dns error"
    );
}

#[test]
fn maps_positions_to_byte_offsets() {
    let lines = Lines::new("a€b\nxy\n");
    assert_eq!(lines.offset(Position::new(0, 2)), 4);
    assert_eq!(lines.offset(Position::new(0, 99)), 5);
    assert_eq!(lines.offset(Position::new(1, 1)), 7);
    assert_eq!(lines.offset(Position::new(9, 0)), 9);
}

#[test]
fn offers_updates_only_to_matching_kinds() {
    let kinds = |list: &[&str]| {
        list.iter()
            .map(|k| CodeActionKind::from(k.to_string()))
            .collect::<Vec<_>>()
    };
    assert!(wants_actions(None));
    assert!(wants_actions(Some(&kinds(&["refactor"]))));
    assert!(wants_actions(Some(&kinds(&[
        "quickfix",
        "refactor.rewrite"
    ]))));
    assert!(!wants_actions(Some(&kinds(&["quickfix"]))));
    assert!(!wants_actions(Some(&kinds(&["refactor.re"]))));
    assert!(!wants_actions(Some(&kinds(&["source.organizeImports"]))));
}
