use super::*;

#[test]
fn maven_registry_urls_preserve_configured_fallback_order() {
    let session = crate::support::tests::session_with_provider_settings(
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
    let input = DocumentInput::new(
        "file:///pom.xml".to_owned(),
        "xml".to_owned(),
        package_file_fixture("urls-preserve-configured-fallback-order.xml"),
        None,
    );
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
            RegistryResponseInput::new("org.example:demo".to_owned(), Maven, "<metadata><versioning><versions></versions></versioning></metadata>"
                    .to_owned()),
            RegistryResponseInput::new("org.example:demo".to_owned(), Maven, "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>"
                    .to_owned()),
        ],
    );

    assert_update(&output, "1.1.0");
}

#[test]
fn maven_registry_urls_include_pom_repositories_before_central() {
    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = DocumentInput::new(
        "file:///pom.xml".to_owned(),
        "xml".to_owned(),
        package_file_fixture("urls-include-pom-repositories-before-central.xml"),
        None,
    );
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);

    assert_maven_urls(
        &session,
        &context,
        &dependencies,
        &[
            "https://packages.example.test/maven/org/example/demo/maven-metadata.xml",
            "https://profile.example.test/releases/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ],
    );
}

#[test]
fn maven_registry_urls_include_pom_plugin_repositories_before_central() {
    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = DocumentInput::new(
        "file:///pom.xml".to_owned(),
        "xml".to_owned(),
        package_file_fixture("urls-include-pom-plugin-repositories-before-central.xml"),
        None,
    );
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);

    assert_maven_urls(
        &session,
        &context,
        &dependencies,
        &[
            "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
        ],
    );
}

#[test]
fn maven_registry_urls_resolve_project_and_parent_interpolation_properties() {
    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = DocumentInput::new(
        "file:///pom.xml".to_owned(),
        "xml".to_owned(),
        crate::support::tests::fixture(
            "tests/fixtures/versionlens-parsers/src/maven_xml/tests",
            "resolves-maven-project-and-parent-interpolation-properties.xml",
        ),
        None,
    );
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
        settings_with_private_repository(
            r#"<servers>
    <server>
      <id>private</id>
      <username>user</username>
      <password>pass</password>
    </server>
  </servers>"#,
        ),
    )
    .unwrap();

    let (session, context, dependencies) = workspace_dependencies(
        &root,
        "maven-documents-use-workspace-settings-repositories-and-auth.txt",
    );

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

    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = workspace_pom_input(
        &root,
        "maven-documents-use-workspace-settings-plugin-repositories-and-auth.txt",
    );
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);

    assert_maven_urls(
        &session,
        &context,
        &dependencies,
        &[
            "https://plugins.example.test/maven/org/example/demo-plugin/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo-plugin/maven-metadata.xml",
        ],
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

    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = workspace_pom_input(
        &root,
        "maven-documents-use-only-active-workspace-settings-profile-repositories.txt",
    );
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);

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

    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = workspace_pom_input(
        &root,
        "maven-local-repository-metadata-resolves-before-remote-registries.txt",
    );
    let output = session.resolve_document(input);

    crate::support::tests::assert_suggestion(&output, 0, "updateAvailable", Some("1.1.0"));
    assert_eq!(output.edits[0].new_text, "1.1.0");

    remove_dir_all(root).unwrap();
}

#[test]
fn maven_settings_mirror_overrides_pom_and_settings_repositories() {
    let root = temp_dir().join(format!("versionlens-maven-mirror-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("settings.xml"),
        settings_with_private_repository(
            r#"<servers>
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
  </mirrors>"#,
        ),
    )
    .unwrap();

    let (session, context, dependencies) = workspace_dependencies(
        &root,
        "maven-settings-mirror-overrides-pom-and-settings-repositories.txt",
    );

    assert_maven_urls(
        &session,
        &context,
        &dependencies,
        &["https://maven.example.test/mirror/org/example/demo/maven-metadata.xml"],
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

    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = workspace_pom_input(
        &root,
        "maven-settings-exact-mirror-replaces-matching-repository-only.txt",
    );
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);

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

fn workspace_pom_input(root: &std::path::Path, fixture: &str) -> DocumentInput {
    DocumentInput::new(
        format!("file://{}", root.join("pom.xml").display()),
        "xml".to_owned(),
        package_file_fixture(fixture),
        Some(root.to_string_lossy().into_owned()),
    )
}

fn workspace_dependencies(
    root: &std::path::Path,
    fixture: &str,
) -> (VersionLensSession, RegistryContext, Vec<Dependency>) {
    let session = crate::support::tests::session_with_provider_settings(crate::default(), false);
    let input = workspace_pom_input(root, fixture);
    let (context, dependencies) =
        crate::support::tests::registry_context_and_dependencies(&session, &input);
    (session, context, dependencies)
}

fn assert_maven_urls(
    session: &VersionLensSession,
    context: &RegistryContext,
    dependencies: &[Dependency],
    expected: &[&str],
) {
    super::assert_dependency_registry_url(session, dependencies, 0, context, expected);
}

fn settings_with_private_repository(prefix: &str) -> String {
    format!(
        r#"<settings>{prefix}
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
</settings>"#
    )
}

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/maven", name)
}
use super::assert_update;
