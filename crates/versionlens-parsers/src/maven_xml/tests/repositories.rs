#[test]
fn effective_maven_settings_repositories_ignore_inactive_profiles() {
    let text = package_file_fixture("effective-maven-settings-repositories-ignore-inactive-profiles.txt");

    assert_eq!(
        parse_maven_effective_settings_repositories(text),
        vec!["https://active.example.test/maven"]
    );
}

#[test]
fn parses_maven_settings_local_repository_as_named_repository() {
    let text = package_file_fixture("maven-settings-local-repository-as-named-repository.txt");

    let repositories = parse_maven_settings_mirrors(text);
    assert!(repositories.is_empty());
    assert_eq!(
        parse_maven_settings_repository_urls(text),
        vec!["/Users/example/.m2/repository"]
    );
}

#[test]
fn parses_maven_settings_mirrors() {
    let text = package_file_fixture("maven-settings-mirrors.txt");

    let mirrors = parse_maven_settings_mirrors(text);
    assert_eq!(mirrors.len(), 1);
    assert_eq!(mirrors[0].id, "internal");
    assert_eq!(mirrors[0].mirror_of, "*");
    assert_eq!(mirrors[0].url, "https://maven.example.test/mirror");
    assert_eq!(
        parse_maven_settings_mirror_urls(text),
        vec!["https://maven.example.test/mirror"]
    );
}

#[test]
fn parses_maven_pom_repositories_with_ids() {
    let repositories = parse_maven_pom_repositories(
        r#"<project>
  <repositories>
    <repository>
      <id>private</id>
      <url>https://maven.example.test/repository/releases</url>
    </repository>
    <repository>
      <url>https://anonymous.example.test/maven</url>
    </repository>
  </repositories>
  <pluginRepositories>
    <pluginRepository>
      <id>plugins</id>
      <url>https://plugins.example.test/maven</url>
    </pluginRepository>
  </pluginRepositories>
</project>"#,
    );

    assert_eq!(repositories.len(), 3);
    assert_eq!(repositories[0].id, "private");
    assert_eq!(
        repositories[0].url,
        "https://maven.example.test/repository/releases"
    );
    assert_eq!(repositories[1].id, "");
    assert_eq!(repositories[1].url, "https://anonymous.example.test/maven");
    assert_eq!(repositories[2].id, "plugins");
    assert_eq!(repositories[2].url, "https://plugins.example.test/maven");
}
