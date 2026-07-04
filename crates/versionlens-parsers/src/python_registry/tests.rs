use super::{
    parse_pip_conf_registry_urls, parse_pip_env_registry_urls, parse_pipfile_source_urls,
    parse_poetry_source_urls, parse_python_registry_urls, parse_uv_registry_urls,
};

#[test]
fn parses_requirements_index_urls() {
    let urls = parse_python_registry_urls(
        r#"
--index-url https://primary.example.test/simple
-i=https://short.example.test/simple/
--extra-index-url "https://extra.example.test/simple" # comment
requests==2.32.0
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://primary.example.test/simple",
            "https://short.example.test/simple",
            "https://extra.example.test/simple",
        ]
    );
}

#[test]
fn parses_pip_conf_index_urls() {
    let urls = parse_pip_conf_registry_urls(
        r#"
[global]
index-url = https://primary.example.test/simple/
extra-index-url = 'https://extra.example.test/simple'
timeout = 10
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://primary.example.test/simple",
            "https://extra.example.test/simple",
        ]
    );
}

#[test]
fn parses_pip_environment_registry_urls() {
    let env = vec![
        (
            "PIP_INDEX_URL".to_owned(),
            "https://env-primary.example.test/simple/".to_owned(),
        ),
        (
            "PIP_EXTRA_INDEX_URL".to_owned(),
            "https://env-extra-one.example.test/simple https://env-extra-two.example.test/simple/"
                .to_owned(),
        ),
    ];
    let urls = parse_pip_env_registry_urls(&env);

    assert_eq!(
        urls,
        vec![
            "https://env-primary.example.test/simple",
            "https://env-extra-one.example.test/simple",
            "https://env-extra-two.example.test/simple",
        ]
    );
}

#[test]
fn parses_pipfile_source_urls() {
    let urls = parse_pipfile_source_urls(
        r#"
[[source]]
name = "pypi"
url = "https://pypi.org/simple/"
verify_ssl = true

[[source]]
name = "private"
url = "https://pypi.example.test/simple"
verify_ssl = true
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://pypi.org/simple",
            "https://pypi.example.test/simple"
        ]
    );
}

#[test]
fn parses_poetry_source_urls() {
    let urls = parse_poetry_source_urls(
        r#"
[[tool.poetry.source]]
name = "private"
url = "https://poetry.example.test/simple/"
priority = "primary"

[[tool.poetry.source]]
name = "mirror"
url = "https://mirror.example.test/simple"
priority = "supplemental"
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://poetry.example.test/simple",
            "https://mirror.example.test/simple",
        ]
    );
}

#[test]
fn parses_uv_pyproject_registry_urls() {
    let urls = parse_python_registry_urls(
        r#"
[tool.uv]
index-url = "https://primary.example.test/simple/"
extra-index-url = ["https://extra.example.test/simple", "https://mirror.example.test/simple/"]

[[tool.uv.index]]
name = "private"
url = "https://private.example.test/simple/"
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://primary.example.test/simple",
            "https://extra.example.test/simple",
            "https://mirror.example.test/simple",
            "https://private.example.test/simple",
        ]
    );
}

#[test]
fn parses_uv_toml_registry_urls() {
    let urls = parse_uv_registry_urls(
        r#"
index-url = "https://primary.example.test/simple/"
extra-index-url = ["https://extra.example.test/simple"]

[[index]]
name = "private"
url = "https://private.example.test/simple/"
"#,
    );

    assert_eq!(
        urls,
        vec![
            "https://primary.example.test/simple",
            "https://extra.example.test/simple",
            "https://private.example.test/simple",
        ]
    );
}
