use crate::document::test_support::extract_range;
use crate::model::Ecosystem::Python;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;

#[test]
fn parses_requirements_txt_dependencies() {
    let text = package_file_fixture("parses-requirements-txt-dependencies.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(dependencies[0].requirement, ">=2.0");
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].ecosystem, Python);
    assert_eq!(dependencies[0].range.start.character, 1);
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">=2.0"
    );
    assert_eq!(dependencies[1].name, "-r");
    assert_eq!(dependencies[1].requirement, "other.txt");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "other.txt"
    );
    assert_eq!(dependencies[2].name, "flask");
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "==3.0.0"
    );
    assert_eq!(dependencies[3].name, "httpx");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, "==");
    assert_eq!(extract_range(text, dependencies[3].range), "httpx");
    assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
    assert_eq!(dependencies[4].name, "importlib-metadata");
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, "==");
    assert_eq!(
        extract_range(text, dependencies[4].range),
        "importlib-metadata"
    );
    assert_eq!(extract_range(text, dependencies[4].requirement_range), "");
    assert_eq!(dependencies[5].name, "local");
    assert_eq!(dependencies[5].requirement, "");
    assert_eq!(dependencies[5].requirement_prefix, "==");
    assert_eq!(extract_range(text, dependencies[5].requirement_range), "");
}

#[test]
fn parses_parenthesized_pep_508_requirements() {
    let text = "unfat (>=0.0.13)
httpx[http2] (>=0.28,<1); python_version >= '3.11'
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "unfat");
    assert_eq!(dependencies[0].requirement, ">=0.0.13");
    assert_eq!(dependencies[0].requirement_suffix, ")");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">=0.0.13"
    );
    assert_eq!(dependencies[1].name, "httpx");
    assert_eq!(dependencies[1].requirement, ">=0.28,<1");
    assert_eq!(dependencies[1].requirement_suffix, ")");
}

#[test]
fn requirements_txt_adjacent_hash_not_part_of_normal_version() {
    let text =
        package_file_fixture("requirements-txt-adjacent-hash-not-part-of-normal-version.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "pkg");
    assert_eq!(dependencies[0].requirement, "==1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "==1.2.3"
    );
}

#[test]
fn parses_smoke_requirements_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-requirements-smoke-shapes.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 10);
    assert_eq!(dependencies[0].ecosystem, Python);
    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(dependencies[0].requirement, "==2.34.2");
    assert_eq!(dependencies[1].name, "flask");
    assert_eq!(dependencies[1].requirement, ">=3.1.3");
    assert_eq!(dependencies[2].name, "django");
    assert_eq!(dependencies[2].requirement, "<=6.0.6");
    assert_eq!(dependencies[5].name, "pandas");
    assert_eq!(dependencies[5].requirement, "~=3.0.3");
    assert_eq!(dependencies[6].name, "urllib3");
    assert_eq!(dependencies[6].requirement, "===2.7.0");
    assert_eq!(
        extract_range(text, dependencies[6].requirement_range),
        "===2.7.0"
    );
    assert_eq!(dependencies[9].name, "not_found_package");
}

#[test]
fn requirements_txt_rejects_names_outside_upstream_regex() {
    let text = package_file_fixture("requirements-txt-rejects-names-outside-upstream-regex.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "valid_name-1.2");
    assert_eq!(dependencies[0].requirement, "==2.0.0");
    assert_eq!(dependencies[1].name, "name");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, "==");
}

#[test]
fn requirements_txt_direct_urls_match_upstream_blank_version() {
    let text =
        package_file_fixture("requirements-txt-direct-urls-match-upstream-blank-version.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "local");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[0].requirement_prefix, "==");
    assert_eq!(extract_range(text, dependencies[0].range), "local");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
}

#[test]
fn requirements_txt_versions_stop_at_first_upstream_version_token() {
    let text =
        package_file_fixture("requirements-txt-versions-stop-at-first-upstream-version-token.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "pkg");
    assert_eq!(dependencies[0].requirement, ">=1.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">=1.0"
    );
    assert_eq!(dependencies[1].name, "other");
    assert_eq!(dependencies[1].requirement, "==1.0");
}
#[test]
fn requirements_txt_descriptor_versions_omit_operator_spacing_like_upstream() {
    let text = package_file_fixture(
        "requirements-txt-descriptor-versions-omit-operator-spacing-like-upstream.txt",
    );
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "pkg");
    assert_eq!(dependencies[0].requirement, ">=1.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        ">= 1.0"
    );
    assert_eq!(dependencies[1].name, "other");
    assert_eq!(dependencies[1].requirement, "==2.0");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "== 2.0"
    );
}
#[test]
fn requirements_txt_accepts_raw_version_without_operator_like_upstream() {
    let text = package_file_fixture(
        "requirements-txt-accepts-raw-version-without-operator-like-upstream.txt",
    );
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "pkg");
    assert_eq!(dependencies[0].requirement, "1.0");
    assert_eq!(dependencies[0].requirement_prefix, "");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.0"
    );
    assert_eq!(dependencies[1].name, "local");
    assert_eq!(dependencies[1].requirement, "");
    assert_eq!(dependencies[1].requirement_prefix, "==");
}
#[test]
fn requirements_txt_option_like_lines_parse_like_upstream() {
    let text = package_file_fixture("requirements-txt-option-like-lines-parse-like-upstream.txt");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "-r");
    assert_eq!(dependencies[0].requirement, "other.txt");
    assert_eq!(extract_range(text, dependencies[0].range), "-r");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "other.txt"
    );
    assert_eq!(dependencies[1].name, "--index-url");
    assert_eq!(dependencies[1].requirement, "https");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/requirements_txt/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
