#[test]
fn sbt_resolvers_are_used_before_maven_central() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///build.sbt".to_owned(),
        language_id: "scala".to_owned(),
        text: package_file_fixture("sbt-resolvers-are-used-before-maven-central.sbt"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "com.example:demo");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://maven.example.test/releases/com/example/demo/maven-metadata.xml",
            "https://repo.maven.apache.org/maven2/com/example/demo/maven-metadata.xml"
        ]
    );
}

#[test]
fn clojure_deps_edn_uses_maven_central_then_clojars() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///deps.edn".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture("clojure-deps-edn-uses-maven-central-then-clojars.edn"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[0].name, "metosin:malli");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec![
            "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
            "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
        ]
    );
}

#[test]
fn leiningen_project_clj_uses_maven_central_then_clojars() {
    let session = crate::version_lens_session(SessionConfig {
        cache_ttl_ms: 300_000,
        enabled_providers: vec![],
        providers: crate::default(),
        suggestion_indicators: crate::standard_suggestion_indicators(),
        show_vulnerabilities: false,
        show_suggestion_stats: false,
        show_prereleases: false,
        http: versionlens_http::standard_http_config(),
    });
    let input = DocumentInput {
        uri: "file:///project.clj".to_owned(),
        language_id: "clojure".to_owned(),
        text: package_file_fixture("leiningen-project-clj-uses-maven-central-then-clojars.clj"),
        workspace_root: None,
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = session.dependencies(&input);

    assert_eq!(dependencies[1].name, "metosin:malli");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec![
            "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
            "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
        ]
    );
}
