use self::parsers::parse_manifest_kind;
use crate::classify::classify_document;
use versionlens_model::{Dependency, DocumentInput, ManifestKind};

pub use project_versions::{parse_project_versions, parse_project_versions_as_manifest};

mod parsers;
mod project_versions;
#[cfg(test)]
pub(crate) mod test_support {
    use crate::MavenNamedRepository;
    use crate::{DocumentInput, parse_document};
    use versionlens_model::Range;

    pub(crate) fn parse_fixture(
        text: &str,
        uri: &str,
        language: &str,
    ) -> Vec<versionlens_model::Dependency> {
        parse_document(&DocumentInput::new(
            uri.to_owned(),
            language.to_owned(),
            text.to_owned(),
            None,
        ))
    }

    pub(crate) fn extract_range(text: &str, range: Range) -> &str {
        let line = text.lines().nth(range.start.line as usize).unwrap_or("");
        let start = utf16_character_to_byte(line, range.start.character);
        let end = utf16_character_to_byte(line, range.end.character);
        &line[start..end]
    }

    pub(crate) fn assert_parenthesized_pep508(
        text: &str,
        dependencies: &[versionlens_model::Dependency],
    ) {
        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].name, "unfat");
        assert_eq!(dependencies[0].requirement, ">=0.0.13");
        assert_eq!(dependencies[0].requirement_suffix, ")");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            ">=0.0.13"
        );
        assert_eq!(dependencies[1].name, "httpx");
        assert_eq!(dependencies[1].requirement, ">=0.28,<1");
        assert_eq!(dependencies[1].requirement_suffix, ")");
    }

    pub(crate) fn assert_two_repositories(
        repositories: &[MavenNamedRepository],
        first: (&str, &str),
        second: (&str, &str),
    ) {
        assert_eq!(repositories.len(), 2);
        for (repository, expected) in repositories.iter().zip([first, second]) {
            assert_eq!(repository.id, expected.0);
            assert_eq!(repository.url, expected.1);
        }
    }

    fn utf16_character_to_byte(line: &str, character: u32) -> usize {
        let target = character as usize;
        let mut units = 0;
        for (offset, value) in line.char_indices() {
            if units >= target {
                return offset;
            }
            units += value.len_utf16();
        }
        line.len()
    }
}

/// Parses the dependencies of a document, using the default dependency paths of its manifest kind.
pub fn parse_document(input: &DocumentInput) -> Vec<Dependency> {
    parse_document_with_dependency_paths::<&str>(input, &[])
}

/// Parses the dependencies of a document, selecting them with `dependency_paths` instead of the defaults of its manifest kind.
pub fn parse_document_with_dependency_paths<P: AsRef<str>>(
    input: &DocumentInput,
    dependency_paths: &[P],
) -> Vec<Dependency> {
    parse_document_as_manifest_with_dependency_paths(
        input,
        classify_document(input),
        dependency_paths,
    )
}

fn as_str_ref<P: AsRef<str>>(value: &P) -> &str {
    value.as_ref()
}

/// Parses a document as the given manifest kind, selecting dependencies with `dependency_paths` instead of the defaults of that kind.
pub fn parse_document_as_manifest_with_dependency_paths<P: AsRef<str>>(
    input: &DocumentInput,
    kind: ManifestKind,
    dependency_paths: &[P],
) -> Vec<Dependency> {
    let paths = dependency_paths.iter().map(as_str_ref).collect::<Vec<_>>();

    parse_manifest_kind(kind, &input.text, &paths)
}

#[cfg(test)]
mod tests {
    use super::parse_document;
    use super::parse_project_versions;
    use super::parse_project_versions_as_manifest;
    use super::test_support::{extract_range, parse_fixture};
    use crate::DocumentInput;
    use crate::parse_clojure_maven_repositories;
    use crate::parse_gradle_dependency_maven_repositories;
    use crate::parse_gradle_maven_repositories;
    use crate::parse_gradle_plugin_maven_repositories;
    use crate::parse_leiningen_maven_repositories;
    use crate::parse_sbt_maven_repositories;
    use versionlens_model::Ecosystem::*;
    use versionlens_model::ManifestKind;
    use versionlens_model::VersionableKind;

    #[test]
    fn parses_package_engines_as_runtime_constraints() {
        let dependencies = parse_document(&DocumentInput::new(
            "file:///work/package.json".to_owned(),
            "json".to_owned(),
            r#"{"engines":{"node":">=20","npm":"10"},"packageManager":"pnpm@9.1.0"}"#.to_owned(),
            None,
        ));

        assert_eq!(dependencies.len(), 3);
        let node = dependencies
            .iter()
            .find(|dependency| dependency.name == "node")
            .expect("node engine should be extracted");
        assert_eq!(node.group, "engines");
        assert_eq!(node.versionable_kind(), VersionableKind::RuntimeConstraint);
        assert!(
            dependencies
                .iter()
                .any(|dependency| dependency.group == "packageManager")
        );
    }

    #[test]
    fn surfaces_npm_workspace_and_catalog_references_as_workspace_versionables() {
        let dependencies = parse_document(&DocumentInput::new(
            "file:///work/package.json".to_owned(),
            "json".to_owned(),
            r#"{"dependencies":{"local":"workspace:*","cataloged":"catalog:"}}"#.to_owned(),
            None,
        ));

        assert_eq!(dependencies.len(), 2);
        assert!(dependencies.iter().all(|dependency| {
            dependency.versionable_kind() == VersionableKind::WorkspaceReference
                && dependency.hosted_url.as_deref() == Some("workspace")
        }));
    }

    #[test]
    fn parses_cargo_rust_version_as_a_toolchain_constraint() {
        let dependencies = parse_document(&DocumentInput::new(
            "file:///work/Cargo.toml".to_owned(),
            "toml".to_owned(),
            "[package]\nname = \"demo\"\nversion = \"1.0.0\"\nrust-version = \"1.78\"\n".to_owned(),
            None,
        ));

        let rust_version = dependencies
            .iter()
            .find(|dependency| dependency.group == "rust-version")
            .expect("rust-version should be extracted");
        assert_eq!(rust_version.name, "rust");
        assert_eq!(rust_version.requirement, "1.78");
        assert_eq!(rust_version.hosted_url.as_deref(), Some("toolchain"));
        assert_eq!(
            rust_version.versionable_kind(),
            VersionableKind::RuntimeConstraint
        );
    }

    #[test]
    fn parses_kotlin_dsl_plugin_declarations() {
        let dependencies = parse_document(&DocumentInput::new(
            "file:///work/build.gradle.kts".to_owned(),
            "kotlin".to_owned(),
            "plugins {\n    kotlin(\"jvm\") version \"2.0.0\"\n}".to_owned(),
            None,
        ));

        assert_eq!(dependencies.len(), 1);
        assert_eq!(
            dependencies[0].name,
            "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
        );
        assert_eq!(dependencies[0].requirement, "2.0.0");
    }

    #[test]
    fn parses_unity_project_manifest_dependencies() {
        let text = package_file_fixture("parses-unity-project-manifest-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/Packages/manifest.json", "json");

        assert_eq!(dependencies.len(), 4);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(
                0,
                Unity,
                "dependencies",
                "com.unity.timeline",
                "1.8.7",
            ),
        );
        assert_eq!(dependencies[0].hosted_url, None);
        assert_eq!(dependencies[1].name, "com.example.tools.physics");
        assert_eq!(
            dependencies[1].hosted_url,
            Some("https://registry.example.com".to_owned())
        );
        assert_eq!(dependencies[2].name, "com.acme.local");
        assert_eq!(dependencies[2].hosted_url, Some("file".to_owned()));
        assert_eq!(dependencies[3].name, "com.acme.git");
        assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
    }

    #[test]
    fn parses_cocoapods_podfile_dependencies() {
        let text = package_file_fixture("parses-cocoapods-podfile-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/Podfile", "ruby");

        assert_eq!(dependencies.len(), 6);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(
                0,
                CocoaPods,
                "target App",
                "AFNetworking",
                "~> 4.0",
            ),
        );
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "~> 4.0"
        );
        assert_eq!(dependencies[0].requirement_prefix, "~> ");
        assert_eq!(
            dependencies[0].hosted_url.as_deref(),
            Some("https://cdn.cocoapods.org")
        );
        assert_eq!(dependencies[1].name, "SSZipArchive");
        assert_eq!(dependencies[1].requirement, "latest");
        assert_eq!(dependencies[1].hosted_url.as_deref(), Some("latest"));
        assert_eq!(dependencies[2].name, "LocalPod");
        assert_eq!(dependencies[2].requirement, "../LocalPod");
        assert_eq!(dependencies[2].hosted_url.as_deref(), Some("path"));
        assert_eq!(dependencies[3].name, "GitPod");
        assert_eq!(dependencies[3].requirement, "1.2.3");
        assert_eq!(dependencies[3].hosted_url.as_deref(), Some("git"));
        assert_eq!(dependencies[4].name, "JSONKit");
        assert_eq!(
            dependencies[4].requirement,
            "https://example.com/JSONKit.podspec"
        );
        assert_eq!(dependencies[4].hosted_url.as_deref(), Some("podspec"));
        assert_eq!(dependencies[5].name, "PonyDebugger");
        assert_eq!(dependencies[5].requirement, "latest");
        assert_eq!(
            dependencies[5].hosted_url.as_deref(),
            Some("https://private.example.com/specs")
        );
    }

    #[test]
    fn parses_kustomization_yaml_images() {
        let text = package_file_fixture("parses-kustomization-yaml-images.txt");

        let dependencies = parse_fixture(text, "file:///work/kustomization.yaml", "yaml");

        assert_eq!(dependencies.len(), 2);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(
                0,
                Docker,
                "images",
                "platform/nginx",
                "1.25.3",
            ),
        );
        assert_eq!(
            dependencies[0].hosted_url,
            Some("registry.example.com".to_owned())
        );
        assert_eq!(dependencies[0].hosted_name, Some("nginx".to_owned()));
        assert_eq!(dependencies[1].name, "redis");
        assert_eq!(dependencies[1].requirement, "sha256:abcdef");
        assert_eq!(dependencies[1].requirement_prefix, "digest: ");
    }
    #[test]
    fn parses_rebar_config_dependencies() {
        let text = package_file_fixture("parses-rebar-config-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/rebar.config", "erlang");

        assert_eq!(dependencies.len(), 6);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(0, Hex, "deps", "rebar", "latest"),
        );
        assert_eq!(dependencies[1].name, "cowboy");
        assert_eq!(dependencies[1].requirement, "2.12.0");
        assert_eq!(dependencies[2].name, "lager_fork");
        assert_eq!(dependencies[2].requirement, "latest");
        assert_eq!(dependencies[2].hosted_name, Some("lager".to_owned()));
        assert_eq!(dependencies[3].name, "jsx_fork");
        assert_eq!(dependencies[3].requirement, "3.1.0");
        assert_eq!(dependencies[3].hosted_name, Some("jsx".to_owned()));
        assert_eq!(dependencies[4].name, "gettext");
        assert_eq!(
            dependencies[4].requirement,
            "https://github.com/elixir-lang/gettext.git"
        );
        assert_eq!(dependencies[4].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[5].name, "local_dep");
        assert_eq!(
            dependencies[5].requirement,
            "https://github.com/example/local.git"
        );
        assert_eq!(dependencies[5].hosted_url, Some("git".to_owned()));
    }

    #[test]
    fn parses_gleam_toml_dependencies() {
        let text = package_file_fixture("parses-gleam-toml-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/gleam.toml", "toml");

        assert_eq!(dependencies.len(), 5);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(0, Hex, "version", "demo", "1.0.0"),
        );
        assert_eq!(dependencies[1].group, "dependencies");
        assert_eq!(dependencies[1].name, "gleam_stdlib");
        assert_eq!(dependencies[1].requirement, ">= 0.44.0 and < 2.0.0");
        assert_eq!(dependencies[2].name, "my_other_project");
        assert_eq!(dependencies[2].requirement, "../my_other_project");
        assert_eq!(dependencies[2].hosted_url, Some("path".to_owned()));
        assert_eq!(dependencies[3].name, "my_library");
        assert_eq!(
            dependencies[3].requirement,
            "https://github.com/my-project/my-library"
        );
        assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[4].group, "dev_dependencies");
        assert_eq!(dependencies[4].name, "gleeunit");
    }

    #[test]
    fn parses_gradle_version_catalog_dependencies() {
        let text = package_file_fixture("parses-gradle-version-catalog-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/gradle/libs.versions.toml", "toml");

        assert_eq!(dependencies.len(), 7);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(
                0, Maven, "versions", "groovy", "3.0.5",
            ),
        );
        assert_eq!(dependencies[1].name, "kotlin");
        assert_eq!(dependencies[1].requirement, "2.0.20");
        assert_eq!(dependencies[1].requirement_prefix, "prefer = \"");
        assert_eq!(dependencies[2].group, "libraries");
        assert_eq!(dependencies[2].name, "org.codehaus.groovy:groovy");
        assert_eq!(dependencies[2].requirement, "groovy");
        assert_eq!(dependencies[2].hosted_url, Some("version.ref".to_owned()));
        assert_eq!(dependencies[3].name, "org.apache.commons:commons-lang3");
        assert_eq!(dependencies[3].requirement, "3.17.0");
        assert_eq!(dependencies[4].group, "libraries");
        assert_eq!(dependencies[4].name, "junit:junit");
        assert_eq!(dependencies[4].requirement, "4.13.2");
        assert_eq!(
            extract_range(text, dependencies[4].requirement_range),
            "4.13.2"
        );
        assert_eq!(dependencies[5].group, "plugins");
        assert_eq!(
            dependencies[5].name,
            "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
        );
        assert_eq!(dependencies[5].requirement, "0.45.0");
        assert_eq!(dependencies[6].group, "plugins");
        assert_eq!(
            dependencies[6].name,
            "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
        );
        assert_eq!(dependencies[6].requirement, "kotlin");
        assert_eq!(dependencies[6].hosted_url, Some("version.ref".to_owned()));
    }

    #[test]
    fn parses_gradle_catalog_inline_version_with_exact_requirement_range() {
        let text = "[libraries]\nfoo = { module = \"example.test:foo\", version = \"1.0.0\" }\n";

        let dependencies = parse_fixture(text, "file:///work/gradle/libs.versions.toml", "toml");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "example.test:foo");
        assert_eq!(dependencies[0].requirement, "1.0.0");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "1.0.0"
        );
    }

    #[test]
    fn gradle_catalog_version_resolves_through_first_referencing_library_in_file_order() {
        let text = "[versions]\nkotlin = \"2.0.0\"\n\n[libraries]\nzeta = { module = \"example.test:first\", version.ref = \"kotlin\" }\nalpha = { module = \"example.test:second\", version.ref = \"kotlin\" }\n";

        let dependencies = parse_fixture(text, "file:///work/gradle/libs.versions.toml", "toml");

        assert_eq!(dependencies.len(), 3);
        assert_eq!(dependencies[0].group, "versions");
        assert_eq!(dependencies[0].name, "kotlin");
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("example.test:first")
        );
        assert_eq!(dependencies[0].hosted_url, None);
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "2.0.0"
        );
    }

    #[test]
    fn gradle_catalog_version_referenced_by_plugin_uses_the_plugin_marker() {
        let text = "[versions]\nksp = \"1.0.0\"\n\n[plugins]\nksp = { id = \"com.google.devtools.ksp\", version.ref = \"ksp\" }\n";

        let dependencies = parse_fixture(text, "file:///work/gradle/libs.versions.toml", "toml");

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].group, "versions");
        assert_eq!(dependencies[0].name, "ksp");
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("com.google.devtools.ksp:com.google.devtools.ksp.gradle.plugin")
        );
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "1.0.0"
        );
    }

    #[test]
    fn gradle_catalog_unreferenced_version_is_unresolved_without_error() {
        let text = "[versions]\norphan = \"9.9.9\"\n";

        let dependencies = parse_fixture(text, "file:///work/gradle/libs.versions.toml", "toml");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].group, "versions");
        assert_eq!(dependencies[0].name, "orphan");
        assert_eq!(dependencies[0].hosted_name, None);
        assert_eq!(dependencies[0].hosted_url.as_deref(), Some("version.alias"));
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "9.9.9"
        );
    }

    #[test]
    fn parses_sbt_build_dependencies() {
        let text = package_file_fixture("parses-sbt-build-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 4);
        crate::support::tests::assert_dependency(
            &dependencies,
            crate::support::tests::DependencyExpectation::new(
                0,
                Maven,
                "libraryDependencies",
                "org.apache.derby:derby",
                "10.4.1.3",
            ),
        );
        assert_eq!(dependencies[1].name, "org.scala-stm:scala-stm_2.13");
        assert_eq!(dependencies[1].requirement, "0.9.1");
        assert_eq!(dependencies[1].hosted_name, Some("scala-stm".to_owned()));
        assert_eq!(dependencies[2].name, "org.typelevel:cats-core_2.13");
        assert_eq!(dependencies[2].requirement, "2.12.0");
        assert_eq!(dependencies[3].name, "com.lihaoyi:os-lib_2.13");
        assert_eq!(dependencies[3].requirement, "0.11.3");
    }

    #[test]
    fn parses_sbt_multiple_dependencies_on_one_line() {
        let text = package_file_fixture("parses-sbt-multiple-dependencies-on-one-line.txt");

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].name, "org.zeta:zeta");
        assert_eq!(dependencies[0].requirement, "1.0.0");
        assert_eq!(dependencies[1].name, "org.alpha:alpha");
        assert_eq!(dependencies[1].requirement, "2.0.0");
        assert_eq!(
            dependencies[1].hosted_url,
            Some("scala-binary-version".to_owned())
        );
    }

    #[test]
    fn parses_zig_github_repository_urls_only_when_the_identity_is_valid() {
        let valid = parse_document(&DocumentInput::new(
        "file:///work/build.zig.zon".to_owned(),
        "zig".to_owned(),
        r#".dependencies = .{
            .known_folders = .{ .url = "https://github.com/ziglibs/known-folders/archive/refs/tags/0.7.0.tar.gz" },
        };"#
        .to_owned(),
        None,
    ));
        assert_eq!(
            valid[0].hosted_name.as_deref(),
            Some("ziglibs/known-folders")
        );
        assert_eq!(valid[0].requirement, "0.7.0");

        for url in [
            "https://github.com/../repo/archive/refs/tags/1.0.0.tar.gz",
            "https://github.com/owner/../repo/archive/refs/tags/1.0.0.tar.gz",
            "https://github.com/owner/re%2Fpo/archive/refs/tags/1.0.0.tar.gz",
            "https://github.com/owner/repo?query=1/archive/refs/tags/1.0.0.tar.gz",
            "https://github.com/owner/repo#fragment/archive/refs/tags/1.0.0.tar.gz",
            "https://github.com/owner/repo name/archive/refs/tags/1.0.0.tar.gz",
        ] {
            let dependencies = parse_document(&DocumentInput::new(
                "file:///work/build.zig.zon".to_owned(),
                "zig".to_owned(),
                format!(".dependencies = .{{ .dependency = .{{ .url = \"{url}\" }} }};"),
                None,
            ));
            assert_eq!(dependencies.len(), 1, "unexpected parse result for {url}");
            assert_eq!(
                dependencies[0].hosted_name, None,
                "unsafe URL accepted: {url}"
            );
            assert_eq!(dependencies[0].hosted_url.as_deref(), Some("url"));
        }
    }

    #[test]
    fn parses_sbt_dependency_overrides() {
        let text = package_file_fixture("parses-sbt-dependency-overridesbuild.sbt");

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].group, "dependencyOverrides");
        assert_eq!(dependencies[0].name, "log4j:log4j");
        assert_eq!(dependencies[0].requirement, "1.2.16");
    }

    #[test]
    fn parses_sbt_double_percent_without_scala_version_as_fixed_source() {
        let text = package_file_fixture(
            "parses-sbt-double-percent-without-scala-version-as-fixed-sourcebuild.sbt",
        );

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "org.typelevel:cats-core");
        assert_eq!(dependencies[0].requirement, "2.12.0");
        assert_eq!(
            dependencies[0].hosted_url,
            Some("scala-binary-version".to_owned())
        );
    }

    #[test]
    fn parses_sbt_scoped_scala_version_for_cross_dependencies() {
        let text =
            package_file_fixture("parses-sbt-scoped-scala-version-for-cross-dependenciesbuild.sbt");

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "org.typelevel:cats-core_2.13");
        assert_eq!(dependencies[0].requirement, "2.12.0");
        assert_eq!(dependencies[0].hosted_name, Some("cats-core".to_owned()));
    }

    #[test]
    fn parses_sbt_url_artifact_dependency_as_non_registry_source() {
        let text = package_file_fixture(
            "parses-sbt-url-artifact-dependency-as-non-registry-sourcebuild.sbt",
        );

        let dependencies = parse_fixture(text, "file:///work/build.sbt", "scala");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "jquery:jquery");
        assert_eq!(dependencies[0].requirement, "3.2.1");
        assert_eq!(dependencies[0].hosted_url, Some("url".to_owned()));
    }

    #[test]
    fn parses_sbt_maven_resolvers() {
        let repositories = parse_sbt_maven_repositories(
            r#"resolvers += "Private Maven" at "https://maven.example.test/releases"
resolvers ++= Seq(
  "Snapshots" at "https://maven.example.test/snapshots",
  Resolver.mavenLocal
)
"#,
        );

        assert_eq!(repositories.len(), 2);
        assert_eq!(repositories[0].id, "Private Maven");
        assert_eq!(repositories[0].url, "https://maven.example.test/releases");
        assert_eq!(repositories[1].id, "Snapshots");
        assert_eq!(repositories[1].url, "https://maven.example.test/snapshots");
    }

    #[test]
    fn parses_gradle_build_dependencies_and_plugins() {
        let text = package_file_fixture("parses-gradle-build-dependencies-and-plugins.txt");

        let dependencies = parse_fixture(text, "file:///work/build.gradle", "groovy");

        assert_eq!(dependencies.len(), 7);
        assert_eq!(dependencies[0].ecosystem, Maven);
        assert_eq!(dependencies[0].group, "plugins");
        assert_eq!(
            dependencies[0].name,
            "com.github.ben-manes.versions:com.github.ben-manes.versions.gradle.plugin"
        );
        assert_eq!(dependencies[0].requirement, "0.51.0");
        assert_eq!(dependencies[1].group, "plugins");
        assert_eq!(
            dependencies[1].name,
            "org.jetbrains.kotlin.jvm:org.jetbrains.kotlin.jvm.gradle.plugin"
        );
        assert_eq!(dependencies[1].requirement, "2.1.20");
        assert_eq!(dependencies[2].group, "implementation");
        assert_eq!(dependencies[2].name, "org.springframework:spring-core");
        assert_eq!(dependencies[2].requirement, "6.2.8");
        assert_eq!(dependencies[3].group, "testImplementation");
        assert_eq!(dependencies[3].name, "junit:junit");
        assert_eq!(dependencies[3].requirement, "4.13.2");
        assert_eq!(dependencies[4].group, "runtimeOnly");
        assert_eq!(dependencies[4].name, "org.postgresql:postgresql");
        assert_eq!(dependencies[4].requirement, "42.7.3");
        assert_eq!(dependencies[5].group, "compileOnly");
        assert_eq!(dependencies[5].name, ":local");
        assert_eq!(dependencies[5].requirement, ":local");
        assert_eq!(dependencies[5].hosted_url, Some("local".to_owned()));
        assert_eq!(dependencies[6].group, "annotationProcessor");
        assert_eq!(dependencies[6].name, "libs/processor.jar");
        assert_eq!(dependencies[6].requirement, "libs/processor.jar");
        assert_eq!(dependencies[6].hosted_url, Some("local".to_owned()));
    }

    #[test]
    fn parses_gradle_settings_plugins() {
        let text = package_file_fixture("parses-gradle-settings-plugins.txt");

        let dependencies = parse_fixture(text, "file:///work/settings.gradle.kts", "kotlin");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].group, "plugins");
        assert_eq!(
            dependencies[0].name,
            "com.android.application:com.android.application.gradle.plugin"
        );
        assert_eq!(dependencies[0].requirement, "8.12.0");
    }

    #[test]
    fn parses_gradle_kotlin_named_argument_dependencies() {
        let text = package_file_fixture(
            "parses-gradle-kotlin-named-argument-dependenciessettings.gradle.kts",
        );

        let dependencies = parse_fixture(text, "file:///work/build.gradle.kts", "kotlin");

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].group, "implementation");
        assert_eq!(dependencies[0].name, "org.slf4j:slf4j-api");
        assert_eq!(dependencies[0].requirement, "2.0.17");
        assert_eq!(dependencies[1].group, "testImplementation");
        assert_eq!(dependencies[1].name, "org.assertj:assertj-core");
        assert_eq!(dependencies[1].requirement, "3.27.3");
    }

    #[test]
    fn parses_gradle_maven_repositories() {
        let repositories = parse_gradle_maven_repositories(
            r#"repositories {
    maven {
        url = uri("https://maven.example.test/releases")
    }
    google()
    gradlePluginPortal()
    maven {
        url 'https://maven.example.test/snapshots'
    }
    mavenCentral()
    mavenLocal()
}
"#,
        );

        assert_eq!(repositories.len(), 5);
        assert_eq!(repositories[0].id, "maven");
        assert_eq!(repositories[0].url, "https://maven.example.test/releases");
        assert_eq!(repositories[1].id, "google");
        assert_eq!(
            repositories[1].url,
            "https://dl.google.com/dl/android/maven2/"
        );
        assert_eq!(repositories[2].id, "gradlePluginPortal");
        assert_eq!(repositories[2].url, "https://plugins.gradle.org/m2/");
        assert_eq!(repositories[3].id, "maven");
        assert_eq!(repositories[3].url, "https://maven.example.test/snapshots");
        assert_eq!(repositories[4].id, "mavenCentral");
        assert_eq!(repositories[4].url, "https://repo.maven.apache.org/maven2");
    }

    #[test]
    fn parses_gradle_plugin_maven_repositories_separately() {
        let text = package_file_fixture("parses-gradle-plugin-maven-repositories-separately.txt");

        let plugin_repositories = parse_gradle_plugin_maven_repositories(text);
        let dependency_repositories = parse_gradle_dependency_maven_repositories(text);

        assert_eq!(plugin_repositories.len(), 2);
        assert_eq!(plugin_repositories[0].id, "maven");
        assert_eq!(
            plugin_repositories[0].url,
            "https://plugins.example.test/releases"
        );
        assert_eq!(plugin_repositories[1].id, "gradlePluginPortal");
        assert_eq!(plugin_repositories[1].url, "https://plugins.gradle.org/m2/");
        assert_eq!(dependency_repositories.len(), 1);
        assert_eq!(dependency_repositories[0].id, "mavenCentral");
        assert_eq!(
            dependency_repositories[0].url,
            "https://repo.maven.apache.org/maven2"
        );
    }

    fn adapter_input(uri: &str, text: &str) -> DocumentInput {
        DocumentInput::new(
            uri.to_owned(),
            "plaintext".to_owned(),
            text.to_owned(),
            None,
        )
    }

    fn with_paths(uri: &str, text: &str, paths: &[&str]) -> Vec<versionlens_model::Dependency> {
        crate::parse_document_with_dependency_paths(&adapter_input(uri, text), paths)
    }

    const NPM: &str = "{\"name\":\"demo\",\"version\":\"0.4.4\",\"dependencies\":{\"a\":\"^1\"}}";
    const CARGO: &str = "[package]\nname = \"demo\"\nversion = \"1.2.3\"\nrust-version = \"1.70\"\n\n[workspace.package]\nversion = \"2.0.0\"\n\n[dependencies]\nserde = \"1\"\n";

    #[test]
    fn npm_version_as_a_user_path_is_not_a_dependency() {
        let uri = "file:///work/package.json";

        assert_eq!(with_paths(uri, NPM, &["version"]), vec![]);
        assert!(
            with_paths(uri, NPM, &["version", "dependencies"])
                .iter()
                .all(|dependency| dependency.group != "version")
        );
        assert_eq!(parse_project_versions(&adapter_input(uri, NPM)).len(), 1);
    }

    #[test]
    fn npm_without_user_paths_uses_the_defaults() {
        let dependencies = parse_document(&adapter_input("file:///work/package.json", NPM));

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "a");
    }

    #[test]
    fn cargo_version_fields_are_not_dependencies_under_any_user_path() {
        let uri = "file:///work/Cargo.toml";

        for paths in [
            &["package.version"][..],
            &["package.*"],
            &["*.*"],
            &["workspace.package"],
            &["workspace.*"],
            &["workspace.package.version"],
        ] {
            let dependencies = with_paths(uri, CARGO, paths);
            assert!(
                dependencies
                    .iter()
                    .all(|dependency| dependency.name != "version"),
                "{paths:?} emitted {dependencies:?}"
            );
        }
        let versions = parse_project_versions(&adapter_input(uri, CARGO));
        assert_eq!(
            versions
                .iter()
                .map(|dependency| dependency.requirement.as_str())
                .collect::<Vec<_>>(),
            ["1.2.3", "2.0.0"]
        );
    }

    #[test]
    fn gleam_ranges_are_exact_with_crlf_line_breaks() {
        let text = "name = \"app\"\r\nversion = \"1.2.3\"\r\n\r\n[dependencies]\r\ngleam_stdlib = \">= 0.34.0 and < 2.0.0\" # c\r\nlocal = { path = \"../local\" }\r\n";
        let dependencies = with_paths("file:///work/gleam.toml", text, &[]);

        assert_eq!(dependencies.len(), 3);
        let expected = [
            ("version = \"1.2.3\"", "1.2.3"),
            (
                "gleam_stdlib = \">= 0.34.0 and < 2.0.0\" # c",
                ">= 0.34.0 and < 2.0.0",
            ),
            ("local = { path = \"../local\" }", "../local"),
        ];
        for (dependency, (line, requirement)) in dependencies.iter().zip(expected) {
            assert_eq!(extract_range(text, dependency.range), line);
            assert_eq!(
                extract_range(text, dependency.requirement_range),
                requirement
            );
        }
    }

    #[test]
    fn julia_ranges_are_exact_with_crlf_line_breaks() {
        let project = "name = \"App\"\r\nversion = \"0.1.0\"\r\n\r\n[deps]\r\nJSON = \"682c06a0\"\r\n\r\n[compat]\r\nJSON = \"0.21\"\r\n";
        let dependencies = with_paths("file:///work/Project.toml", project, &[]);

        assert_eq!(dependencies.len(), 2);
        assert_eq!(
            extract_range(project, dependencies[0].range),
            "version = \"0.1.0\""
        );
        assert_eq!(
            extract_range(project, dependencies[1].range),
            "JSON = \"0.21\""
        );
        assert_eq!(
            extract_range(project, dependencies[1].requirement_range),
            "0.21"
        );

        let manifest = "[[deps.JSON]]\r\nuuid = \"x\"\r\nversion = \"0.21.4\"\r\n\r\n[[deps.Local]]\r\npath = \"/home/Local\"\r\n";
        let dependencies = with_paths("file:///work/Manifest.toml", manifest, &[]);

        assert_eq!(dependencies.len(), 2);
        let block = |dependency: &versionlens_model::Dependency| {
            (
                dependency.range.start.line,
                dependency.range.start.character,
                dependency.range.end.line,
                dependency.range.end.character,
            )
        };
        assert_eq!(block(&dependencies[0]), (0, 0, 4, 0));
        assert_eq!(
            extract_range(manifest, dependencies[0].requirement_range),
            "0.21.4"
        );
        assert_eq!(block(&dependencies[1]), (4, 0, 6, 0));
        assert_eq!(
            extract_range(manifest, dependencies[1].requirement_range),
            "/home/Local"
        );
    }

    #[test]
    fn stack_yaml_reads_git_github_quoted_and_commented_entries() {
        let text = "resolver: lts-21.25\nextra-deps:\n  - acme-1.2.3 # pinned\n  - \"quoted-2.0\"\n  - 'single-3.0'\n  - github: owner/repo\n    commit: abc\n  - git: https://example.test/x.git\n    commit: def\n  - ./local\n";
        let dependencies = with_paths("file:///work/stack.yaml", text, &[]);

        let summary = dependencies
            .iter()
            .map(|dependency| {
                (
                    dependency.name.as_str(),
                    dependency.requirement.as_str(),
                    dependency.hosted_url.as_deref(),
                    extract_range(text, dependency.requirement_range),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            summary,
            [
                ("stackage-lts", "21.25", Some("stackage"), "21.25"),
                ("acme", "1.2.3", None, "1.2.3"),
                ("quoted", "2.0", None, "2.0"),
                ("single", "3.0", None, "3.0"),
                ("owner/repo", "owner/repo", Some("git"), "owner/repo"),
                (
                    "https://example.test/x.git",
                    "https://example.test/x.git",
                    Some("git"),
                    "https://example.test/x.git"
                ),
                ("./local", "./local", Some("path"), "./local"),
            ]
        );
    }
    use versionlens_model::Ecosystem::Cpp;

    #[test]
    fn parses_cpp_cmake_dependencies() {
        let text = package_file_fixture("parses-cpp-cmake-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/CMakeLists.txt", "cmake");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Cpp);
        assert_eq!(dependencies[0].group, "FetchContent_Declare");
        assert_eq!(dependencies[0].name, "fmt");
        assert_eq!(dependencies[0].requirement, "10.2.1");
        assert_eq!(dependencies[0].hosted_name.as_deref(), Some("fmtlib/fmt"));
        assert_eq!(dependencies[1].group, "ExternalProject_Add");
        assert_eq!(dependencies[1].name, "zlib_project");
        assert_eq!(dependencies[1].requirement, "v1.3.1");
        assert_eq!(dependencies[2].group, "CPMAddPackage");
        assert_eq!(dependencies[2].name, "Catch2");
        assert_eq!(extract_range(text, dependencies[2].range), "Catch2");
        assert_eq!(dependencies[2].requirement, "v3.6.0");
        assert_eq!(dependencies[3].group, "CPMAddPackage");
        assert_eq!(dependencies[3].name, "spdlog");
        assert_eq!(extract_range(text, dependencies[3].range), "spdlog");
        assert_eq!(dependencies[3].requirement, "1.14.1");
        assert_eq!(
            dependencies[3].hosted_name.as_deref(),
            Some("gabime/spdlog")
        );
    }

    #[test]
    fn parses_cpp_xmake_dependencies() {
        let text = package_file_fixture("parses-cpp-xmake-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/xmake.lua", "lua");

        assert_eq!(dependencies.len(), 3);
        assert_eq!(dependencies[0].ecosystem, Cpp);
        assert_eq!(dependencies[0].group, "add_requires");
        assert_eq!(dependencies[0].name, "zlib");
        assert_eq!(dependencies[0].requirement, "1.3.1");
        assert_eq!(dependencies[1].name, "fmt");
        assert_eq!(dependencies[1].requirement, "10.2.1");
        assert_eq!(dependencies[2].name, "openssl");
        assert_eq!(dependencies[2].requirement, "*");
        assert_eq!(dependencies[2].requirement_prefix, " ");
        assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
    }

    #[test]
    fn parses_cpp_meson_wrap_dependency() {
        let text = package_file_fixture("parses-cpp-meson-wrap-dependency.txt");

        let dependencies = parse_fixture(text, "file:///work/subprojects/gtest.wrap", "meson");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].ecosystem, Cpp);
        assert_eq!(dependencies[0].group, "meson.wrap");
        assert_eq!(dependencies[0].name, "googletest-1.14.0");
        assert_eq!(dependencies[0].requirement, "v1.14.0");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "v1.14.0"
        );
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("google/googletest")
        );
    }

    #[test]
    fn parses_cpp_bazel_workspace_dependency() {
        let text = package_file_fixture("parses-cpp-bazel-workspace-dependency.txt");

        let dependencies = parse_fixture(text, "file:///work/WORKSPACE", "starlark");

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].ecosystem, Cpp);
        assert_eq!(dependencies[0].group, "http_archive");
        assert_eq!(dependencies[0].name, "rules_cc");
        assert_eq!(dependencies[0].requirement, "0.0.9");
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("bazelbuild/rules_cc")
        );
    }
    #[test]
    fn parses_versioned_github_actions() {
        let dependencies = parse_document(&DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("parses-github-actions-version-refs.yaml").to_owned(),
            None,
        ));

        assert_eq!(dependencies.len(), 3);
        assert!(
            dependencies
                .iter()
                .all(|dependency| dependency.ecosystem == GitHub)
        );
        assert_eq!(dependencies[0].name, "actions/checkout");
        assert_eq!(dependencies[0].requirement, "4");
        assert_eq!(
            dependencies[0].hosted_name.as_deref(),
            Some("actions/checkout")
        );
        assert_eq!(dependencies[1].requirement, "4.1.0");
        assert_eq!(
            dependencies[1].hosted_name.as_deref(),
            Some("actions/setup-node")
        );
        assert_eq!(
            dependencies[2].name,
            "acme/automation/.github/workflows/release.yml"
        );
        assert_eq!(dependencies[2].requirement, "1.2.0");
        assert_eq!(
            dependencies[2].hosted_name.as_deref(),
            Some("acme/automation")
        );
    }

    #[test]
    fn parses_each_structural_github_actions_reference_kind() {
        let input = DocumentInput::new(
            "file:///work/.github/workflows/ci.yml".to_owned(),
            "yaml".to_owned(),
            package_file_fixture("parses-github-actions-reference-kinds.yaml").to_owned(),
            Some("/work".to_owned()),
        );
        let dependencies = parse_document(&input);

        assert_eq!(dependencies.len(), 4);
        assert!(matches!(
            dependencies[0].canonical_reference,
            Some(versionlens_model::CanonicalReference::GitHubActionLocal {
                reusable_workflow: true,
                ..
            })
        ));
        assert!(matches!(
            dependencies[1].canonical_reference,
            Some(versionlens_model::CanonicalReference::GitHubActionRef { .. })
        ));
        assert!(matches!(
            dependencies[2].canonical_reference,
            Some(versionlens_model::CanonicalReference::GitHubActionLocal {
                reusable_workflow: false,
                ..
            })
        ));
        assert!(matches!(
            dependencies[3].canonical_reference,
            Some(versionlens_model::CanonicalReference::GitHubActionExpression { .. })
        ));
    }
    #[test]
    fn parses_clojure_deps_edn_dependencies() {
        let text = package_file_fixture("parses-clojure-deps-edn-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/deps.edn", "clojure");

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

        super::test_support::assert_two_repositories(
            &repositories,
            ("private", "https://maven.example.test/releases"),
            ("snapshots", "https://maven.example.test/snapshots"),
        );
    }

    #[test]
    fn parses_leiningen_project_clj_dependencies() {
        let text = package_file_fixture("parses-leiningen-project-clj-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/project.clj", "clojure");

        assert_eq!(dependencies.len(), 7);
        assert_eq!(dependencies[0].ecosystem, Maven);
        assert_eq!(dependencies[0].group, "version");
        assert_eq!(dependencies[0].name, "demo");
        assert_eq!(dependencies[0].requirement, "0.1.0-SNAPSHOT");
        assert_eq!(
            dependencies[0].versionable_kind(),
            VersionableKind::ProjectVersion
        );
        assert_eq!(dependencies[1].group, "dependencies");
        assert_eq!(dependencies[1].name, "org.clojure:clojure");
        assert_eq!(dependencies[1].requirement, "1.11.3");
        assert_eq!(
            dependencies[1].versionable_kind(),
            VersionableKind::Dependency
        );
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

        super::test_support::assert_two_repositories(
            &repositories,
            ("private", "https://maven.example.test/releases"),
            ("snapshots", "https://maven.example.test/snapshots"),
        );
    }

    #[test]
    fn parses_dune_project_package_dependencies() {
        let text = package_file_fixture("parses-dune-project-package-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/dune-project", "plaintext");

        assert_common_opam_dependencies(&dependencies);
        assert_eq!(dependencies[2].name, "fmt");
        assert_eq!(dependencies[2].requirement, ">= 0.6");
        assert_eq!(dependencies[3].name, "lwt");
        assert_eq!(dependencies[3].requirement, "latest");
    }

    #[test]
    fn parses_opam_dependencies() {
        let text = package_file_fixture("parses-opam-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/demo.opam", "plaintext");

        assert_eq!(dependencies.len(), 6);
        assert_common_opam_dependencies(&dependencies);
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

    fn assert_common_opam_dependencies(dependencies: &[versionlens_model::Dependency]) {
        assert_eq!(dependencies[0].ecosystem, Opam);
        assert_eq!(dependencies[0].group, "version");
        assert_eq!(dependencies[0].name, "demo");
        assert_eq!(dependencies[0].requirement, "1.2.3");
        assert_eq!(dependencies[1].group, "depends");
        assert_eq!(dependencies[1].name, "ocaml");
        assert_eq!(dependencies[1].requirement, ">= 4.14");
    }

    #[test]
    fn parses_cabal_dependencies() {
        let text = package_file_fixture("parses-cabal-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/demo.cabal", "plaintext");

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

        let dependencies = parse_fixture(text, "file:///work/cabal.project", "plaintext");

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

        let dependencies = parse_fixture(text, "file:///work/stack.yaml", "yaml");

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

        let dependencies = parse_fixture(text, "file:///work/conanfile.txt", "plaintext");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Conan);
        crate::support::tests::assert_dependency_metadata(
            &dependencies,
            0,
            "requires",
            "zlib",
            "1.3.1",
        );
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

        let dependencies = parse_fixture(text, "file:///work/conanfile.py", "python");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Conan);
        crate::support::tests::assert_dependency_metadata(
            &dependencies,
            0,
            "requires",
            "hello",
            "1.0",
        );
        assert_eq!(dependencies[1].name, "otherlib");
        assert_eq!(dependencies[1].requirement, "2.1");
        assert_eq!(dependencies[1].requirement_suffix, "@otheruser/testing");
        assert_eq!(dependencies[2].group, "tool_requires");
        assert_eq!(dependencies[3].group, "test_requires");
    }

    #[test]
    fn parses_vcpkg_json_dependencies_features_and_overrides() {
        let text =
            package_file_fixture("parses-vcpkg-json-dependencies-features-and-overrides.txt");

        let dependencies = parse_fixture(text, "file:///work/vcpkg.json", "json");

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
        let text =
            package_file_fixture("parses-swift-package-dependencies-and-unsupported-sources.txt");

        let dependencies = parse_fixture(text, "file:///work/Package.swift", "swift");

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

        let dependencies = parse_fixture(text, "file:///work/build.zig.zon", "zig");

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

        let dependencies = parse_fixture(text, "file:///work/demo.nimble", "nim");

        assert_eq!(dependencies.len(), 5);
        assert_eq!(dependencies[0].ecosystem, Nim);
        assert_eq!(dependencies[0].group, "requires");
        assert_eq!(dependencies[0].name, "nim");
        assert_eq!(dependencies[0].requirement, ">= 2.0.0");
        assert_eq!(dependencies[1].name, "jester");
        assert_eq!(dependencies[1].requirement, ">= 0.4.1");
        assert_eq!(
            extract_range(text, dependencies[1].requirement_range),
            ">= 0.4.1"
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

        let dependencies = parse_fixture(text, "file:///work/demo-1.0.0-1.rockspec", "lua");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, LuaRocks);
        assert_eq!(dependencies[0].group, "dependencies");
        assert_eq!(dependencies[0].name, "lua");
        assert_eq!(dependencies[0].requirement, ">= 5.1, < 5.5");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            ">= 5.1, < 5.5"
        );
        assert_eq!(dependencies[1].name, "luasocket");
        assert_eq!(dependencies[1].requirement, "== 3.1.0");
        assert_eq!(dependencies[2].group, "build_dependencies");
        assert_eq!(dependencies[2].name, "luafilesystem");
        assert_eq!(dependencies[3].group, "test_dependencies");
        assert_eq!(dependencies[3].name, "busted");
        assert_eq!(dependencies[3].requirement_prefix, "~> ");
    }
    #[test]
    fn parses_cpanfile_dependencies() {
        let text = package_file_fixture("parses-cpanfile-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/cpanfile", "perl");

        assert_eq!(dependencies.len(), 7);
        assert_eq!(dependencies[0].ecosystem, Cpan);
        assert_eq!(dependencies[0].group, "requires");
        assert_eq!(dependencies[0].name, "Plack");
        assert_eq!(dependencies[0].requirement, "1.0");
        assert_eq!(dependencies[1].name, "JSON");
        assert_eq!(dependencies[1].requirement, ">= 2.00, < 2.80");
        assert_eq!(
            extract_range(text, dependencies[1].requirement_range),
            ">= 2.00, < 2.80"
        );
        assert_eq!(dependencies[2].group, "recommends");
        assert_eq!(dependencies[3].group, "conflicts");
        assert_eq!(dependencies[4].group, "test.requires");
        assert_eq!(dependencies[4].name, "Test::More");
        assert_eq!(dependencies[5].group, "feature.sqlite.recommends");
        assert_eq!(dependencies[5].name, "DBD::SQLite");
        assert_eq!(dependencies[5].requirement, "0");
        assert_eq!(dependencies[6].group, "build.requires");
        assert_eq!(dependencies[6].name, "Module::Build");
    }

    #[test]
    fn parses_r_description_dependencies() {
        let text = package_file_fixture("parses-r-description-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/DESCRIPTION", "plaintext");

        assert_eq!(dependencies.len(), 7);
        assert_eq!(dependencies[0].ecosystem, Cran);
        assert_eq!(dependencies[0].group, "Version");
        assert_eq!(dependencies[0].name, "demo");
        assert_eq!(dependencies[0].requirement, "0.1.0");
        assert_eq!(dependencies[1].name, "R");
        assert_eq!(dependencies[1].requirement, ">= 4.3");
        assert_eq!(dependencies[2].name, "dplyr");
        assert_eq!(dependencies[2].requirement, ">= 1.1.0");
        assert_eq!(dependencies[3].name, "tidyr");
        assert_eq!(dependencies[3].requirement, "latest");
        assert_eq!(dependencies[4].group, "Imports");
        assert_eq!(dependencies[5].group, "Suggests");
        assert_eq!(dependencies[6].group, "LinkingTo");
    }

    #[test]
    fn parses_renv_lock_repository_packages_and_sources() {
        let text = package_file_fixture("parses-renv-lock-repository-packages-and-sources.txt");

        let dependencies = parse_fixture(text, "file:///work/renv.lock", "json");

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].ecosystem, Cran);
        assert_eq!(dependencies[0].group, "Packages");
        assert_eq!(dependencies[0].name, "dplyr");
        assert_eq!(dependencies[0].requirement, "1.1.4");
        assert_eq!(dependencies[0].hosted_url, None);
        assert_eq!(dependencies[1].name, "localpkg");
        assert_eq!(dependencies[1].hosted_url, Some("local".to_owned()));
    }

    #[test]
    fn parses_julia_project_dependencies_and_sources() {
        let text = package_file_fixture("parses-julia-project-dependencies-and-sources.lock");

        let dependencies = parse_fixture(text, "file:///work/Project.toml", "toml");

        assert_eq!(dependencies.len(), 5);
        assert_eq!(dependencies[0].ecosystem, Julia);
        assert_eq!(dependencies[0].group, "version");
        assert_eq!(dependencies[0].name, "Demo");
        assert_eq!(dependencies[0].requirement, "0.1.0");
        assert_eq!(dependencies[1].group, "compat");
        assert_eq!(dependencies[1].name, "Example");
        assert_eq!(dependencies[1].requirement, "1.2");
        assert_eq!(dependencies[2].name, "LocalPkg");
        assert_eq!(dependencies[2].requirement, "deps/LocalPkg");
        assert_eq!(dependencies[2].hosted_url, Some("path".to_owned()));
        assert_eq!(dependencies[3].name, "GitPkg");
        assert_eq!(
            dependencies[3].requirement,
            "https://github.com/example/GitPkg.jl"
        );
        assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[4].name, "julia");
        assert_eq!(dependencies[4].requirement, "1.10");
    }

    #[test]
    fn parses_julia_manifest_entries() {
        let text = package_file_fixture("parses-julia-manifest-entries.txt");

        let dependencies = parse_fixture(text, "file:///work/Manifest.toml", "toml");

        assert_eq!(dependencies.len(), 3);
        assert_eq!(dependencies[0].ecosystem, Julia);
        assert_eq!(dependencies[0].group, "deps");
        assert_eq!(dependencies[0].name, "Example");
        assert_eq!(dependencies[0].requirement, "1.2.3");
        assert_eq!(dependencies[1].name, "LocalPkg");
        assert_eq!(
            dependencies[1].requirement,
            "/home/user/.julia/dev/LocalPkg"
        );
        assert_eq!(dependencies[1].hosted_url, Some("path".to_owned()));
        assert_eq!(dependencies[2].name, "GitPkg");
        assert_eq!(
            dependencies[2].requirement,
            "https://github.com/example/GitPkg.jl.git"
        );
        assert_eq!(dependencies[2].hosted_url, Some("git".to_owned()));
    }

    #[test]
    fn parses_mix_exs_dependencies() {
        let text = package_file_fixture("parses-mix-exs-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/mix.exs", "elixir");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Hex);
        crate::support::tests::assert_dependency_metadata(
            &dependencies,
            0,
            "deps",
            "plug",
            ">= 1.15.0",
        );
        crate::support::tests::assert_two_dependency_requirements(
            &dependencies,
            "plug",
            ">= 1.15.0",
            "phoenix",
            "~> 1.7",
        );
        assert_eq!(dependencies[1].group, "deps.dev,test");
        assert_eq!(dependencies[2].name, "gettext");
        assert_eq!(
            dependencies[2].requirement,
            "https://github.com/elixir-lang/gettext.git"
        );
        assert_eq!(dependencies[2].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[3].name, "local_dependency");
        assert_eq!(dependencies[3].requirement, "../local_dependency");
        assert_eq!(dependencies[3].hosted_url, Some("path".to_owned()));
    }

    #[test]
    fn parses_haxelib_json_dependencies() {
        let text = package_file_fixture("parses-haxelib-json-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/haxelib.json", "json");

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].ecosystem, Haxelib);
        assert_eq!(dependencies[0].group, "dependencies");
        assert_eq!(dependencies[0].name, "tink_core");
        assert_eq!(dependencies[0].requirement, "2.0.0");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "2.0.0"
        );
        assert_eq!(dependencies[1].group, "dependencies");
        assert_eq!(dependencies[1].name, "tink_macro");
        assert_eq!(dependencies[1].requirement, "");
        assert_eq!(dependencies[1].hosted_url, Some("latest".to_owned()));
    }

    #[test]
    fn parses_terraform_required_providers() {
        let text = package_file_fixture("parses-terraform-required-providers.txt");

        let dependencies = parse_fixture(text, "file:///work/main.tf", "terraform");

        assert_eq!(dependencies.len(), 3);
        assert_eq!(dependencies[0].ecosystem, Terraform);
        assert_eq!(dependencies[0].group, "required_providers");
        assert_eq!(dependencies[0].name, "hashicorp/aws");
        assert_eq!(dependencies[0].hosted_name, Some("aws".to_owned()));
        assert_eq!(dependencies[0].requirement, "~> 5.0");
        assert_eq!(dependencies[0].requirement_prefix, "");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "5.0"
        );
        assert_eq!(dependencies[1].name, "hashicorp/random");
        assert_eq!(dependencies[1].hosted_name, Some("random".to_owned()));
        assert_eq!(dependencies[1].requirement, "~> 3.6");
        assert_eq!(dependencies[2].name, "terraform.io/builtin/terraform");
        assert_eq!(dependencies[2].requirement, "latest");
        assert_eq!(dependencies[2].hosted_url, Some("builtin".to_owned()));
    }

    #[test]
    fn parses_helm_chart_dependencies() {
        let text = package_file_fixture("parses-helm-chart-dependencies.txt");

        let dependencies = parse_fixture(text, "file:///work/Chart.yaml", "yaml");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Helm);
        assert_eq!(dependencies[0].group, "dependencies");
        assert_eq!(dependencies[0].name, "apache");
        assert_eq!(
            dependencies[0].hosted_url,
            Some("https://example.com/charts".to_owned())
        );
        assert_eq!(dependencies[0].requirement, "1.2.3");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "1.2.3"
        );
        assert_eq!(dependencies[1].name, "database");
        assert_eq!(dependencies[1].hosted_name, Some("mysql".to_owned()));
        assert_eq!(
            dependencies[1].hosted_url,
            Some("oci://registry.example.com/charts".to_owned())
        );
        assert_eq!(dependencies[1].requirement, "~3.2.1");
        assert_eq!(
            extract_range(text, dependencies[1].requirement_range),
            "3.2.1"
        );
        assert_eq!(dependencies[2].name, "local");
        assert_eq!(dependencies[2].hosted_url, Some("file".to_owned()));
        assert_eq!(dependencies[3].name, "repo-alias");
        assert_eq!(dependencies[3].hosted_url, Some("repo-alias".to_owned()));
    }

    #[test]
    fn parses_ansible_galaxy_requirements() {
        let text = package_file_fixture("parses-ansible-galaxy-requirements.txt");

        let dependencies = parse_fixture(text, "file:///work/requirements.yml", "yaml");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, AnsibleGalaxy);
        assert_eq!(dependencies[0].group, "roles");
        assert_eq!(dependencies[0].name, "geerlingguy.java");
        assert_eq!(dependencies[0].requirement, "1.9.6");
        assert_eq!(dependencies[1].name, "nginx");
        assert_eq!(dependencies[1].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[1].requirement, "main");
        assert_eq!(dependencies[2].group, "collections");
        assert_eq!(dependencies[2].name, "community.general");
        assert_eq!(dependencies[2].requirement, ">=7.0.0");
        assert_eq!(
            dependencies[2].hosted_url,
            Some("https://galaxy.ansible.com".to_owned())
        );
        assert_eq!(dependencies[2].requirement_prefix, ">=");
        assert_eq!(
            extract_range(text, dependencies[2].requirement_range),
            "7.0.0"
        );
        assert_eq!(dependencies[3].name, "acme.private");
        assert_eq!(
            dependencies[3].hosted_url,
            Some("https://galaxy.example.test".to_owned())
        );
    }

    #[test]
    fn parses_bazel_module_dependencies_and_overrides() {
        let text = package_file_fixture("parses-bazel-module-dependencies-and-overrides.txt");

        let dependencies = parse_fixture(text, "file:///work/MODULE.bazel", "starlark");

        assert_eq!(dependencies.len(), 5);
        assert_eq!(dependencies[0].ecosystem, Bazel);
        assert_eq!(dependencies[0].group, "bazel_dep");
        assert_eq!(dependencies[0].name, "rules_cc");
        assert_eq!(dependencies[0].requirement, "0.0.9");
        assert_eq!(dependencies[1].name, "bazel_skylib");
        assert_eq!(dependencies[1].hosted_name, Some("skylib".to_owned()));
        assert_eq!(dependencies[1].requirement, "1.3.0");
        assert_eq!(dependencies[2].group, "single_version_override");
        assert_eq!(dependencies[2].name, "protobuf");
        assert_eq!(dependencies[2].requirement, "29.0");
        assert_eq!(
            dependencies[2].hosted_url,
            Some("https://bcr.example.test".to_owned())
        );
        assert_eq!(dependencies[3].group, "git_override");
        assert_eq!(dependencies[3].name, "rules_local");
        assert_eq!(dependencies[3].hosted_url, Some("git".to_owned()));
        assert_eq!(dependencies[3].requirement, "abc123");
        assert_eq!(dependencies[4].group, "local_path_override");
        assert_eq!(dependencies[4].name, "local_rules");
        assert_eq!(dependencies[4].hosted_url, Some("path".to_owned()));
        assert_eq!(dependencies[4].requirement, "../local_rules");
        assert_eq!(
            extract_range(text, dependencies[0].requirement_range),
            "0.0.9"
        );
    }

    #[test]
    fn parses_nix_flake_inputs() {
        let text = package_file_fixture("parses-nix-flake-inputs.txt");

        let dependencies = parse_fixture(text, "file:///work/flake.nix", "nix");

        assert_eq!(dependencies.len(), 4);
        assert_eq!(dependencies[0].ecosystem, Nix);
        assert_eq!(dependencies[0].group, "inputs");
        assert_eq!(dependencies[0].name, "NixOS/nixpkgs");
        assert_eq!(dependencies[0].requirement, "nixos-unstable");
        assert_eq!(
            dependencies[0].hosted_url,
            Some("https://api.github.com/repos/NixOS/nixpkgs/tags".to_owned())
        );
        assert_eq!(dependencies[0].hosted_name, Some("nixpkgs".to_owned()));
        assert_eq!(dependencies[1].name, "numtide/flake-utils");
        assert_eq!(dependencies[1].requirement, "v1.0.0");
        assert_eq!(dependencies[2].name, "nix-community/home-manager");
        assert_eq!(dependencies[2].requirement, "24.05");
        assert_eq!(dependencies[3].name, "local");
        assert_eq!(dependencies[3].hosted_url, Some("path".to_owned()));
        assert_eq!(dependencies[3].requirement, "path:../local");
        assert_eq!(
            extract_range(text, dependencies[1].requirement_range),
            "v1.0.0"
        );
    }
    #[test]
    fn project_version_lookup_reads_npm_and_cargo_version_fields_with_exact_ranges() {
        let npm = r#"{"name":"demo","version":"0.4.4"}"#;
        let npm_versions = parse_project_versions(&DocumentInput::new(
            "file:///work/package.json".to_owned(),
            "json".to_owned(),
            npm.to_owned(),
            None,
        ));
        assert_eq!(npm_versions.len(), 1);
        assert_eq!(npm_versions[0].group, "version");
        assert_eq!(npm_versions[0].requirement, "0.4.4");
        assert_eq!(
            extract_range(npm, npm_versions[0].requirement_range),
            "0.4.4"
        );

        let cargo = "[package]\nname = \"demo\"\nversion = \"1.2.3\"\n\n[workspace.package]\nversion = \"2.0.0\"\n";
        let cargo_versions = parse_project_versions(&DocumentInput::new(
            "file:///work/Cargo.toml".to_owned(),
            "toml".to_owned(),
            cargo.to_owned(),
            None,
        ));
        assert_eq!(cargo_versions.len(), 2);
        assert_eq!(cargo_versions[0].group, "package");
        assert_eq!(cargo_versions[0].name, "version");
        assert_eq!(cargo_versions[0].requirement, "1.2.3");
        assert_eq!(
            extract_range(cargo, cargo_versions[0].requirement_range),
            "1.2.3"
        );
        assert_eq!(cargo_versions[1].requirement, "2.0.0");
        assert_eq!(
            extract_range(cargo, cargo_versions[1].requirement_range),
            "2.0.0"
        );
    }

    #[test]
    fn project_version_lookup_reads_inherited_cargo_version_as_a_workspace_reference() {
        let cargo = "[package]\nname = \"member\"\nversion.workspace = true\n";
        let versions = parse_project_versions(&DocumentInput::new(
            "file:///work/member/Cargo.toml".to_owned(),
            "toml".to_owned(),
            cargo.to_owned(),
            None,
        ));
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].requirement, "workspace:true");
        assert_eq!(extract_range(cargo, versions[0].requirement_range), "true");
    }

    #[test]
    fn project_version_lookup_takes_the_manifest_kind_for_non_file_uris() {
        let text = r#"{"name":"demo","version":"1.0.0"}"#;
        let input = DocumentInput::new(
            "package.json".to_owned(),
            "json".to_owned(),
            text.to_owned(),
            None,
        );
        let versions = parse_project_versions_as_manifest(&input, ManifestKind::NpmPackageJson);
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].requirement, "1.0.0");
    }

    fn package_file_fixture(name: &str) -> &'static str {
        crate::support::tests::fixture(
            "tests/fixtures/versionlens-parsers/src/document/tests",
            name,
        )
    }
}
