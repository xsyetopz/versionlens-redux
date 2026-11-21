use super::*;

#[test]
fn expands_placeholders() {
    assert_eq!(expand("{cargo}", "Serde", None).unwrap(), "se/rd/serde");
    assert_eq!(expand("{cargo}", "syn", None).unwrap(), "3/s/syn");
    assert_eq!(
        expand("{enc}", "@types/node", None).unwrap(),
        "@types%2fnode"
    );
    assert_eq!(
        expand("{go}", "github.com/BurntSushi/toml", None).unwrap(),
        "github.com/!burnt!sushi/toml"
    );
    assert_eq!(
        expand("{path}", "org.junit:junit", None).unwrap(),
        "org/junit/junit"
    );
    assert_eq!(expand("{path}", "ring", None).unwrap(), "ring/ring");
    assert_eq!(
        expand("{ns}/{short}", "community.general", None).unwrap(),
        "community/general"
    );
    assert_eq!(expand("{docker}", "node", None).unwrap(), "library/node");
    assert_eq!(
        expand("{docker}", "docker.io/bitnami/redis", None).unwrap(),
        "bitnami/redis"
    );
    assert!(expand("{docker}", "ghcr.io/a/b", None).is_err());
    assert_eq!(
        expand("{source}/index.yaml", "x", Some("https://c.io/")).unwrap(),
        "https://c.io/index.yaml"
    );
    assert!(expand("{source}", "x", Some("oci://c.io")).is_err());
}

#[test]
fn parses_each_response_shape() {
    let parse = |id: &str, body: &str, name: &str| get(id).unwrap().parse(body, name).unwrap();
    let npm = parse(
        "npm",
        r#"{"versions":{"1.0.0":{},"2.0.0":{}},"dist-tags":{"latest":"1.0.0"}}"#,
        "x",
    );
    assert_eq!(
        npm,
        Versions {
            list: vec!["1.0.0".into(), "2.0.0".into()],
            latest: Some("1.0.0".into())
        }
    );
    assert_eq!(
        parse(
            "composer",
            r#"{"packages":{"a/b":[{"version":"v1.0"}]}}"#,
            "a/b"
        )
        .list,
        ["v1.0"]
    );
    assert_eq!(parse("cran", r#"{"Version":"1.2"}"#, "x").list, ["1.2"]);
    assert_eq!(
        parse(
            "vcpkg",
            r#"{"versions":[{"version-semver":"1.0.0"},{"version":"0.9"}]}"#,
            "x"
        )
        .list,
        ["1.0.0", "0.9"]
    );
    let cargo = "{\"vers\":\"1.0.0\",\"yanked\":false}\n{\"vers\":\"1.1.0\",\"yanked\":true}\n";
    assert_eq!(parse("cargo", cargo, "x").list, ["1.0.0"]);
    assert_eq!(
        parse("go", "v1.0.0\nv1.1.0\n", "x").list,
        ["v1.0.0", "v1.1.0"]
    );
    let opam = r#"<a href="../dune/dune.3.17.2/"> <a href="../alba/alba.0.4.4/">"#;
    assert_eq!(parse("opam", opam, "dune").list, ["3.17.2"]);
    assert!(get("npm").unwrap().parse("<html>", "x").is_err());
}

#[test]
fn reads_helm_index_entries() {
    let index = "apiVersion: v1\nentries:\n  other:\n  - version: 9.9.9\n  redis:\n  - apiVersion: v2\n    version: 17.1.0\n    dependencies:\n    - name: common\n      version: 2.0.0\n  - version: \"17.0.0\"\n  zed:\n  - version: 1.0.0\n";
    assert_eq!(helm(index, "redis"), ["17.1.0", "17.0.0"]);
}
