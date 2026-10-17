use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Swift;

const CARTFILE: &str = r#"# Networking
github "Alamofire/Alamofire" ~> 5.8
github "ReactiveX/RxSwift" == 6.6.0 # pinned
git "https://github.com/Quick/Nimble.git" >= 12.0
github "owner/branchy" "main"
binary "https://example.com/spec.json" ~> 1.0
github "https://github.com/onevcat/Kingfisher" ~> 7.0
"#;

#[test]
fn parses_carthage_cartfiles() {
    for uri in ["file:///work/Cartfile", "file:///work/Cartfile.private"] {
        let dependencies = crate::support::tests::parse_test_document(CARTFILE, uri, "plaintext");

        let found = dependencies
            .iter()
            .map(|dependency| {
                assert_eq!(dependency.ecosystem, Swift, "{uri}");
                (
                    extract_range(CARTFILE, dependency.range),
                    extract_range(CARTFILE, dependency.requirement_range),
                    dependency.requirement.as_str(),
                    dependency.hosted_name.as_deref(),
                    dependency.hosted_url.as_deref(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            found,
            [
                (
                    "Alamofire/Alamofire",
                    "~> 5.8",
                    "~> 5.8",
                    Some("Alamofire/Alamofire"),
                    Some("https://api.github.com/repos/Alamofire/Alamofire/tags"),
                ),
                (
                    "ReactiveX/RxSwift",
                    "== 6.6.0",
                    "== 6.6.0",
                    Some("ReactiveX/RxSwift"),
                    Some("https://api.github.com/repos/ReactiveX/RxSwift/tags"),
                ),
                (
                    "https://github.com/Quick/Nimble.git",
                    ">= 12.0",
                    ">= 12.0",
                    Some("Quick/Nimble"),
                    Some("https://api.github.com/repos/Quick/Nimble/tags"),
                ),
                ("owner/branchy", "main", "main", None, Some("git")),
                (
                    "https://github.com/onevcat/Kingfisher",
                    "~> 7.0",
                    "~> 7.0",
                    Some("onevcat/Kingfisher"),
                    Some("https://api.github.com/repos/onevcat/Kingfisher/tags"),
                ),
            ],
            "{uri}"
        );
    }
}

#[test]
fn the_resolved_cartfile_is_not_a_manifest() {
    let dependencies = crate::support::tests::parse_test_document(
        "github \"Alamofire/Alamofire\" \"5.8.1\"\n",
        "file:///work/Cartfile.resolved",
        "plaintext",
    );

    assert!(dependencies.is_empty());
}
