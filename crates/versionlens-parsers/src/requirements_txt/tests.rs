use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn parses_requirements_txt_dependencies() {
    let text = "# nope\n requests>=2.0 # comment\n-r other.txt\nflask==3.0.0\nhttpx[socks]>=0.28; python_version >= '3.12'\nimportlib-metadata; python_version < '3.8'\nlocal @ https://example.test/local.whl#sha256=abc\n";
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
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Python);
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
fn requirements_txt_adjacent_hash_not_part_of_normal_version() {
    let text = "pkg==1.2.3#keep\n";
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
    let text = "# Requirements for smoke testing
requests==2.34.2
flask>=3.1.3
django<=6.0.6
pytest>9.1.1
numpy<=2.5.0
pandas~=3.0.3
urllib3===2.7.0
six==1.17.0
python-dateutil==2.9.0
not_found_package==1.17.0
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 10);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Python);
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
    let text = "@scope/pkg==1.0.0
valid_name-1.2==2.0.0
name$bad==3.0.0
";
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
    let text = "local @ https://example.test/local.whl#sha256=abc
";
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
    let text = "pkg>=1.0,<2.0
other==1.0; python_version >= '3.12'
";
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
    let text = "pkg>= 1.0\nother == 2.0\n";
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
    let text = "pkg 1.0\nlocal @ https://example.test/local.whl\n";
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
    let text = "-r other.txt\n--index-url https://example.test/simple\n";
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
