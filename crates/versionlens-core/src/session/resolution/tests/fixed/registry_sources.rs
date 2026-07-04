use super::{DocumentInput, standard_session};

#[test]
fn requirements_index_urls_are_ignored_for_upstream_parity() {
    let input = DocumentInput {
        uri: "file:///repo/requirements.txt".to_owned(),
        language_id: "pip-requirements".to_owned(),
        text: "--index-url https://primary.example.test/simple\n--extra-index-url https://extra.example.test/simple\nrequests==2.32.0\n".to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    let requests = dependencies
        .iter()
        .find(|dependency| dependency.name == "requests")
        .unwrap();

    assert_eq!(
        session.registry_urls_with_context(requests, &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );
}

#[test]
fn pipfile_sources_are_ignored_for_upstream_parity() {
    let input = DocumentInput {
        uri: "file:///repo/Pipfile".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[[source]]
name = "private"
url = "https://pypi.example.test/simple/"
verify_ssl = true

[packages]
requests = "==2.32.0"
"#
        .to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );
}

#[test]
fn python_documents_ignore_workspace_pip_conf_for_upstream_parity() {
    let root = std::env::temp_dir().join(format!("versionlens-pip-conf-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("pip.conf"),
        "[global]\nindex-url = https://primary.example.test/simple/\nextra-index-url = https://extra.example.test/simple\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("requirements.txt").display()),
        language_id: "pip-requirements".to_owned(),
        text: "requests==2.32.0\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn python_documents_ignore_pip_environment_for_upstream_parity() {
    let root = std::env::temp_dir().join(format!("versionlens-pip-env-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".env"),
        "PIP_INDEX_URL=https://env-primary.example.test/simple/\nPIP_EXTRA_INDEX_URL=https://env-extra-one.example.test/simple https://env-extra-two.example.test/simple/\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("requirements.txt").display()),
        language_id: "pip-requirements".to_owned(),
        text: "requests==2.32.0\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gemfile_source_urls_override_ruby_registry_urls() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: "source 'https://gems.example.test/'\ngem 'rails', '8.1.3'\n".to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "rails");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://gems.example.test"]
    );
}

#[test]
fn gemfile_source_blocks_override_dependency_registry_url() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: r#"
source "https://private.gems.example.test/" do
  gem "private_gem", "1.0.0"
end
"#
        .to_owned(),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://private.gems.example.test"]
    );
}

#[test]
fn gemfile_source_option_overrides_dependency_registry_url() {
    let input = DocumentInput {
        uri: "file:///repo/Gemfile".to_owned(),
        language_id: "ruby".to_owned(),
        text: r#"
gem "private_gem", "1.0.0", source: "https://private.gems.example.test/"
"#
        .to_owned(),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "private_gem");
    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec!["https://private.gems.example.test"]
    );
}

#[test]
fn pyproject_uv_indexes_are_ignored_for_upstream_parity() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[project]
dependencies = ["requests==2.32.0"]

[tool.uv]
index-url = "https://primary.example.test/simple/"
extra-index-url = ["https://extra.example.test/simple"]

[[tool.uv.index]]
name = "private"
url = "https://private.example.test/simple/"
"#
        .to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );
}

#[test]
fn pyproject_poetry_sources_are_ignored_for_upstream_parity() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[tool.poetry.dependencies]
requests = "^2.32"

[[tool.poetry.source]]
name = "private"
url = "https://poetry.example.test/simple/"
priority = "primary"
"#
        .to_owned(),
        workspace_root: None,
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].name, "requests");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );
}

#[test]
fn pyproject_poetry_dependency_source_is_ignored_for_upstream_parity() {
    let input = DocumentInput {
        uri: "file:///repo/pyproject.toml".to_owned(),
        language_id: "toml".to_owned(),
        text: r#"
[tool.poetry.dependencies]
public = "^1.0"
private = { version = "^2.0", source = "private" }

[[tool.poetry.source]]
name = "private"
url = "https://private.example.test/simple/"
priority = "explicit"

[[tool.poetry.source]]
name = "mirror"
url = "https://mirror.example.test/simple/"
priority = "supplemental"
"#
        .to_owned(),
        workspace_root: None,
    };
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[1].name, "private");
    assert_eq!(dependencies[1].hosted_url, None);
    assert_eq!(
        session.registry_urls(&dependencies[1]),
        vec!["https://pypi.org/rss/project/private/releases.xml"]
    );
}

#[test]
fn python_documents_ignore_workspace_uv_toml_for_upstream_parity() {
    let root = std::env::temp_dir().join(format!("versionlens-uv-toml-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("uv.toml"),
        "index-url = 'https://primary.example.test/simple/'\nextra-index-url = ['https://extra.example.test/simple']\n[[index]]\nname = 'private'\nurl = 'https://private.example.test/simple/'\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("pyproject.toml").display()),
        language_id: "toml".to_owned(),
        text: "[project]\ndependencies = ['requests==2.32.0']\n".to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::RegistryContext::from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://pypi.org/rss/project/requests/releases.xml"]
    );

    std::fs::remove_dir_all(root).unwrap();
}
