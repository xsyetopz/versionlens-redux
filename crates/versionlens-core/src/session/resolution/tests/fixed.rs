use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::id;
use std::time::UNIX_EPOCH;

use super::{DocumentInput, RegistryResponseInput, standard_session};
use versionlens_parsers::Ecosystem::{Cran, Go, Helm, Maven, Npm, Ruby, Terraform};

mod dotnet;
mod npm;
mod registry_sources;

#[test]
fn missing_local_dependencies_return_directory_not_found_without_registry_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project/package.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("missing-local-dependencies-return-directory-not-found-without-registry-lookup.json"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Npm,
            body: r#"{"dist-tags":{"latest":"9.9.9"}}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directoryNotFound");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../local"));
    assert!(output.edits.is_empty());
}

#[test]
fn ruby_path_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("ruby-directory");
    let app = root.join("app");
    let local = app.join("vendor/local");
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("Gemfile")),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("ruby-path-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Ruby,
            body: r#"[{"number":"9.9.9"}]"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("vendor/local")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn stack_custom_resolver_resolves_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/stack.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "stack-custom-resolver-resolves-as-fixed-without-registry-updates.yaml",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest,
        Some("stack resolver".to_owned())
    );
}

#[test]
fn terraform_builtin_provider_resolves_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///main.tf".to_owned(),
            language_id: "terraform".to_owned(),
            text: package_file_fixture(
                "terraform-builtin-provider-resolves-as-fixed-without-registry-updates.tf",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "terraform.io/builtin/terraform".to_owned(),
            ecosystem: Terraform,
            body: r#"{"versions":[{"version":"9.9.9"}]}"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("built-in provider")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn helm_local_and_repository_alias_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///Chart.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("helm-local-and-repository-alias-dependencies-resolve-as-fixed-without-registry-updates.yaml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "local".to_owned(),
            ecosystem: Helm,
            body: "apiVersion: v1\nentries:\n  local:\n    - version: 9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("local chart"));
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("repository alias")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn ansible_git_role_dependencies_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/requirements.yml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture(
                "ansible-git-role-dependencies-resolve-as-fixed-without-registry-updates.yml",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn bazel_non_registry_overrides_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/MODULE.bazel".to_owned(),
            language_id: "starlark".to_owned(),
            text: package_file_fixture("bazel-non-registry-overrides-resolve-as-fixed-without-registry-updatesMODULE.bazel"),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local module")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn nix_local_inputs_resolve_as_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///work/flake.nix".to_owned(),
            language_id: "nix".to_owned(),
            text: package_file_fixture(
                "nix-local-inputs-resolve-as-fixed-without-registry-updates.nix",
            ),
            workspace_root: None,
        },
        &[],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("local flake"));
    assert!(output.edits.is_empty());
}

#[test]
fn renv_non_repository_packages_resolve_as_fixed_sources() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/app/renv.lock".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture(
                "renv-non-repository-packages-resolve-as-fixed-sources.lock",
            ),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "localpkg".to_owned(),
            ecosystem: Cran,
            body: "Package: localpkg\nVersion: 9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn go_replace_local_dependencies_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("go-directory");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&app.join("go.mod")),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-replace-local-dependencies-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "example.test/local".to_owned(),
            ecosystem: Go,
            body: "v9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("../local"));
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn go_work_use_directories_resolve_as_directories() {
    let session = standard_session();
    let root = local_test_root("go-work-use-directory");
    let app = root.join("app");
    let lib = root.join("lib");
    create_dir_all(&app).unwrap();
    create_dir_all(&lib).unwrap();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: file_uri(&root.join("go.work")),
            language_id: "go.mod".to_owned(),
            text: package_file_fixture("go-work-use-directories-resolve-as-directories.txt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "./app".to_owned(),
            ecosystem: Go,
            body: "v9.9.9\n".to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert!(
        output
            .suggestions
            .iter()
            .all(|suggestion| suggestion.status == "directory")
    );
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("./app"));
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("./lib"));
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

#[test]
fn gradle_version_catalog_references_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gradle-version-catalog-references-are-fixed-without-registry-updates.versions.toml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.codehaus.groovy:groovy".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>4.0.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("version catalog alias")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("version catalog reference")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_version_catalog_direct_library_versions_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/gradle/libs.versions.toml".to_owned(),
            language_id: "toml".to_owned(),
            text: package_file_fixture("gradle-version-catalog-direct-library-versions-use-maven-lookup.versions.toml"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.apache.commons:commons-lang3".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>3.17.0</version><version>3.18.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("3.17.0"));
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_scala_cross_dependencies_without_scala_version_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-scala-cross-dependencies-without-scala-version-are-fixed.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.typelevel:cats-core".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.13.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("Scala binary version")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_maven_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-maven-dependencies-use-maven-lookup.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.scala-stm:scala-stm_2.13".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.9.1</version><version>0.9.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("0.9.1"));
    assert!(output.edits.is_empty());
}

#[test]
fn sbt_url_artifact_dependencies_are_fixed_without_registry_updates() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.sbt".to_owned(),
            language_id: "scala".to_owned(),
            text: package_file_fixture("sbt-url-artifact-dependencies-are-fixed-without-registry-updates.sbt"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "jquery:jquery".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>3.2.2</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("package URL"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_build_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("gradle-build-dependencies-use-maven-lookup.gradle.kts"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.springframework:spring-core".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>6.2.8</version><version>6.2.9</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("6.2.8"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_plugin_markers_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/settings.gradle".to_owned(),
            language_id: "groovy".to_owned(),
            text: package_file_fixture("gradle-plugin-markers-use-maven-lookup.gradle"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
                .to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>0.51.0</version><version>0.52.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("0.51.0"));
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_project_and_file_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle".to_owned(),
            language_id: "groovy".to_owned(),
            text: package_file_fixture("gradle-project-and-file-dependencies-are-fixed.gradle"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: ":shared".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("local package")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn gradle_kotlin_named_argument_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/build.gradle.kts".to_owned(),
            language_id: "kotlin".to_owned(),
            text: package_file_fixture("gradle-kotlin-named-argument-dependencies-use-maven-lookup.gradle.kts"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.slf4j:slf4j-api".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>2.0.17</version><version>2.0.18</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("2.0.17"));
    assert!(output.edits.is_empty());
}

#[test]
fn clojure_deps_edn_git_and_local_dependencies_are_fixed() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("clojure-deps-edn-git-and-local-dependencies-are-fixed.edn"),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "io.github.sally:awesome".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
            },
            RegistryResponseInput {
                package: "my.dev:project".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>9.9.9</version></versions></versioning></metadata>"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("git repository")
    );
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(
        output.suggestions[1].latest.as_deref(),
        Some("local package")
    );
    assert!(output.edits.is_empty());
}

#[test]
fn clojure_deps_edn_maven_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("clojure-deps-edn-maven-dependencies-use-maven-lookup.edn"),
            workspace_root: None,
        },
        &[RegistryResponseInput {
            package: "org.clojure:tools.reader".to_owned(),
            ecosystem: Maven,
            body: r#"<metadata><versioning><versions><version>1.1.1</version><version>1.2.0</version></versions></versioning></metadata>"#.to_owned(),
        }],
    );

    assert_eq!(output.suggestions.len(), 1);
    assert_eq!(output.suggestions[0].status, "fixed");
    assert_eq!(output.suggestions[0].latest.as_deref(), Some("1.1.1"));
    assert!(output.edits.is_empty());
}

#[test]
fn leiningen_project_clj_dependencies_use_maven_lookup() {
    let session = standard_session();

    let output = session.resolve_document_with_responses(
        DocumentInput {
            uri: "file:///repo/project.clj".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("leiningen-project-clj-dependencies-use-maven-lookup.clj"),
            workspace_root: None,
        },
        &[
            RegistryResponseInput {
                package: "demo".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>0.1.0-SNAPSHOT</version></versions></versioning></metadata>"#.to_owned(),
            },
            RegistryResponseInput {
                package: "org.clojure:clojure".to_owned(),
                ecosystem: Maven,
                body: r#"<metadata><versioning><versions><version>1.11.3</version><version>1.12.0</version></versions></versioning></metadata>"#.to_owned(),
            },
        ],
    );

    assert_eq!(output.suggestions.len(), 2);
    assert_eq!(output.suggestions[0].status, "current");
    assert_eq!(output.suggestions[1].status, "fixed");
    assert_eq!(output.suggestions[1].latest.as_deref(), Some("1.11.3"));
    assert!(output.edits.is_empty());
}

include!("fixed_jvm.rs");
include!("fixed_more.rs");

fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/fixed")
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
