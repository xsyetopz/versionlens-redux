use super::{DocumentInput, standard_session};
use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;

#[test]
fn requirements_index_urls_override_python_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: package_file_fixture("requirements-index-urls-override-python-registry-urls.txt"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    let requests = dependencies
        .iter()
        .find(|dependency| dependency.name == "requests")
        .unwrap();

    assert_eq!(
        session.registry_urls_with_context(requests, &context),
        vec![
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
        ]
    );
}

#[test]
fn pipfile_sources_override_python_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/Pipfile".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("pipfile-sources-override-python-registry-urls.Pipfile"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.example.test/simple/requests/"]
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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("requirements.txt").display()),
        language_id: "pip-requirements".to_owned(),
        text: package_file_fixture("python-documents-use-workspace-pip-conf-registry-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
        ]
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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("requirements.txt").display()),
        language_id: "pip-requirements".to_owned(),
        text: package_file_fixture("python-documents-use-pip-environment-registry-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://env-primary.example.test/simple/requests/",
            "https://env-extra-one.example.test/simple/requests/",
            "https://env-extra-two.example.test/simple/requests/",
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn gemfile_source_urls_override_ruby_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("gemfile-source-urls-override-ruby-registry-urlsGemfile"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://gems.example.test/api/v1/versions/rails.json"]
    );
}

#[test]
fn gemfile_source_blocks_override_dependency_registry_url() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture("gemfile-source-blocks-override-dependency-registry-urlGemfile"),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://private.gems.example.test/api/v1/versions/private_gem.json"]
    );
}

#[test]
fn gemfile_source_option_overrides_dependency_registry_url() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture(
            "gemfile-source-option-overrides-dependency-registry-urlGemfile",
        ),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://private.gems.example.test/api/v1/versions/private_gem.json"]
    );
}

#[test]
fn pyproject_uv_indexes_override_python_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("pyproject-uv-indexes-override-python-registry-urls.toml"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
            "https://private.example.test/simple/requests/",
        ]
    );
}

#[test]
fn pyproject_poetry_sources_override_python_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture("pyproject-poetry-sources-override-python-registry-urls.toml"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://poetry.example.test/simple/requests/"]
    );
}

#[test]
fn pyproject_poetry_dependency_source_overrides_python_registry_url() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: package_file_fixture(
            "pyproject-poetry-dependency-source-overrides-python-registry-url.toml",
        ),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[1].name, "private");
    assert_eq!(dependencies[1].hosted_url, Some("private".to_owned()));
    let context = crate::registry::registry_context_from_document(&input);
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

    let input = DocumentInput {
        uri: format!("file://{}", root.join("pyproject.toml").display()),
        language_id: "toml".to_owned(),
        text: package_file_fixture("python-documents-use-workspace-uv-toml-registry-urls.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://primary.example.test/simple/requests/",
            "https://extra.example.test/simple/requests/",
            "https://private.example.test/simple/requests/",
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn podfile_external_and_latest_dependencies_resolve_as_fixed_without_registry_updates() {
    let input = DocumentInput {
        uri: "file:///repo/Podfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: package_file_fixture(
            "podfile-external-and-latest-dependencies-resolve-as-fixed-without-registry-updatesPodfile",
        ),
        workspace_root: None,
    };
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
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/fixed/registry_sources")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}
