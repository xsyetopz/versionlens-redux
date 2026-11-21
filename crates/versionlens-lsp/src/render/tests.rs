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
