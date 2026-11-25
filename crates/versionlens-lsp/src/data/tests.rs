use super::*;

fn one(text: &str) -> Row {
    let mut rows = rows("test.toml", text, "row").unwrap();
    assert_eq!(rows.len(), 1);
    rows.remove(0)
}

#[test]
fn unknown_key_names_its_file_and_row() {
    let row = one("[[row]]\nid = 'x'\nglob = []\n");

    let error = row.only(&["id", "globs"]).unwrap_err();

    assert_eq!(error, "test.toml: x: unknown key glob");
}

#[test]
fn shared_pattern_fills_in() {
    let row = one("[[row]]\nid = 'x'\npattern = 'a${DOCKER}b'\n");

    let pattern = row.need("pattern").unwrap();

    assert_eq!(pattern, format!("a{}b", PATTERNS["DOCKER"]));
}

#[test]
fn unknown_pattern_fails() {
    let row = one("[[row]]\nid = 'x'\npattern = '${NOPE}'\n");

    let error = row.need("pattern").unwrap_err();

    assert_eq!(error, "test.toml: x: no shared pattern NOPE");
}

#[test]
fn inline_table_reports_as_its_row() {
    let row = one("[[row]]\nid = 'x'\nrules = [{ keys = 1 }]\n");

    let error = row.tables("rules").unwrap()[0].need("keys").unwrap_err();

    assert_eq!(error, "test.toml: x: keys is not a string");
}
