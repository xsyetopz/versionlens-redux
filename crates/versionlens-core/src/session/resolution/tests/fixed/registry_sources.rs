use super::{DocumentInput, standard_session};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;

#[test]
fn requirements_index_urls_override_python_registry_urls() {
    let input = DocumentInput::new(
        "file:///repo/requirements.txt".to_owned(),
        "pip-requirements".to_owned(),
        package_file_fixture("requirements-index-urls-override-python-registry-urls.txt"),
        None,
    );
    assert_registry_urls(
        &input,
        "requests",
        &[
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
        ],
    );
}

#[test]
fn pipfile_sources_override_python_registry_urls() {
    let input = DocumentInput::new(
        "file:///repo/Pipfile".to_owned(),
        "toml".to_owned(),
        package_file_fixture("pipfile-sources-override-python-registry-urls.Pipfile"),
        None,
    );
    assert_registry_urls(
        &input,
        "requests",
        &["https://pypi.example.test/simple/requests/"],
    );
}

#[test]
fn python_documents_use_workspace_pip_conf_registry_urls() {
    let root = temp_dir().join(format!("versionlens-pip-conf-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("pip.conf"),
        "[global]\nindex-url = https://primary.example.test/simple/\nextra-index-url = https://extra.example.test/simple\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("requirements.txt").display()),
        "pip-requirements".to_owned(),
        package_file_fixture("python-documents-use-workspace-pip-conf-registry-urls.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    assert_registry_urls(
        &input,
        "requests",
        &[
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
        ],
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn python_documents_use_pip_environment_registry_urls() {
    let root = temp_dir().join(format!("versionlens-pip-env-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".env"),
        "PIP_INDEX_URL=https://env-primary.example.test/simple/\nPIP_EXTRA_INDEX_URL=https://env-extra-one.example.test/simple https://env-extra-two.example.test/simple/\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("requirements.txt").display()),
        "pip-requirements".to_owned(),
        package_file_fixture("python-documents-use-pip-environment-registry-urls.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    assert_registry_urls(
        &input,
        "requests",
        &[
            "https://env-primary.example.test/simple/requests/",
            "https://env-extra-one.example.test/simple/requests/",
            "https://env-extra-two.example.test/simple/requests/",
        ],
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gemfile_source_urls_override_ruby_registry_urls() {
    let input = DocumentInput::new(
        "file:///repo/Gemfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("gemfile-source-urls-override-ruby-registry-urlsGemfile"),
        None,
    );
    assert_registry_urls(
        &input,
        "rails",
        &["https://gems.example.test/api/v1/versions/rails.json"],
    );
}

#[test]
fn gemfile_source_blocks_override_dependency_registry_url() {
    let input = DocumentInput::new(
        "file:///repo/Gemfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("gemfile-source-blocks-override-dependency-registry-urlGemfile"),
        None,
    );
    assert_private_gem_registry_url(&input);
}

#[test]
fn gemfile_source_option_overrides_dependency_registry_url() {
    let input = DocumentInput::new(
        "file:///repo/Gemfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture("gemfile-source-option-overrides-dependency-registry-urlGemfile"),
        None,
    );
    assert_private_gem_registry_url(&input);
}

fn assert_private_gem_registry_url(input: &DocumentInput) {
    let session = standard_session();
    let dependencies = session.dependencies(input);
    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://private.gems.example.test/api/v1/versions/private_gem.json"]
    );
}

#[test]
fn pyproject_uv_indexes_override_python_registry_urls() {
    let input = DocumentInput::new(
        "file:///repo/pyproject.toml".to_owned(),
        "toml".to_owned(),
        package_file_fixture("uv-indexes-override-python-registry-urls.toml"),
        None,
    );
    assert_registry_urls(
        &input,
        "requests",
        &[
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
            "https://private.example.test/simple/requests/",
        ],
    );
}

#[test]
fn pyproject_poetry_sources_override_python_registry_urls() {
    let input = DocumentInput::new(
        "file:///repo/pyproject.toml".to_owned(),
        "toml".to_owned(),
        package_file_fixture("poetry-sources-override-python-registry-urls.toml"),
        None,
    );
    assert_registry_urls(
        &input,
        "requests",
        &["https://poetry.example.test/simple/requests/"],
    );
}

#[test]
fn pyproject_poetry_dependency_source_overrides_python_registry_url() {
    let input = DocumentInput::new(
        "file:///repo/pyproject.toml".to_owned(),
        "toml".to_owned(),
        package_file_fixture("poetry-dependency-source-overrides-python-registry-url.toml"),
        None,
    );
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[1].name, "private");
    assert_eq!(dependencies[1].hosted_url, Some("private".to_owned()));
    let context = crate::registry::RegistryContext::from_document(&input);
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://private.example.test/simple/private/"]
    );
}

#[test]
fn python_documents_use_workspace_uv_toml_registry_urls() {
    let root = temp_dir().join(format!("versionlens-uv-toml-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("uv.toml"),
        "index-url = 'https://primary.example.test/simple/'\nextra-index-url = ['https://extra.example.test/simple']\n[[index]]\nname = 'private'\nurl = 'https://private.example.test/simple/'\n",
    )
    .unwrap();

    let input = DocumentInput::new(
        format!("file://{}", root.join("pyproject.toml").display()),
        "toml".to_owned(),
        package_file_fixture("python-documents-use-workspace-uv-toml-registry-urls.txt"),
        Some(root.to_string_lossy().into_owned()),
    );
    assert_registry_urls(
        &input,
        "requests",
        &[
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
            "https://private.example.test/simple/requests/",
        ],
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn podfile_external_and_latest_dependencies_resolve_as_fixed_without_registry_updates() {
    let input = DocumentInput::new(
        "file:///repo/Podfile".to_owned(),
        "ruby".to_owned(),
        package_file_fixture(
            "podfile-external-and-latest-dependencies-resolve-as-fixed-without-registry-updatesPodfile",
        ),
        None,
    );
    let output = standard_session().resolve_document(input);

    assert_eq!(output.suggestions.len(), 4);
    assert!(output.edits.is_empty());
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("latest version")
    );
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("local pod"));
    assert_eq!(
        output.suggestions[2].latest.as_deref(),
        Some("git repository")
    );
    assert_eq!(
        output.suggestions[3].latest.as_deref(),
        Some("podspec source")
    );
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture(
        "tests/fixtures/session/resolution/tests/fixed/registry_sources",
        name,
    )
}

fn assert_registry_urls(input: &DocumentInput, dependency_name: &str, expected: &[&str]) {
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(input);
    let dependency = dependencies
        .iter()
        .find(|dependency| dependency.name == dependency_name)
        .unwrap_or_else(|| panic!("missing dependency {dependency_name}"));
    assert_eq!(
        session.registry_urls_with_context(dependency, &context),
        expected
            .iter()
            .map(|url| (*url).to_owned())
            .collect::<Vec<_>>()
    );
}
