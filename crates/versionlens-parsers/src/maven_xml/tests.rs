use crate::{
    DocumentInput, Ecosystem, document::test_support::extract_range, parse_document,
    parse_document_with_dependency_paths,
};

use super::{
    extract_maven_repository_urls, parse_maven_effective_settings_https_repositories,
    parse_maven_effective_settings_https_repository_sources,
    parse_maven_effective_settings_repositories, parse_maven_effective_settings_repository_sources,
    parse_maven_metadata_versions, parse_maven_pom_repositories, parse_maven_pom_repository_urls,
    parse_maven_settings_auth_entries, parse_maven_settings_mirror_urls,
    parse_maven_settings_mirrors, parse_maven_settings_repository_urls,
};

#[test]
fn parses_maven_pom_dependencies() {
    let text = r#"<project>
  <version>1.3.6-SNAPSHOT</version>
  <properties>
    <tomcat.version>9.0.12</tomcat.version>
    <tomcat.artifactId>tomcat</tomcat.artifactId>
  </properties>
  <parent>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-parent</artifactId>
    <version>1.5.16.RELEASE</version>
  </parent>
  <dependencies>
    <dependency>
      <groupId>org.springframework</groupId>
      <artifactId>spring-core</artifactId>
      <version>5.0.7.RELEASE</version>
    </dependency>
    <dependency>
      <groupId>org.apache.tomcat</groupId>
      <artifactId>${tomcat.artifactId}</artifactId>
      <version>${tomcat.version}</version>
    </dependency>
  </dependencies>
</project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Maven);
    assert_eq!(dependencies[0].group, "project.version");
    assert_eq!(dependencies[0].name, "version");
    assert_eq!(dependencies[0].requirement, "1.3.6-SNAPSHOT");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.3.6-SNAPSHOT"
    );
    assert_eq!(
        dependencies[1].name,
        "org.springframework.boot:spring-boot-starter-parent"
    );
    assert_eq!(dependencies[1].requirement, "1.5.16.RELEASE");
    assert_eq!(dependencies[2].name, "org.springframework:spring-core");
    assert_eq!(dependencies[2].requirement, "5.0.7.RELEASE");
    assert_eq!(dependencies[2].range.start.line, 12);
    assert_eq!(dependencies[2].range.start.character, 4);
    assert_eq!(dependencies[2].range.end.line, 16);
    assert_eq!(dependencies[2].range.end.character, 17);
    assert_eq!(
        extract_range(text, dependencies[2].requirement_range),
        "5.0.7.RELEASE"
    );
    assert_eq!(dependencies[3].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[3].requirement, "9.0.12");
    assert_eq!(
        extract_range(text, dependencies[3].requirement_range),
        "9.0.12"
    );
}

#[test]
fn maven_property_references_trim_before_resolution_like_upstream() {
    let text = r#"<project>
  <properties>
    <example.group>com.example</example.group>
    <example.artifact>demo</example.artifact>
    <example.version>1.2.3</example.version>
  </properties>
  <dependencies>
    <dependency>
      <groupId> ${example.group} </groupId>
      <artifactId> ${example.artifact} </artifactId>
      <version> ${example.version} </version>
    </dependency>
  </dependencies>
</project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.2.3"
    );
}

#[test]
fn parses_smoke_maven_pom_smoke_shapes() {
    let text = r#"<project
  xmlns="http://maven.apache.org/POM/4.0.0"
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:schemaLocation="http://maven.apache.org/POM/4.0.0 http://maven.apache.org/maven-v4_0_0.xsd"
>
  <modelVersion>4.0.0</modelVersion>
  <groupId>vscode-contrib</groupId>
  <artifactId>vscode-versionlens</artifactId>
  <packaging>war</packaging>
  <version>1.3.6-SNAPSHOT</version>
  <name>smoke-test</name>
  <properties>
    <tomcat.groupId>org.apache.tomcat</tomcat.groupId>
    <tomcat.artifactId>tomcat</tomcat.artifactId>
    <tomcat.version>11.0.23</tomcat.version>
  </properties>
  <parent>
    <groupId>org.springframework.boot</groupId>
    <artifactId>spring-boot-starter-parent</artifactId>
    <version>4.1.0</version>
  </parent>
  <dependencies>
    <dependency>
      <groupId>org.springframework</groupId>
      <artifactId>spring-core</artifactId>
      <version>7.0.8</version>
    </dependency>
    <dependency>
      <groupId>junit</groupId>
      <artifactId>junit</artifactId>
      <version>4.13.2</version>
    </dependency>
    <dependency>
      <groupId>${tomcat.groupId}</groupId>
      <artifactId>${tomcat.artifactId}</artifactId>
      <version>${tomcat.version}</version>
      <type>pom</type>
    </dependency>
    <dependency>
      <groupId>com.oracle</groupId>
      <artifactId>ojdbc6</artifactId>
      <version>10.0</version>
    </dependency>
    <dependency>
      <groupId>crespo.fernando</groupId>
      <artifactId>condominio</artifactId>
      <version>*</version>
    </dependency>
    <dependency>
      <groupId>crespo.fernando</groupId>
      <artifactId>other</artifactId>
      <version>*</version>
    </dependency>
  </dependencies>
  <repositories>
    <repository>
      <url>https://packages.atlassian.com/maven-3rdparty/</url>
    </repository>
  </repositories>
</project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 8);
    assert_eq!(dependencies[0].group, "project.version");
    assert_eq!(dependencies[0].requirement, "1.3.6-SNAPSHOT");
    assert_eq!(
        dependencies[1].name,
        "org.springframework.boot:spring-boot-starter-parent"
    );
    assert_eq!(dependencies[1].requirement, "4.1.0");
    assert_eq!(dependencies[4].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[4].requirement, "11.0.23");
    assert_eq!(dependencies[5].name, "com.oracle:ojdbc6");
    assert_eq!(dependencies[5].requirement, "10.0");
    assert_eq!(dependencies[6].name, "crespo.fernando:condominio");
    assert_eq!(dependencies[6].requirement, "*");
    assert_eq!(dependencies[7].name, "crespo.fernando:other");
    assert_eq!(dependencies[7].requirement, "*");
}

#[test]
fn parses_maven_dependency_management_dependencies_when_configured() {
    let text = r#"<project>
  <dependencyManagement>
    <dependencies>
      <dependency>
        <groupId>org.example</groupId>
        <artifactId>managed</artifactId>
        <version>2.3.4</version>
      </dependency>
    </dependencies>
  </dependencyManagement>
</project>"#;
    let default_dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });
    assert_eq!(default_dependencies.len(), 0);

    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project.dependencyManagement.dependencies.dependency"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(
        dependencies[0].group,
        "project.dependencyManagement.dependencies.dependency"
    );
    assert_eq!(dependencies[0].name, "org.example:managed");
    assert_eq!(dependencies[0].requirement, "2.3.4");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "2.3.4"
    );
}

#[test]
fn parses_configured_maven_plugin_dependency_paths() {
    let text = r#"<project>
  <build>
    <plugins>
      <plugin>
        <groupId>org.apache.maven.plugins</groupId>
        <artifactId>maven-compiler-plugin</artifactId>
        <version>3.14.0</version>
      </plugin>
    </plugins>
  </build>
</project>"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project.build.plugins.plugin"],
    );

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "project.build.plugins.plugin");
    assert_eq!(
        dependencies[0].name,
        "org.apache.maven.plugins:maven-compiler-plugin"
    );
    assert_eq!(dependencies[0].requirement, "3.14.0");
}

#[test]
fn configured_maven_dependency_paths_match_exact_nodes_only() {
    let text = r#"<project>
  <dependencies>
    <dependency>
      <groupId>org.springframework</groupId>
      <artifactId>spring-core</artifactId>
      <version>5.0.7.RELEASE</version>
    </dependency>
  </dependencies>
</project>"#;
    let dependencies = parse_document_with_dependency_paths(
        &DocumentInput {
            uri: "file:///work/pom.xml".to_owned(),
            language_id: "xml".to_owned(),
            text: text.to_owned(),
            workspace_root: None,
        },
        &["project.dependencies"],
    );

    assert_eq!(dependencies.len(), 0);
}

#[test]
fn maven_property_resolution_uses_first_matching_property() {
    let text = r#"<project>
  <properties>
    <tomcat.version>9.0.12</tomcat.version>
    <tomcat.version>11.0.23</tomcat.version>
  </properties>
  <dependencies>
    <dependency>
      <groupId>org.apache.tomcat</groupId>
      <artifactId>tomcat</artifactId>
      <version>${tomcat.version}</version>
    </dependency>
  </dependencies>
</project>"#;
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/pom.xml".to_owned(),
        language_id: "xml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "org.apache.tomcat:tomcat");
    assert_eq!(dependencies[0].requirement, "9.0.12");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "9.0.12"
    );
}

#[test]
fn parses_maven_pom_repository_urls() {
    let text = r#"<project>
  <repositories>
    <repository>
      <url>https://packages.example.test/maven</url>
    </repository>
    <repository>
      <url>https://repo.maven.apache.org/maven2</url>
    </repository>
  </repositories>
  <profiles>
    <profile>
      <repositories>
        <repository>
          <url>https://profile.example.test/releases</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</project>"#;

    assert_eq!(
        parse_maven_pom_repository_urls(text),
        vec![
            "https://packages.example.test/maven",
            "https://repo.maven.apache.org/maven2",
            "https://profile.example.test/releases",
        ]
    );
}

#[test]
fn parses_maven_effective_settings_repositories() {
    let text = r#"
[INFO] Effective user-specific configuration settings:
<?xml version="1.0" encoding="UTF-8"?>
<settings>
  <localRepository>/Users/example/.m2/repository</localRepository>
  <profiles>
    <profile>
      <repositories>
        <repository>
          <url>https://repo1.maven.org/maven2</url>
        </repository>
        <repository>
          <url>http://repo.example.test/releases</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>
"#;

    assert_eq!(
        parse_maven_effective_settings_repositories(text),
        vec![
            "/Users/example/.m2/repository",
            "https://repo1.maven.org/maven2",
            "http://repo.example.test/releases",
        ]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repositories(text),
        vec!["https://repo1.maven.org/maven2"]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repositories(""),
        vec!["https://repo.maven.apache.org/maven2/"]
    );
}

#[test]
fn parses_maven_metadata_versions() {
    let text = r#"<metadata>
  <groupId>org.springframework</groupId>
  <artifactId>spring-core</artifactId>
  <versioning>
    <versions>
      <version>5.0.7.RELEASE</version>
      <version>5.1.0.RELEASE</version>
    </versions>
  </versioning>
</metadata>"#;

    assert_eq!(
        parse_maven_metadata_versions(text),
        vec!["5.0.7.RELEASE", "5.1.0.RELEASE"]
    );
}

#[test]
fn parses_maven_repository_sources() {
    let text = r#"
<?xml version="1.0" encoding="UTF-8"?>
<settings>
  <localRepository>/Users/example/.m2/repository</localRepository>
  <profiles>
    <profile>
      <repositories>
        <repository><url>https://repo1.maven.org/maven2</url></repository>
        <repository><url>http://repo.example.test/releases</url></repository>
      </repositories>
    </profile>
  </profiles>
</settings>
"#;

    let sources = parse_maven_effective_settings_repository_sources(text);

    assert_eq!(sources.len(), 3);
    assert_eq!(sources[0].url, "/Users/example/.m2/repository");
    assert_eq!(sources[0].protocol, "file:");
    assert_eq!(sources[1].protocol, "https:");
    assert_eq!(sources[2].protocol, "http:");
    assert_eq!(
        extract_maven_repository_urls(&sources),
        vec![
            "/Users/example/.m2/repository",
            "https://repo1.maven.org/maven2",
            "http://repo.example.test/releases",
        ]
    );
    assert_eq!(
        parse_maven_effective_settings_https_repository_sources(text),
        vec![sources[1].clone()]
    );
    assert_eq!(
        parse_maven_effective_settings_repository_sources(""),
        vec![super::MavenRepository {
            url: "https://repo.maven.apache.org/maven2/".to_owned(),
            protocol: "https:".to_owned(),
        }]
    );
}

#[test]
fn parses_maven_settings_repositories_and_auth_entries() {
    let text = r#"<settings>
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
        <repository>
          <id>public</id>
          <url>https://repo.maven.apache.org/maven2</url>
        </repository>
      </repositories>
    </profile>
  </profiles>
</settings>"#;

    assert_eq!(
        parse_maven_settings_repository_urls(text),
        vec![
            "https://maven.example.test/repository/releases",
            "https://repo.maven.apache.org/maven2",
        ]
    );

    let entries = parse_maven_settings_auth_entries(text);
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].registry,
        "https://maven.example.test/repository/releases"
    );
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
}

#[test]
fn parses_maven_settings_local_repository_as_named_repository() {
    let text = r#"<settings>
  <localRepository>/Users/example/.m2/repository</localRepository>
</settings>"#;

    let repositories = parse_maven_settings_mirrors(text);
    assert!(repositories.is_empty());
    assert_eq!(
        parse_maven_settings_repository_urls(text),
        vec!["/Users/example/.m2/repository"]
    );
}

#[test]
fn parses_maven_settings_mirrors() {
    let text = r#"<settings>
  <mirrors>
    <mirror>
      <id>internal</id>
      <mirrorOf>*</mirrorOf>
      <url>https://maven.example.test/mirror</url>
    </mirror>
  </mirrors>
</settings>"#;

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
</project>"#,
    );

    assert_eq!(repositories.len(), 2);
    assert_eq!(repositories[0].id, "private");
    assert_eq!(
        repositories[0].url,
        "https://maven.example.test/repository/releases"
    );
    assert_eq!(repositories[1].id, "");
    assert_eq!(repositories[1].url, "https://anonymous.example.test/maven");
}
