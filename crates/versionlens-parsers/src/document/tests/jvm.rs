use versionlens_model::Ecosystem::{Conan, Hackage, LuaRocks, Nim, Opam, Swift, Vcpkg, Zig};
#[test]
fn parses_clojure_deps_edn_dependencies() {
    let text = package_file_fixture("parses-clojure-deps-edn-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/deps.edn".to_owned(),
        language_id: "clojure".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "deps");
    assert_eq!(dependencies[0].name, "org.clojure:tools.reader");
    assert_eq!(dependencies[0].requirement, "1.1.1");
    assert_eq!(dependencies[1].name, "io.github.sally:awesome");
    assert_eq!(dependencies[1].requirement, "v1.2.3");
    assert_eq!(dependencies[1].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[2].name, "my.dev:project");
    assert_eq!(dependencies[2].requirement, "../project");
    assert_eq!(dependencies[2].hosted_url, Some("local".to_owned()));
    assert_eq!(dependencies[3].group, "aliases.test.extra-deps");
    assert_eq!(dependencies[3].name, "criterium:criterium");
    assert_eq!(dependencies[3].requirement, "0.4.4");
    assert_eq!(dependencies[4].group, "aliases.test.override-deps");
    assert_eq!(dependencies[4].name, "org.clojure:clojure");
    assert_eq!(dependencies[4].requirement, "1.11.3");
    assert_eq!(dependencies[5].group, "aliases.test.default-deps");
    assert_eq!(dependencies[5].name, "org.clojure:core.cache");
    assert_eq!(dependencies[5].requirement, "1.1.234");
}

#[test]
fn parses_clojure_deps_edn_mvn_repositories() {
    let repositories = parse_clojure_maven_repositories(
        r#"{:deps {com.example/demo {:mvn/version "1.0.0"}}
 :mvn/repos {"private" {:url "https://maven.example.test/releases"}
             "snapshots" {:url "https://maven.example.test/snapshots"}}}"#,
    );

    assert_eq!(repositories.len(), 2);
    assert_eq!(repositories[0].id, "private");
    assert_eq!(repositories[0].url, "https://maven.example.test/releases");
    assert_eq!(repositories[1].id, "snapshots");
    assert_eq!(repositories[1].url, "https://maven.example.test/snapshots");
}

#[test]
fn parses_leiningen_project_clj_dependencies() {
    let text = package_file_fixture("parses-leiningen-project-clj-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/project.clj".to_owned(),
        language_id: "clojure".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 7);
    assert_eq!(dependencies[0].ecosystem, Maven);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "0.1.0-SNAPSHOT");
    assert_eq!(dependencies[1].group, "dependencies");
    assert_eq!(dependencies[1].name, "org.clojure:clojure");
    assert_eq!(dependencies[1].requirement, "1.11.3");
    assert_eq!(dependencies[2].name, "net.3scale:3scale-api");
    assert_eq!(dependencies[2].requirement, "3.0.2");
    assert_eq!(dependencies[3].name, "cheshire:cheshire");
    assert_eq!(dependencies[3].requirement, "5.13.0");
    assert_eq!(dependencies[4].group, "managed-dependencies");
    assert_eq!(dependencies[4].name, "org.clojure:core.async");
    assert_eq!(dependencies[5].group, "plugins");
    assert_eq!(dependencies[5].name, "lein-ring:lein-ring");
    assert_eq!(dependencies[6].group, "profiles.dev.dependencies");
    assert_eq!(dependencies[6].name, "criterium:criterium");
}

#[test]
fn parses_leiningen_project_clj_repositories() {
    let repositories = parse_leiningen_maven_repositories(
        r#"(defproject demo "0.1.0"
  :repositories [["private" "https://maven.example.test/releases"]
                 ["snapshots" {:url "https://maven.example.test/snapshots"}]])"#,
    );

    assert_eq!(repositories.len(), 2);
    assert_eq!(repositories[0].id, "private");
    assert_eq!(repositories[0].url, "https://maven.example.test/releases");
    assert_eq!(repositories[1].id, "snapshots");
    assert_eq!(repositories[1].url, "https://maven.example.test/snapshots");
}

#[test]
fn parses_dune_project_package_dependencies() {
    let text = package_file_fixture("parses-dune-project-package-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dune-project".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Opam);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(dependencies[1].group, "depends");
    assert_eq!(dependencies[1].name, "ocaml");
    assert_eq!(dependencies[1].requirement, ">= 4.14");
    assert_eq!(dependencies[2].name, "fmt");
    assert_eq!(dependencies[2].requirement, ">= 0.6");
    assert_eq!(dependencies[3].name, "lwt");
    assert_eq!(dependencies[3].requirement, "latest");
}

#[test]
fn parses_opam_dependencies() {
    let text = package_file_fixture("parses-opam-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/demo.opam".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 6);
    assert_eq!(dependencies[0].ecosystem, Opam);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "1.2.3");
    assert_eq!(dependencies[1].group, "depends");
    assert_eq!(dependencies[1].name, "ocaml");
    assert_eq!(dependencies[1].requirement, ">= 4.14");
    assert_eq!(dependencies[1].requirement_prefix, ">= \"");
    assert_eq!(dependencies[1].requirement_suffix, "\"");
    assert_eq!(dependencies[2].name, "dune");
    assert_eq!(dependencies[2].requirement, ">= 3.18");
    assert_eq!(dependencies[3].name, "lwt");
    assert_eq!(dependencies[3].requirement, ">= 5.9");
    assert_eq!(dependencies[4].group, "depopts");
    assert_eq!(dependencies[4].name, "conf-libev");
    assert_eq!(dependencies[4].requirement, "latest");
    assert_eq!(dependencies[5].group, "conflicts");
    assert_eq!(dependencies[5].name, "oldpkg");
    assert_eq!(dependencies[5].requirement, "< 1.0");
}

#[test]
fn parses_cabal_dependencies() {
    let text = package_file_fixture("parses-cabal-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/demo.cabal".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Hackage);
    assert_eq!(dependencies[0].group, "version");
    assert_eq!(dependencies[0].name, "demo");
    assert_eq!(dependencies[0].requirement, "0.1.0.0");
    assert_eq!(dependencies[1].group, "build-depends");
    assert_eq!(dependencies[1].name, "base");
    assert_eq!(dependencies[1].requirement, ">= 4.14");
    assert_eq!(dependencies[1].requirement_prefix, ">= ");
    assert_eq!(dependencies[2].name, "text");
    assert_eq!(dependencies[2].requirement, "^>= 2.0");
    assert_eq!(dependencies[3].name, "tasty");
    assert_eq!(dependencies[3].requirement, ">= 1.4");
}

#[test]
fn parses_cabal_project_constraints() {
    let text = package_file_fixture("parses-cabal-project-constraints.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/cabal.project".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Hackage);
    assert_eq!(dependencies[0].group, "constraints");
    assert_eq!(dependencies[0].name, "lens");
    assert_eq!(dependencies[0].requirement, "== 5.2.3");
    assert_eq!(dependencies[1].name, "text");
    assert_eq!(dependencies[1].requirement, ">= 2.0");
}

#[test]
fn parses_stack_yaml_extra_deps() {
    let text = package_file_fixture("parses-stack-yaml-extra-deps.project");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/stack.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Hackage);
    assert_eq!(dependencies[0].group, "resolver");
    assert_eq!(dependencies[0].name, "stackage-lts");
    assert_eq!(dependencies[0].requirement, "22.43");
    assert_eq!(dependencies[0].hosted_url, Some("stackage".to_owned()));
    assert_eq!(dependencies[1].group, "extra-deps");
    assert_eq!(dependencies[1].name, "acme-missiles");
    assert_eq!(dependencies[1].requirement, "0.3");
    assert_eq!(dependencies[2].name, "text");
    assert_eq!(dependencies[2].requirement, "2.1.1");
    assert_eq!(dependencies[3].name, "./vendor/local-package");
    assert_eq!(dependencies[3].hosted_url, Some("path".to_owned()));
    assert_eq!(
        dependencies[4].name,
        "https://github.com/example/package.git"
    );
    assert_eq!(dependencies[4].hosted_url, Some("git".to_owned()));
}

#[test]
fn parses_conanfile_txt_requirements() {
    let text = package_file_fixture("parses-conanfile-txt-requirements.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/conanfile.txt".to_owned(),
        language_id: "plaintext".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Conan);
    assert_eq!(dependencies[0].group, "requires");
    assert_eq!(dependencies[0].name, "zlib");
    assert_eq!(dependencies[0].requirement, "1.3.1");
    assert_eq!(dependencies[1].name, "poco");
    assert_eq!(dependencies[1].requirement, ">1.0 <1.9");
    assert_eq!(dependencies[1].requirement_prefix, "[");
    assert_eq!(dependencies[1].requirement_suffix, "]");
    assert_eq!(dependencies[2].group, "tool_requires");
    assert_eq!(dependencies[2].name, "cmake");
    assert_eq!(dependencies[3].group, "test_requires");
    assert_eq!(dependencies[3].name, "gtest");
    assert_eq!(dependencies[3].requirement_suffix, "#rev0");
}

#[test]
fn parses_conanfile_py_requirement_attributes() {
    let text = package_file_fixture("parses-conanfile-py-requirement-attributes.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/conanfile.py".to_owned(),
        language_id: "python".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, Conan);
    assert_eq!(dependencies[0].group, "requires");
    assert_eq!(dependencies[0].name, "hello");
    assert_eq!(dependencies[0].requirement, "1.0");
    assert_eq!(dependencies[1].name, "otherlib");
    assert_eq!(dependencies[1].requirement, "2.1");
    assert_eq!(dependencies[1].requirement_suffix, "@otheruser/testing");
    assert_eq!(dependencies[2].group, "tool_requires");
    assert_eq!(dependencies[3].group, "test_requires");
}

#[test]
fn parses_vcpkg_json_dependencies_features_and_overrides() {
    let text = package_file_fixture("parses-vcpkg-json-dependencies-features-and-overrides.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/vcpkg.json".to_owned(),
        language_id: "json".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Vcpkg);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "zlib");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[0].hosted_url, Some("baseline".to_owned()));
    assert_eq!(dependencies[1].name, "fmt");
    assert_eq!(dependencies[1].requirement, "10.1.1#1");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "10.1.1#1"
    );
    assert_eq!(dependencies[2].name, "openssl");
    assert_eq!(dependencies[2].hosted_url, Some("baseline".to_owned()));
    assert_eq!(dependencies[3].group, "features.tools.dependencies");
    assert_eq!(dependencies[3].name, "vcpkg-cmake");
    assert_eq!(dependencies[3].requirement, "2024-04-23");
    assert_eq!(dependencies[4].group, "overrides");
    assert_eq!(dependencies[4].name, "curl");
    assert_eq!(dependencies[4].requirement, "8.7.1#2");
}

#[test]
fn parses_swift_package_dependencies_and_unsupported_sources() {
    let text = package_file_fixture("parses-swift-package-dependencies-and-unsupported-sources.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Package.swift".to_owned(),
        language_id: "swift".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Swift);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "swift-argument-parser");
    assert_eq!(
        dependencies[0].hosted_name,
        Some("apple/swift-argument-parser".to_owned())
    );
    assert_eq!(dependencies[0].requirement, "1.3.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.3.0"
    );
    assert_eq!(dependencies[1].name, "NIO");
    assert_eq!(
        dependencies[1].hosted_name,
        Some("apple/swift-nio".to_owned())
    );
    assert_eq!(dependencies[1].requirement, "2.65.0");
    assert_eq!(dependencies[2].name, "example-package");
    assert_eq!(dependencies[2].requirement, "1.2.3");
    assert_eq!(dependencies[3].name, "package");
    assert_eq!(dependencies[3].requirement, "main");
    assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    assert_eq!(dependencies[4].name, "LocalPackage");
    assert_eq!(dependencies[4].requirement, "../LocalPackage");
    assert_eq!(dependencies[4].hosted_url, Some("path".to_owned()));
}

#[test]
fn parses_zig_package_dependencies_from_zon() {
    let text = package_file_fixture("parses-zig-package-dependencies-from-zon.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build.zig.zon".to_owned(),
        language_id: "zig".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Zig);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "known_folders");
    assert_eq!(
        dependencies[0].hosted_name,
        Some("ziglibs/known-folders".to_owned())
    );
    assert_eq!(dependencies[0].requirement, "0.7.0");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "0.7.0"
    );
    assert_eq!(dependencies[1].name, "local_dep");
    assert_eq!(dependencies[1].requirement, "../local_dep");
    assert_eq!(dependencies[1].hosted_url, Some("path".to_owned()));
}

#[test]
fn parses_nimble_requires_dependencies() {
    let text = package_file_fixture("parses-nimble-requires-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/demo.nimble".to_owned(),
        language_id: "nim".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Nim);
    assert_eq!(dependencies[0].group, "requires");
    assert_eq!(dependencies[0].name, "nim");
    assert_eq!(dependencies[0].requirement, ">= 2.0.0");
    assert_eq!(dependencies[1].name, "jester");
    assert_eq!(dependencies[1].requirement, ">= 0.4.1");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "0.4.1"
    );
    assert_eq!(dependencies[2].name, "pkg");
    assert_eq!(dependencies[2].hosted_name, Some("user/pkg".to_owned()));
    assert_eq!(dependencies[2].requirement, ">= 2.0.0");
    assert_eq!(dependencies[3].name, "foobar");
    assert_eq!(dependencies[3].requirement, "head");
    assert_eq!(dependencies[3].hosted_url, Some("head".to_owned()));
    assert_eq!(dependencies[4].group, "dev.requires");
    assert_eq!(dependencies[4].name, "unittest2");
}

#[test]
fn parses_luarocks_rockspec_dependencies() {
    let text = package_file_fixture("parses-luarocks-rockspec-dependencies.txt");

    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/demo-1.0.0-1.rockspec".to_owned(),
        language_id: "lua".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 4);
    assert_eq!(dependencies[0].ecosystem, LuaRocks);
    assert_eq!(dependencies[0].group, "dependencies");
    assert_eq!(dependencies[0].name, "lua");
    assert_eq!(dependencies[0].requirement, ">= 5.1, < 5.5");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "5.1, < 5.5"
    );
    assert_eq!(dependencies[1].name, "luasocket");
    assert_eq!(dependencies[1].requirement, "== 3.1.0");
    assert_eq!(dependencies[2].group, "build_dependencies");
    assert_eq!(dependencies[2].name, "luafilesystem");
    assert_eq!(dependencies[3].group, "test_dependencies");
    assert_eq!(dependencies[3].name, "busted");
    assert_eq!(dependencies[3].requirement_prefix, "~> ");
}
