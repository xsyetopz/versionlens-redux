#[test]
fn sbt_resolvers_are_used_before_maven_central() {
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///build.sbt", "scala", "sbt-resolvers-are-used-before-maven-central.sbt");
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

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
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///deps.edn", "clojure", "clojure-deps-edn-uses-maven-central-then-clojars.edn");
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

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
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///project.clj", "clojure", "leiningen-project-clj-uses-maven-central-then-clojars.clj");
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert_eq!(dependencies[1].name, "metosin:malli");
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec![
            "https://repo.maven.apache.org/maven2/metosin/malli/maven-metadata.xml",
            "https://repo.clojars.org/metosin/malli/maven-metadata.xml"
        ]
    );
}
