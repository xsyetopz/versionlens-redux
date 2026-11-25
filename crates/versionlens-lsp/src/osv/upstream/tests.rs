use super::found;
use crate::registry::get;

/// The repository URL a registry's upstream pattern finds in a document excerpt.
fn repo(id: &str, text: &str) -> Option<String> {
    found(get(id).unwrap().upstream, "", text)
}

#[test]
fn finds_the_repository_each_registry_names() {
    let cases = [
        (
            "bazel",
            r#""repository": ["github:bazel-contrib/rules_go"]"#,
            "bazel-contrib/rules_go",
        ),
        (
            "vcpkg",
            "vcpkg_from_github(\n    OUT_SOURCE_PATH SOURCE_PATH\n    REPO fmtlib/fmt\n",
            "fmtlib/fmt",
        ),
        (
            "dub",
            r#""repository":{"kind":"github","owner":"vibe-d","project":"vibe.d"}"#,
            "vibe-d/vibe.d",
        ),
        (
            "cocoapods",
            r#""source":{"git":"https://github.com/Alamofire/Alamofire.git","tag":"5.0.0"}"#,
            "Alamofire/Alamofire",
        ),
        (
            "chef",
            r#""source_url":"https://github.com/sous-chefs/nginx""#,
            "sous-chefs/nginx",
        ),
        (
            "conda",
            r#""dev_url": "https://github.com/numpy/numpy/""#,
            "numpy/numpy",
        ),
        (
            "xmake",
            "add_urls(\"https://github.com/madler/zlib/archive/refs/tags/$(version).tar.gz\",",
            "madler/zlib",
        ),
        (
            "cpan",
            r#""repository" : { "web" : "https://github.com/moose/Moose" }"#,
            "moose/Moose",
        ),
    ];
    for (id, text, expected) in cases {
        assert_eq!(
            repo(id, text),
            Some(format!("https://github.com/{expected}")),
            "{id}"
        );
    }
}

#[test]
fn finds_nothing_where_the_registry_names_no_github_repository() {
    assert_eq!(
        repo("chef", r#""source_url":"https://gitlab.com/a/b""#),
        None
    );
}
