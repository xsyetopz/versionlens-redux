use serde_json::json;

use super::*;

#[test]
fn encodes_base64_with_padding() {
    for (plain, coded) in [
        ("", ""),
        ("f", "Zg=="),
        ("fo", "Zm8="),
        ("foo", "Zm9v"),
        ("user:secret", "dXNlcjpzZWNyZXQ="),
    ] {
        assert_eq!(base64(plain.as_bytes()), coded, "{plain}");
    }
}

#[test]
fn turns_each_entry_kind_into_its_header() {
    assert_eq!(header(&json!("Token abc ")), Some("Token abc".into()));
    assert_eq!(
        header(&json!({ "token": "abc" })),
        Some("Bearer abc".into())
    );
    let basic = json!({ "user": "user", "password": "secret" });
    assert_eq!(header(&basic), Some("Basic dXNlcjpzZWNyZXQ=".into()));
    for blank in [
        json!(""),
        json!({ "token": " " }),
        json!({ "user": "u", "password": "" }),
        json!(1),
    ] {
        assert_eq!(header(&blank), None, "{blank}");
    }
}

#[test]
fn takes_a_registry_prefix_from_its_url_template() {
    let settings = Settings::default();
    assert_eq!(prefix(&settings, "npm"), "https://registry.npmjs.org/");
    assert_eq!(prefix(&settings, "no-such-registry"), "https://");
    let mut custom = Settings::default();
    custom
        .registries
        .insert("npm".into(), "https://npm.example.com/api/{enc}".into());
    assert_eq!(prefix(&custom, "npm"), "https://npm.example.com/api/");
}

#[test]
fn puts_the_cursor_in_the_first_blank_or_on_a_bare_scheme() {
    let text = "{\n  \"a/\": \"x\",\n  \"https://b/\": {\n    \"password\": \"\",\n    \"user\": \"\"\n  }\n}\n";
    let at = cursor(text, "https://b/");
    assert_eq!(at["start"], json!({ "line": 3, "character": 17 }));
    assert_eq!(at["start"], at["end"]);
    let bare = cursor("{\n  \"https://\": \"\"\n}\n", "https://");
    assert_eq!(bare["start"], json!({ "line": 1, "character": 3 }));
    assert_eq!(bare["end"], json!({ "line": 1, "character": 11 }));
}

#[test]
fn reads_a_missing_file_as_empty_and_names_a_corrupt_one() {
    let dir = std::env::temp_dir().join(format!("versionlens-stored-{}", std::process::id()));
    let path = dir.join("authorization.json");
    drop(std::fs::remove_dir_all(&dir));
    assert!(read(&path).unwrap().is_empty());
    let mut entries = Map::new();
    entries.insert("https://a/".into(), json!("H"));
    write(&path, &entries).unwrap();
    assert_eq!(read(&path).unwrap(), entries);
    std::fs::write(&path, "[1]").unwrap();
    assert!(read(&path).unwrap_err().contains("not a JSON object"));
    drop(std::fs::remove_dir_all(&dir));
}
