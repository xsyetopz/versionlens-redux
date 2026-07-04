use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_go_mod_dependencies() {
    let text = "\
module example.test/app

require example.test/one v1.2.3
replace example.test/local => ../local
replace example.test/old v1.0.0 => ./vendor/old

require (
\tgithub.com/docker/cli v26.1.3+incompatible
\tk8s.io/utils v0.0.0-20230726121419-3b25d923346b // indirect
\texample.test/prerelease v1.0.0-alpha-beta
)

exclude example.test/bad v0.5.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Go);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/one");
    assert_eq!(dependencies[0].requirement, "v1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "v1.2.3"
    );
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.test/local");
    assert_eq!(dependencies[1].requirement, "=>");
    assert_eq!(extract_range(text, dependencies[1].requirement_range), "=>");
    assert_eq!(dependencies[2].group, "replace");
    assert_eq!(dependencies[2].name, "example.test/old");
    assert_eq!(dependencies[2].requirement, "v1.0.0");
    assert_eq!(dependencies[3].name, "github.com/docker/cli");
    assert_eq!(dependencies[3].requirement, "v26.1.3+incompatible");
    assert_eq!(dependencies[3].requirement_suffix, "+incompatible");
    assert_eq!(dependencies[4].group, "exclude");
    assert_eq!(dependencies[4].name, "example.test/bad");
}

#[test]
fn parses_smoke_go_mod_smoke_shapes() {
    let text = "\
module github.com/xxx/yyy

go 1.26.4

retract v1.1.0 // Published accidentally.

retract [v1.0.0, v1.0.5] // Build broken on some platforms.

require (
\tgithub.com/docker/buildx v0.35.0
\tgithub.com/docker/cli v29.6.0+incompatible
\tgithub.com/docker/cli-docs-tool v0.7.0
\tgithub.com/docker/docker v28.5.2+incompatible
\tgithub.com/docker/go-connections v0.7.0
\tgithub.com/docker/go-units v0.5.0
)

require golang.org/x/term v0.44.0

require (
\tk8s.io/api v0.36.2 // indirect
\tk8s.io/apimachinery v0.36.2 // indirect
\tk8s.io/apiserver v0.36.2 // indirect
\tk8s.io/client-go v1.5.2 // indirect
\tk8s.io/klog/v2 v2.140.0 // indirect
\tk8s.io/kube-openapi v0.0.0-20231010175941-2dd684a91f00 // indirect
\tk8s.io/utils v0.0.0-20230726121419-3b25d923346b // indirect
)

exclude github.com/docker/go-units v0.5.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 13);
    assert_eq!(dependencies[0].name, "github.com/docker/buildx");
    assert_eq!(dependencies[1].name, "github.com/docker/cli");
    assert_eq!(dependencies[1].requirement_suffix, "+incompatible");
    assert_eq!(dependencies[6].name, "golang.org/x/term");
    assert_eq!(dependencies[7].name, "k8s.io/api");
    assert_eq!(dependencies[11].name, "k8s.io/klog/v2");
    assert_eq!(dependencies[12].group, "exclude");
    assert_eq!(dependencies[12].name, "github.com/docker/go-units");
}

#[test]
fn go_mod_dependency_without_version_parses_blank_requirement_like_upstream() {
    let text = "require example.test/blank\n";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "require");
    assert_eq!(dependencies[0].name, "example.test/blank");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(
        extract_range(text, dependencies[0].range),
        "example.test/blank"
    );
    assert_eq!(
        dependencies[0].requirement_range.start,
        dependencies[0].range.end
    );
    assert_eq!(
        dependencies[0].requirement_range.end,
        dependencies[0].range.end
    );
}

#[test]
fn go_mod_versions_with_two_hyphens_are_skipped_like_upstream() {
    let text = "require example.test/prerelease v1.0.0-alpha-beta\n";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}

#[test]
fn go_mod_single_line_directives_require_literal_space_like_upstream() {
    let text = "require\texample.test/tab v1.2.3\n";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert!(dependencies.is_empty());
}

#[test]
fn go_mod_replace_dependencies_use_second_token_like_upstream() {
    let text = "\
replace example.test/local => ../local
replace example.test/old v1.0.0 => ./vendor/old
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/go.mod".to_owned(),
        language_id: "go.mod".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].group, "replace");
    assert_eq!(dependencies[0].name, "example.test/local");
    assert_eq!(dependencies[0].requirement, "=>");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "=>");
    assert_eq!(dependencies[1].group, "replace");
    assert_eq!(dependencies[1].name, "example.test/old");
    assert_eq!(dependencies[1].requirement, "v1.0.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "v1.0.0"
    );
}
