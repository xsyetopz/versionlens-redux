use super::{
    DocumentInput, ProviderSettings, RegistryResponseInput, RegistryUrlConfig, parse_document,
    session_with_settings,
};
use std::env;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use versionlens_parsers::Ecosystem::Maven;

#[test]
fn maven_registry_urls_preserve_configured_fallback_order() {
    let session = session_with_settings(
        ProviderSettings {
            registry_urls: vec![
                RegistryUrlConfig {
                    ecosystem: Maven,
                    url: "https://mirror.example.test/maven2".to_owned(),
                },
                RegistryUrlConfig {
                    ecosystem: Maven,
                    url: "https://repo.maven.apache.org/maven2".to_owned(),
                },
            ],
            ..crate::default()
        },
        false,
    );
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture("maven-registry-urls-preserve-configured-fallback-order.xml"),
        workspace_root: None,
    };
    let dependencies = parse_document(&input);

    assert_eq!(
        session.registry_urls(&dependencies[0]),
        vec![
            "https://mirror.example.test/maven2/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    let output = session.resolve_document_with_responses(
        input,
        &[
            RegistryResponseInput {
                package: "org.example:demo".to_owned(),
                ecosystem: Maven,
                body: "<metadata><versioning><versions></versions></versioning></metadata>"
                    .to_owned(),
            },
            RegistryResponseInput {
                package: "org.example:demo".to_owned(),
                ecosystem: Maven,
                body: "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>"
                    .to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.edits[0].new_text, "1.1.0");
}

#[test]
fn maven_registry_urls_include_pom_repositories_before_central() {
    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-registry-urls-include-pom-repositories-before-central.xml",
        ),
        workspace_root: None,
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://packages.example.test/maven/org/example/demo/maven-metadata.xml",
            "https://profile.example.test/releases/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );
}

#[test]
fn maven_registry_urls_include_pom_plugin_repositories_before_central() {
    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-registry-urls-include-pom-plugin-repositories-before-central.xml",
        ),
        workspace_root: None,
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
        ]
    );
}

#[test]
fn maven_registry_urls_resolve_project_and_parent_interpolation_properties() {
    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-registry-urls-resolve-project-and-parent-interpolation-properties.xml",
        ),
        workspace_root: None,
    };
    let dependencies = parse_document(&input);

    assert_eq!(dependencies[1].name, "org.parent:runtime");
    assert_eq!(dependencies[1].requirement, "3.4.5");
    assert_eq!(
        session.registry_urls(&dependencies[1]),
        vec!["https://repo.maven.apache.org/maven2/org/parent/runtime/maven-metadata.xml"]
    );
}

#[test]
fn maven_documents_use_workspace_settings_repositories_and_auth() {
    let root = temp_dir().join(format!("versionlens-maven-settings-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        r#"<settings>
  <servers>
    <server>
      <id>private</id>
      <username>user</username>
      <password>pass</password>
    </server>
  </servers>
  <profiles>
    <profile>
      <repositories>
        <repository>
          <id>private</id>
          <url>https://maven.example.test/repository/releases</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>"#,
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-documents-use-workspace-settings-repositories-and-auth.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    let headers = context.auth_headers_for_url(
        Maven,
        "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_documents_use_workspace_settings_plugin_repositories_and_auth() {
    let root = temp_dir().join(format!("versionlens-maven-plugin-settings-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        r#"<settings>
  <servers>
    <server>
      <id>private-plugins</id>
      <username>plugin-user</username>
      <password>plugin-pass</password>
    </server>
  </servers>
  <profiles>
    <profile>
      <pluginRepositories>
        <pluginRepository>
          <id>private-plugins</id>
          <url>https://plugins.example.test/maven</url>
        </pluginRepository>
      </pluginRepositories>
    </profile>
  </profiles>
</settings>"#,
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-documents-use-workspace-settings-plugin-repositories-and-auth.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
        ]
    );

    let headers = context.auth_headers_for_url(
        Maven,
        "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic cGx1Z2luLXVzZXI6cGx1Z2luLXBhc3M=");

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_documents_use_only_active_workspace_settings_profile_repositories() {
    let root = temp_dir().join(format!(
        "versionlens-maven-active-profile-settings-{}",
        id()
    ));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        r#"<settings>
  <profiles>
    <profile>
      <id>active</id>
      <repositories>
        <repository>
          <id>active-repo</id>
          <url>https://active.example.test/maven</url>
        </repository>
      </repositories>
    </profile>
    <profile>
      <id>inactive</id>
      <repositories>
        <repository>
          <id>inactive-repo</id>
          <url>https://inactive.example.test/maven</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
  <activeProfiles>
    <activeProfile>active</activeProfile>
  </activeProfiles>
</settings>"#,
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-documents-use-only-active-workspace-settings-profile-repositories.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://active.example.test/maven/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_local_repository_metadata_resolves_before_remote_registries() {
    let root = temp_dir().join(format!("versionlens-maven-local-{}", id()));
    let local_repo = root.join(".m2").join("repository");
    let metadata_dir = local_repo.join("org").join("example").join("demo");
    create_dir_all(&metadata_dir).unwrap();
    write(
        root.join("settings.xml"),
        format!(
            "<settings><localRepository>{}</localRepository></settings>",
            local_repo.display()
        ),
    )
    .unwrap();
    write(
        metadata_dir.join("maven-metadata.xml"),
        "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>",
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-local-repository-metadata-resolves-before-remote-registries.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let output = session.resolve_document(input);

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.0"));
    assert_eq!(output.edits[0].new_text, "1.1.0");

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_settings_mirror_overrides_pom_and_settings_repositories() {
    let root = temp_dir().join(format!("versionlens-maven-mirror-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        r#"<settings>
  <servers>
    <server>
      <id>internal</id>
      <username>mirror-user</username>
      <password>mirror-pass</password>
    </server>
  </servers>
  <mirrors>
    <mirror>
      <id>internal</id>
      <mirrorOf>*</mirrorOf>
      <url>https://maven.example.test/mirror</url>
    </mirror>
  </mirrors>
  <profiles>
    <profile>
      <repositories>
        <repository>
          <id>private</id>
          <url>https://maven.example.test/repository/releases</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>"#,
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-settings-mirror-overrides-pom-and-settings-repositories.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://maven.example.test/mirror/org/example/demo/maven-metadata.xml"]
    );

    let headers = context.auth_headers_for_url(
        Maven,
        "https://maven.example.test/mirror/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_settings_exact_mirror_replaces_matching_repository_only() {
    let root = temp_dir().join(format!("versionlens-maven-exact-mirror-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        r#"<settings>
  <servers>
    <server>
      <id>private-mirror</id>
      <username>mirror-user</username>
      <password>mirror-pass</password>
    </server>
  </servers>
  <mirrors>
    <mirror>
      <id>private-mirror</id>
      <mirrorOf>private</mirrorOf>
      <url>https://maven.example.test/private-mirror</url>
    </mirror>
  </mirrors>
</settings>"#,
    )
    .unwrap();

    let session = session_with_settings(crate::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: package_file_fixture(
            "maven-settings-exact-mirror-replaces-matching-repository-only.txt",
        ),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = crate::registry::registry_context_from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
            "https://public.example.test/maven/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    let headers = context.auth_headers_for_url(
        Maven,
        "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

    remove_dir_all(root).unwrap();
}

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/maven")
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
