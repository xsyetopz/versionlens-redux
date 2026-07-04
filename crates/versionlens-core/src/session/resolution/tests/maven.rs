use super::{
    DocumentInput, Ecosystem, ProviderSettings, RegistryResponseInput, RegistryUrlConfig,
    parse_document, session_with_settings,
};
use crate::registry::RegistryContext;

#[test]
fn maven_registry_urls_preserve_configured_fallback_order() {
    let session = session_with_settings(
        ProviderSettings {
            registry_urls: vec![
                RegistryUrlConfig {
                    ecosystem: Ecosystem::Maven,
                    url: "https://mirror.example.test/maven2".to_owned(),
                },
                RegistryUrlConfig {
                    ecosystem: Ecosystem::Maven,
                    url: "https://repo.maven.apache.org/maven2".to_owned(),
                },
            ],
            ..ProviderSettings::default()
        },
        false,
    );
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<project><dependencies><dependency><groupId>org.example</groupId><artifactId>demo</artifactId><version>1.0.0</version></dependency></dependencies></project>"#
            .to_owned(),
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
                ecosystem: Ecosystem::Maven,
                body: "<metadata><versioning><versions></versions></versioning></metadata>"
                    .to_owned(),
            },
            RegistryResponseInput {
                package: "org.example:demo".to_owned(),
                ecosystem: Ecosystem::Maven,
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
    let session = session_with_settings(ProviderSettings::default(), false);
    let input = DocumentInput {
        uri: "file:///pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: r#"<project>
  <repositories>
    <repository><url>https://packages.example.test/maven</url></repository>
  </repositories>
  <profiles>
    <profile>
      <repositories>
        <repository><url>https://profile.example.test/releases</url></repository>
      </repositories>
    </profile>
  </profiles>
  <dependencies>
    <dependency>
      <groupId>org.example</groupId>
      <artifactId>demo</artifactId>
      <version>1.0.0</version>
    </dependency>
  </dependencies>
</project>"#
            .to_owned(),
        workspace_root: None,
    };
    let dependencies = parse_document(&input);
    let context = RegistryContext::from_document(&input);

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
fn maven_documents_use_workspace_settings_repositories_and_auth() {
    let root =
        std::env::temp_dir().join(format!("versionlens-maven-settings-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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

    let session = session_with_settings(ProviderSettings::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: r#"<project><dependencies><dependency><groupId>org.example</groupId><artifactId>demo</artifactId><version>1.0.0</version></dependency></dependencies></project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = RegistryContext::from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Maven,
        "https://maven.example.test/repository/releases/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic dXNlcjpwYXNz");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn maven_local_repository_metadata_resolves_before_remote_registries() {
    let root = std::env::temp_dir().join(format!("versionlens-maven-local-{}", std::process::id()));
    let local_repo = root.join(".m2").join("repository");
    let metadata_dir = local_repo.join("org").join("example").join("demo");
    std::fs::create_dir_all(&metadata_dir).unwrap();
    std::fs::write(
        root.join("settings.xml"),
        format!(
            "<settings><localRepository>{}</localRepository></settings>",
            local_repo.display()
        ),
    )
    .unwrap();
    std::fs::write(
        metadata_dir.join("maven-metadata.xml"),
        "<metadata><versioning><versions><version>1.1.0</version></versions></versioning></metadata>",
    )
    .unwrap();

    let session = session_with_settings(ProviderSettings::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: r#"<project><dependencies><dependency><groupId>org.example</groupId><artifactId>demo</artifactId><version>1.0.0</version></dependency></dependencies></project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let output = session.resolve_document(input);

    assert_eq!(output.suggestions[0].status, "updateAvailable");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.0"));
    assert_eq!(output.edits[0].new_text, "1.1.0");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn maven_settings_mirror_overrides_pom_and_settings_repositories() {
    let root =
        std::env::temp_dir().join(format!("versionlens-maven-mirror-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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

    let session = session_with_settings(ProviderSettings::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: r#"<project><repositories><repository><url>https://pom.example.test/maven</url></repository></repositories><dependencies><dependency><groupId>org.example</groupId><artifactId>demo</artifactId><version>1.0.0</version></dependency></dependencies></project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = RegistryContext::from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://maven.example.test/mirror/org/example/demo/maven-metadata.xml"]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Maven,
        "https://maven.example.test/mirror/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn maven_settings_exact_mirror_replaces_matching_repository_only() {
    let root = std::env::temp_dir().join(format!(
        "versionlens-maven-exact-mirror-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
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

    let session = session_with_settings(ProviderSettings::default(), false);
    let input = DocumentInput {
        uri: format!("file://{}", root.join("pom.xml").display()),
        language_id: "xml".to_owned(),
        text: r#"<project><repositories><repository><id>private</id><url>https://private.example.test/maven</url></repository><repository><id>public</id><url>https://public.example.test/maven</url></repository></repositories><dependencies><dependency><groupId>org.example</groupId><artifactId>demo</artifactId><version>1.0.0</version></dependency></dependencies></project>"#
            .to_owned(),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let dependencies = parse_document(&input);
    let context = RegistryContext::from_document(&input);

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
            "https://public.example.test/maven/org/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/org/example/demo/maven-metadata.xml",
        ]
    );

    let headers = context.auth_headers_for_url(
        Ecosystem::Maven,
        "https://maven.example.test/private-mirror/org/example/demo/maven-metadata.xml",
    );
    assert_eq!(headers.len(), 1);
    assert_eq!(headers[0].value, "Basic bWlycm9yLXVzZXI6bWlycm9yLXBhc3M=");

    std::fs::remove_dir_all(root).unwrap();
}
