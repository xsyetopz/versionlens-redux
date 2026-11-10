mod collect;
mod deno;
mod dependency;
mod detect;
mod entry;
mod npm;
mod parse;

pub(crate) use entry::{
    looks_like_package_json, parse_composer_json_with_paths, parse_deno_json_with_paths,
    parse_dotnet_project_json_with_paths, parse_dub_json_with_paths, parse_jsr_json_with_paths,
    parse_package_json_with_paths,
};

#[cfg(test)]
mod tests {
    use crate::{DocumentInput, parse_document, parse_document_with_dependency_paths};

    mod composer {
        use crate::document::test_support::extract_range;

        use super::{DocumentInput, parse_document};
        use versionlens_model::Ecosystem::Composer;

        #[test]
        fn parses_composer_json_dependency_groups() {
            let dependencies = parse_document(&DocumentInput::new(
                "file:///work/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("parses-composer-json-dependency-groups.json").to_owned(),
                None,
            ));

            assert_eq!(dependencies.len(), 6);
            assert_eq!(dependencies[0].ecosystem, Composer);
            assert_eq!(dependencies[0].group, "version");
            assert_eq!(dependencies[1].group, "require");
            assert_eq!(dependencies[1].name, "php");
            assert_eq!(dependencies[2].name, "phpunit/phpunit");
            assert_eq!(dependencies[3].requirement, "../local");
            assert_eq!(dependencies[4].requirement, "git@example.com:org/repo.git");
            assert_eq!(dependencies[5].group, "require-dev");
        }

        #[test]
        fn parses_composer_package_link_groups() {
            let dependencies = parse_document(&DocumentInput::new(
                "file:///work/composer.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("parses-composer-package-link-groups.json").to_owned(),
                None,
            ));

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].ecosystem, Composer);
            assert_eq!(dependencies[0].group, "conflict");
            assert_eq!(dependencies[0].name, "vendor/bad");
            assert_eq!(dependencies[0].requirement, "<1.0 || >=2.0");
            assert_eq!(dependencies[1].group, "replace");
            assert_eq!(dependencies[1].name, "vendor/original");
            assert_eq!(dependencies[1].requirement, "self.version");
            assert_eq!(dependencies[2].group, "provide");
            assert_eq!(dependencies[2].name, "psr/log-implementation");
            assert_eq!(dependencies[2].requirement, "1.0|2.0");
        }

        #[test]
        fn parses_composer_stability_flags_and_references_as_suffixes() {
            let text = package_file_fixture(
                "parses-composer-stability-flags-and-references-as-suffixes.txt",
            );
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/composer.json",
                "json",
            );

            assert_eq!(dependencies.len(), 4);
            assert_eq!(dependencies[0].name, "monolog/monolog");
            assert_eq!(dependencies[0].requirement, "1.0.*");
            assert_eq!(dependencies[0].requirement_suffix, "@beta");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "1.0.*@beta"
            );
            assert_eq!(dependencies[1].name, "acme/foo");
            assert_eq!(dependencies[1].requirement, "dev-master");
            assert_eq!(dependencies[1].requirement_suffix, "#abc123");
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "dev-master#abc123"
            );
            assert_eq!(dependencies[2].name, "acme/bar");
            assert_eq!(dependencies[2].requirement, "1.0.x-dev");
            assert_eq!(dependencies[2].requirement_suffix, "#def456");
            assert_eq!(
                extract_range(text, dependencies[2].requirement_range),
                "1.0.x-dev#def456"
            );
            assert_eq!(dependencies[3].name, "acme/inline");
            assert_eq!(dependencies[3].requirement, "dev-bugfix");
            assert_eq!(dependencies[3].requirement_suffix, " as 1.0.x-dev");
            assert_eq!(
                extract_range(text, dependencies[3].requirement_range),
                "dev-bugfix as 1.0.x-dev"
            );
        }

        #[test]
        fn parses_smoke_composer_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-composer-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/composer.json",
                "json",
            );

            assert_eq!(dependencies.len(), 7);
            assert_eq!(dependencies[0].ecosystem, Composer);
            assert_eq!(dependencies[0].group, "version");
            assert_eq!(dependencies[0].name, "1.0.0");
            assert_eq!(dependencies[1].group, "require");
            assert_eq!(dependencies[1].name, "php");
            assert_eq!(dependencies[1].requirement, "^7.1.3");
            assert_eq!(dependencies[3].name, "phpunit/phpunit");
            assert_eq!(dependencies[3].requirement, "13.2.1");
            assert_eq!(dependencies[5].group, "require-dev");
            assert_eq!(dependencies[6].name, "squizlabs/php_codesniffer");
            assert_eq!(dependencies[6].requirement, "^4.0.1");
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/json_manifest/tests/composer",
                name,
            )
        }
    }
    mod deno {
        use super::{DocumentInput, parse_document_with_dependency_paths};
        use crate::document::test_support::extract_range;
        use versionlens_model::Ecosystem::*;

        #[test]
        fn parses_deno_json_imports() {
            let text = package_file_fixture("parses-deno-json-imports.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

            assert_eq!(dependencies.len(), 8);
            assert_eq!(dependencies[0].ecosystem, Deno);
            assert_deno_jsr_dependency(text, &dependencies, "@std/assert");
            assert_eq!(dependencies[1].name, "luca");
            assert_eq!(dependencies[1].requirement, "1.0.0");
            assert_eq!(dependencies[1].requirement_prefix, "jsr:@luca/cases@");
            assert_eq!(dependencies[1].hosted_name.as_deref(), Some("@luca/cases"));
            assert_eq!(dependencies[2].ecosystem, Npm);
            assert_eq!(dependencies[2].name, "chalk");
            assert_eq!(dependencies[2].requirement, "5.3.0");
            assert_eq!(dependencies[2].requirement_prefix, "npm:chalk@");
            assert_eq!(dependencies[2].hosted_name.as_deref(), Some("chalk"));
            assert_eq!(dependencies[3].name, "emptyJsr");
            assert_eq!(dependencies[3].requirement, "");
            assert_eq!(dependencies[3].requirement_prefix, "jsr:@std/assert@");
            assert_eq!(dependencies[4].ecosystem, Npm);
            assert_eq!(dependencies[4].name, "emptyNpm");
            assert_eq!(dependencies[4].requirement, "");
            assert_eq!(dependencies[4].requirement_prefix, "npm:chalk@");
            assert_eq!(dependencies[5].ecosystem, Deno);
            assert_eq!(dependencies[5].name, "url");
            assert_eq!(dependencies[5].requirement, "https://deno.land/std/mod.ts");
            assert_eq!(dependencies[6].ecosystem, Deno);
            assert_eq!(dependencies[6].group, "scopes.https://deno.land/x/app/");
            assert_eq!(dependencies[6].name, "@scope/pkg");
            assert_eq!(dependencies[6].requirement, "0.2.0");
            assert_eq!(dependencies[6].requirement_prefix, "jsr:@scope/pkg@");
            assert_eq!(dependencies[7].ecosystem, Npm);
            assert_eq!(dependencies[7].group, "scopes.https://deno.land/x/other/");
            assert_eq!(dependencies[7].name, "chalk");
            assert_eq!(dependencies[7].requirement, "5.4.0");
            assert_eq!(dependencies[7].requirement_prefix, "npm:chalk@");
        }

        #[test]
        fn deno_imports_preserve_import_specifiers_like_upstream_npm_parser() {
            let text = package_file_fixture(
                "deno-imports-preserve-import-specifiers-like-upstream-npm-parser.txt",
            );
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].ecosystem, Deno);
            assert_eq!(dependencies[0].name, "@std/assert");
            assert_eq!(dependencies[0].requirement, "^1.0.0");
            assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "jsr:@std/assert@^1.0.0"
            );
            assert_eq!(dependencies[1].ecosystem, Npm);
            assert_eq!(dependencies[1].name, "chalk");
            assert_eq!(dependencies[1].requirement, "5.3.0");
            assert_eq!(dependencies[1].requirement_prefix, "npm:chalk@");
            assert_eq!(dependencies[2].ecosystem, Deno);
            assert_eq!(dependencies[2].name, "url");
            assert_eq!(dependencies[2].requirement, "https://deno.land/std/mod.ts");
        }

        #[test]
        fn parses_configured_deno_scopes() {
            let text = package_file_fixture("parses-configured-deno-scopes.txt");
            let dependencies = parse_document_with_dependency_paths(
                &DocumentInput::new(
                    "file:///work/deno.json".to_owned(),
                    "jsonc".to_owned(),
                    text.to_owned(),
                    None,
                ),
                &["imports", "scopes"],
            );

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].group, "imports");
            assert_eq!(dependencies[0].name, "@std/assert");
            assert_eq!(dependencies[0].requirement, "^1.0.0");
            assert_eq!(dependencies[1].ecosystem, Deno);
            assert_eq!(dependencies[1].group, "scopes.https://deno.land/x/app/");
            assert_eq!(dependencies[1].name, "@scope/pkg");
            assert_eq!(dependencies[1].requirement, "0.2.0");
            assert_eq!(dependencies[2].ecosystem, Npm);
            assert_eq!(dependencies[2].group, "scopes.https://deno.land/x/other/");
            assert_eq!(dependencies[2].name, "chalk");
            assert_eq!(dependencies[2].requirement, "5.4.0");
        }

        #[test]
        fn parses_smoke_deno_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-deno-smoke-shapes.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "json");

            assert_eq!(dependencies.len(), 4);
            crate::support::tests::assert_dependency_group(&dependencies, 4, 0, Deno, "imports");
            assert_eq!(dependencies[0].name, "@std/assert");
            assert_eq!(dependencies[0].requirement, "1.0.19");
            assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "jsr:@std/assert@1.0.19"
            );
            assert_eq!(dependencies[1].name, "luca");
            assert_eq!(dependencies[1].requirement, "1.0.0");
            assert_eq!(dependencies[2].ecosystem, Npm);
            assert_eq!(dependencies[2].name, "cowsay");
            assert_eq!(dependencies[2].requirement, "1.6.0");
            assert_eq!(dependencies[3].ecosystem, Deno);
            assert_eq!(dependencies[3].name, "cases");
            assert_eq!(
                dependencies[3].requirement,
                "https://deno.land/x/case/mod.ts"
            );
        }

        #[test]
        fn parses_unversioned_deno_imports_as_empty_requirements_with_specifier_prefixes() {
            let text = package_file_fixture(
                "parses-unversioned-deno-imports-as-empty-requirements-with-specifier-prefixes.txt",
            );
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].ecosystem, Deno);
            assert_eq!(dependencies[0].name, "@std/assert");
            assert_eq!(dependencies[0].requirement, "");
            assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
            assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
            assert_eq!(dependencies[1].ecosystem, Npm);
            assert_eq!(dependencies[1].name, "chalk");
            assert_eq!(dependencies[1].requirement, "");
            assert_eq!(dependencies[1].requirement_prefix, "npm:chalk@");
        }

        #[test]
        fn parses_deno_scopes_by_default() {
            let text = package_file_fixture("parses-deno-scopes-by-default.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].group, "imports");
            assert_eq!(dependencies[0].name, "@std/assert");
            assert_eq!(dependencies[1].group, "scopes.https://deno.land/x/app/");
            assert_eq!(dependencies[1].name, "@scope/pkg");
            assert_eq!(dependencies[1].requirement, "0.2.0");
        }

        #[test]
        fn parses_deno_json_jsr_project_version() {
            let text = package_file_fixture("parses-deno-json-jsr-project-version.json");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/deno.json", "jsonc");

            assert_eq!(dependencies.len(), 2);
            assert_deno_project_version(text, &dependencies[0]);
            assert_eq!(dependencies[1].group, "imports");
            assert_eq!(dependencies[1].name, "@std/assert");
        }

        #[test]
        fn parses_import_map_json_dependencies() {
            let text = package_file_fixture("parses-import-map-json-dependencies.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/import_map.json",
                "json",
            );

            crate::support::tests::assert_dependency_group(&dependencies, 4, 0, Deno, "imports");
            assert_eq!(dependencies[0].name, "@std/async");
            assert_eq!(dependencies[0].requirement, "^1.0.0");
            assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/async@");
            assert_eq!(dependencies[0].hosted_name.as_deref(), Some("@std/async"));
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "jsr:@std/async@^1.0.0"
            );
            assert_eq!(dependencies[1].name, "@std/async/");
            assert_eq!(dependencies[1].requirement, "^1.0.0");
            assert_eq!(dependencies[1].requirement_prefix, "jsr:/@std/async@");
            assert_eq!(dependencies[1].requirement_suffix, "/");
            assert_eq!(dependencies[1].hosted_name.as_deref(), Some("@std/async"));
            assert_eq!(dependencies[2].ecosystem, Npm);
            assert_eq!(dependencies[2].name, "chalk");
            assert_eq!(dependencies[2].requirement, "5.4.0");
            assert_eq!(dependencies[2].requirement_prefix, "npm:chalk@");
            assert_eq!(dependencies[2].hosted_name.as_deref(), Some("chalk"));
            assert_eq!(dependencies[3].group, "scopes.https://deno.land/x/app/");
            assert_eq!(dependencies[3].name, "cases");
            assert_eq!(dependencies[3].requirement, "1.0.0");
            assert_eq!(dependencies[3].requirement_prefix, "jsr:@luca/cases@");
            assert_eq!(dependencies[3].hosted_name.as_deref(), Some("@luca/cases"));
        }

        #[test]
        fn parses_jsr_json_project_version() {
            let text = package_file_fixture("parses-jsr-json-project-version.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/jsr.json", "json");

            assert_eq!(dependencies.len(), 1);
            assert_deno_project_version(text, &dependencies[0]);
        }

        fn assert_deno_project_version(text: &str, dependency: &versionlens_model::Dependency) {
            assert_eq!(dependency.ecosystem, Deno);
            assert_eq!(dependency.group, "version");
            assert_eq!(dependency.name, "@scope/pkg");
            assert_eq!(dependency.requirement, "1.2.3");
            assert_eq!(extract_range(text, dependency.range), "@scope/pkg");
            assert_eq!(extract_range(text, dependency.requirement_range), "1.2.3");
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/json_manifest/tests/deno",
                name,
            )
        }

        fn assert_deno_jsr_dependency(
            text: &str,
            dependencies: &[versionlens_model::Dependency],
            name: &str,
        ) {
            assert_eq!(dependencies[0].group, "imports");
            assert_eq!(dependencies[0].name, name);
            assert_eq!(dependencies[0].requirement, "^1.0.0");
            assert_eq!(dependencies[0].requirement_prefix, "jsr:@std/assert@");
            crate::support::tests::assert_dependency_requirement_range(
                text,
                dependencies,
                0,
                "jsr:@std/assert@^1.0.0",
            );
        }
    }
    mod dotnet {
        use crate::document::test_support::extract_range;
        use versionlens_model::Ecosystem::Dotnet;

        #[test]
        fn parses_dotnet_project_json_dependencies() {
            let text = package_file_fixture("parses-dotnet-project-json-dependencies.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.json",
                "json",
            );

            assert_eq!(dependencies.len(), 4);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            assert_eq!(dependencies[0].group, "dependencies");
            assert_eq!(dependencies[0].name, "Newtonsoft.Json");
            assert_eq!(dependencies[0].requirement, "13.0.1");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "13.0.1"
            );
            assert_eq!(dependencies[1].name, "NUnit");
            assert_eq!(dependencies[1].requirement, "4.3.2");
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "4.3.2"
            );
            assert_eq!(dependencies[2].group, "frameworks.net472.dependencies");
            assert_eq!(dependencies[2].name, "System.Text.Json");
            assert_eq!(dependencies[2].requirement, "8.0.5");
            assert_eq!(
                extract_range(text, dependencies[2].requirement_range),
                "8.0.5"
            );
            assert_eq!(dependencies[3].group, "runtimes.win.dependencies");
            assert_eq!(dependencies[3].name, "runtime.win.System.IO");
            assert_eq!(dependencies[3].requirement, "4.3.0");
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/json_manifest/tests/dotnet",
                name,
            )
        }

        #[test]
        fn parses_dotnet_local_tool_manifests() {
            let text = r#"{
  "version": 1,
  "isRoot": true,
  "tools": {
    "dotnet-ef": { "version": "8.0.4", "commands": ["dotnet-ef"] },
    "csharpier": { "version": "0.28.2", "commands": ["dotnet-csharpier"] }
  }
}
"#;
            for uri in [
                "file:///work/dotnet-tools.json",
                "file:///work/.config/dotnet-tools.json",
            ] {
                let dependencies = crate::support::tests::parse_test_document(text, uri, "json");

                let found: Vec<_> = dependencies
                    .iter()
                    .map(|dependency| {
                        (
                            dependency.ecosystem,
                            dependency.group.as_str(),
                            dependency.name.as_str(),
                            extract_range(text, dependency.requirement_range),
                        )
                    })
                    .collect();
                assert_eq!(
                    found,
                    [
                        (Dotnet, "tools", "dotnet-ef", "8.0.4"),
                        (Dotnet, "tools", "csharpier", "0.28.2"),
                    ],
                    "{uri}"
                );
            }
        }
    }
    mod dub {
        use super::{DocumentInput, parse_document_with_dependency_paths};
        use crate::document::test_support::extract_range;
        use versionlens_model::Ecosystem::Dub;

        #[test]
        fn parses_dub_json_dependency_groups() {
            let text = package_file_fixture("parses-dub-json-dependency-groups.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/dub.json", "json");

            assert_eq!(dependencies.len(), 5);
            assert_dub_dependency(text, &dependencies, "dependencies", "vibe-d", "~>0.9.7");
            assert_eq!(dependencies[1].name, "painlessjson");
            assert_eq!(dependencies[1].requirement, "1.4.0");
            assert_eq!(dependencies[2].name, "local");
            assert_eq!(dependencies[2].requirement, "../local");
            assert_eq!(
                extract_range(text, dependencies[2].requirement_range),
                "../local"
            );
            assert_eq!(dependencies[3].name, "remote");
            assert_eq!(dependencies[3].requirement, "git@example.com:org/repo.git");
            assert_eq!(
                extract_range(text, dependencies[3].requirement_range),
                "git@example.com:org/repo.git"
            );
            assert_eq!(dependencies[4].group, "versions");
            assert_eq!(dependencies[4].name, "imageformats");
            assert_eq!(dependencies[4].requirement, "1.0.0");
        }

        #[test]
        fn parses_dub_json_configuration_dependencies() {
            let text = package_file_fixture("parses-dub-json-configuration-dependencies.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/dub.json", "json");

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].ecosystem, Dub);
            assert_eq!(dependencies[0].group, "configurations.tls.dependencies");
            assert_eq!(dependencies[0].name, "openssl");
            assert_eq!(dependencies[0].requirement, "~>2.0.0");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "~>2.0.0"
            );
            assert_eq!(dependencies[1].group, "configurations.tls.dependencies");
            assert_eq!(dependencies[1].name, "localdep");
            assert_eq!(dependencies[1].requirement, "../localdep");
            assert_eq!(dependencies[1].hosted_url.as_deref(), Some("path"));
        }

        #[test]
        fn parses_configured_dub_subpackages() {
            let text = package_file_fixture("parses-configured-dub-subpackages.txt");
            let dependencies = parse_document_with_dependency_paths(
                &DocumentInput::new(
                    "file:///work/dub.json".to_owned(),
                    "json".to_owned(),
                    text.to_owned(),
                    None,
                ),
                &["dependencies", "versions", "subPackages"],
            );

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].name, "vibe-d");
            assert_eq!(dependencies[0].requirement, "~>0.9.7");
            assert_eq!(dependencies[1].group, "subPackages");
            assert_eq!(dependencies[1].name, "standardpaths");
            assert_eq!(dependencies[1].requirement, "~>0.2.1");
        }

        #[test]
        fn parses_dub_selections_versions() {
            let text = package_file_fixture("parses-dub-selections-versions.json");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/dub.selections.json",
                "json",
            );

            assert_eq!(dependencies.len(), 2);
            assert_dub_dependency(text, &dependencies, "versions", "gtk-d:gtkd", "3.11.0");
        }

        #[test]
        fn parses_dub_sdl_dependency_directives() {
            let text = package_file_fixture("parses-dub-sdl-dependency-directives.selections.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/dub.sdl",
                "plaintext",
            );

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].ecosystem, Dub);
            assert_eq!(dependencies[0].group, "dependencies");
            assert_eq!(dependencies[0].name, "vibe-d");
            assert_eq!(dependencies[0].requirement, "~>0.9.7");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "~>0.9.7"
            );
            assert_eq!(dependencies[1].name, "localdep");
            assert_eq!(dependencies[1].requirement, "../localdep");
            assert_eq!(dependencies[1].hosted_url.as_deref(), Some("path"));
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "../localdep"
            );
            assert_eq!(dependencies[2].name, "remote");
            assert_eq!(
                dependencies[2].requirement,
                "git+https://example.org/remote.git"
            );
            assert_eq!(
                extract_range(text, dependencies[2].requirement_range),
                "git+https://example.org/remote.git"
            );
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/json_manifest/tests/dub",
                name,
            )
        }

        fn assert_dub_dependency(
            text: &str,
            dependencies: &[versionlens_model::Dependency],
            group: &str,
            name: &str,
            requirement: &str,
        ) {
            assert_eq!(dependencies[0].ecosystem, Dub);
            assert_eq!(dependencies[0].group, group);
            assert_eq!(dependencies[0].name, name);
            assert_eq!(dependencies[0].requirement, requirement);
            crate::support::tests::assert_dependency_requirement_range(
                text,
                dependencies,
                0,
                requirement,
            );
        }
    }
    mod npm {
        use super::{DocumentInput, parse_document_with_dependency_paths};

        mod basic {
            use crate::document::test_support::extract_range;
            use versionlens_model::Ecosystem::Npm;

            #[test]
            fn parses_package_json_dependency_groups() {
                let (text, dependencies) =
                    parse_package_fixture("package-json-dependency-groups.txt");

                assert_eq!(dependencies.len(), 11);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "dependencies");
                assert_eq!(dependencies[0].name, "@types/node");
                assert_eq!(dependencies[0].requirement, "26.0.1");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "26.0.1"
                );
                assert_eq!(dependencies[1].name, "pacote");
                assert_eq!(dependencies[1].requirement, "11.1.9");
                assert_eq!(dependencies[1].requirement_prefix, "npm:pacote@");
                assert_eq!(
                    extract_range(text, dependencies[1].requirement_range),
                    "npm:pacote@11.1.9"
                );
                assert_eq!(dependencies[2].name, "@types/react");
                assert_eq!(dependencies[2].requirement, "^19.0.0");
                assert_eq!(dependencies[2].requirement_prefix, "npm:@types/react@");
                assert_eq!(
                    extract_range(text, dependencies[2].requirement_range),
                    "npm:@types/react@^19.0.0"
                );
                assert_eq!(dependencies[3].name, "types-react");
                assert_eq!(dependencies[3].requirement, "");
                assert_eq!(dependencies[3].requirement_prefix, "npm:types-react@");
                assert_eq!(extract_range(text, dependencies[3].requirement_range), "");
                assert_eq!(dependencies[4].name, "@types/react");
                assert_eq!(dependencies[4].requirement, "");
                assert_eq!(dependencies[4].requirement_prefix, "npm:@types/react@");
                assert_eq!(extract_range(text, dependencies[4].requirement_range), "");
                assert_eq!(dependencies[5].name, "local-file");
                assert_eq!(dependencies[5].requirement, "file:../local");
                assert_eq!(
                    extract_range(text, dependencies[5].requirement_range),
                    "file:../local"
                );
                assert_eq!(dependencies[6].name, "local");
                assert_eq!(dependencies[6].requirement, "file:../local/package.json");
                assert_eq!(
                    extract_range(text, dependencies[6].requirement_range),
                    "link:../local"
                );
                assert_eq!(dependencies[9].group, "devDependencies");
                assert_eq!(dependencies[10].group, "peerDependencies");
            }

            #[test]
            fn parses_package_json5_dependency_groups() {
                let text = package_file_fixture("package-json5-dependency-groups.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json5",
                    "json5",
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "dependencies");
                assert_eq!(dependencies[0].name, "left-pad");
                assert_eq!(dependencies[0].requirement, "1.0.0");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "1.0.0"
                );
            }

            #[test]
            fn package_json_ranges_count_utf16_code_units_before_dependencies() {
                let text = package_file_fixture(
                    "package-json-ranges-count-utf16-code-units-before-dependencies.json5",
                );
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].name, "left-pad");
                assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
                assert_eq!(dependencies[0].range.start.character, 31);
            }

            #[test]
            fn parses_npm_bundle_dependency_name_arrays_by_default() {
                let (text, dependencies) =
                    parse_package_fixture("bundle-dependency-name-arrays-by-default.json");

                assert_eq!(dependencies.len(), 2);
                assert_eq!(dependencies[0].group, "bundledDependencies");
                assert_eq!(dependencies[0].name, "left-pad");
                assert_eq!(dependencies[0].requirement, "");
                assert_eq!(extract_range(text, dependencies[0].range), "left-pad");
                assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
                assert_eq!(dependencies[1].group, "bundleDependencies");
                assert_eq!(dependencies[1].name, "right-pad");
                assert_eq!(dependencies[1].requirement, "");
            }

            #[test]
            fn parses_bun_trusted_dependencies_by_default() {
                let (text, dependencies) =
                    parse_package_fixture("bun-trusted-dependencies-by-default.txt");

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "trustedDependencies");
                assert_eq!(dependencies[0].name, "my-trusted-package");
                assert_eq!(dependencies[0].requirement, "");
                assert_eq!(
                    extract_range(text, dependencies[0].range),
                    "my-trusted-package"
                );
                assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
            }

            #[test]
            fn parses_yarn_resolutions_by_default() {
                let (text, dependencies) =
                    parse_package_fixture("yarn-resolutions-by-default.json");

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "resolutions");
                assert_eq!(dependencies[0].name, "left-pad");
                assert_eq!(dependencies[0].requirement, "1.1.0");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "1.1.0"
                );
            }

            #[test]
            fn parses_yarn_resolution_selectors_by_terminal_package() {
                let (_, dependencies) =
                    parse_package_fixture("yarn-resolution-selectors-by-terminal-package.json");

                assert_eq!(dependencies.len(), 4);
                assert_eq!(dependencies[0].group, "resolutions");
                assert_eq!(dependencies[0].name, "memory-fs");
                assert_eq!(dependencies[0].requirement, "0.4.1");
                assert_eq!(dependencies[1].name, "json5");
                assert_eq!(dependencies[1].requirement, "2.1.0");
                assert_eq!(dependencies[2].name, "@babel/generator");
                assert_eq!(dependencies[2].requirement, "7.3.4");
                assert_eq!(dependencies[3].name, "@babel/generator");
                assert_eq!(dependencies[3].requirement, "7.3.4");
            }

            #[test]
            fn parses_package_manager_prerelease_build_metadata() {
                let (text, dependencies) =
                    parse_package_fixture("package-manager-prerelease-build-metadata.txt");

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].group, "packageManager");
                assert_eq!(dependencies[0].name, "pnpm");
                assert_eq!(
                    dependencies[0].requirement,
                    "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
                );
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "9.0.0-rc.2+sha.6d21a1f908b66fe37f42f3170d4ba8fd5e2dcde886ec85863a5e7cac"
                );
            }

            #[test]
            fn ignores_invalid_package_manager_values() {
                let (_, dependencies) =
                    parse_package_fixture("ignores-invalid-package-manager-values.json");

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].name, "left-pad");
            }

            #[test]
            fn parses_dev_engines_package_manager_version() {
                let (text, dependencies) =
                    parse_package_fixture("dev-engines-package-manager-version.json");

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "devEngines.packageManager");
                assert_eq!(dependencies[0].name, "npm");
                assert_eq!(dependencies[0].requirement, "^10.0.0");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "^10.0.0"
                );
            }

            #[test]
            fn parses_npm_overrides_dot_self_override() {
                let (text, dependencies) =
                    parse_package_fixture("overrides-dot-self-override.json");

                assert_eq!(dependencies.len(), 2);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "overrides");
                assert_eq!(dependencies[0].name, "foo");
                assert_eq!(dependencies[0].requirement, "1.0.0");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "1.0.0"
                );
                assert_eq!(dependencies[1].group, "overrides");
                assert_eq!(dependencies[1].name, "bar");
                assert_eq!(dependencies[1].requirement, "2.0.0");
            }

            #[test]
            fn parses_scoped_npm_override_version_selector_by_package_name() {
                assert_scoped_override_fixture(
                    "scoped-npm-override-version-selector-by-package-name.txt",
                );
            }

            #[test]
            fn parses_scoped_npm_override_dot_self_selector_by_package_name() {
                assert_scoped_override_fixture(
                    "scoped-npm-override-dot-self-selector-by-package-name.json",
                );
            }

            fn assert_scoped_override_fixture(name: &str) {
                let text = package_file_fixture(name);
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );
                let dependency = dependencies.first().expect("scoped override dependency");
                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependency.ecosystem, Npm);
                assert_eq!(dependency.group, "overrides");
                assert_eq!(dependency.name, "@scope/pkg");
                assert_eq!(dependency.requirement, "1.2.4");
                assert_eq!(extract_range(text, dependency.requirement_range), "1.2.4");
            }

            fn parse_package_fixture(
                name: &str,
            ) -> (&'static str, Vec<versionlens_model::Dependency>) {
                let text = package_file_fixture(name);
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );
                (text, dependencies)
            }

            fn package_file_fixture(name: &str) -> &'static str {
                crate::support::tests::fixture(
                    "tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_basic",
                    name,
                )
            }
        }
        mod github {
            use crate::document::test_support::extract_range;

            #[test]
            fn parses_package_json_github_dependencies() {
                let text = package_file_fixture("parses-package-json-github-dependencies.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 8);
                assert_eq!(dependencies[0].name, "octokit/core.js");
                assert_eq!(dependencies[0].requirement, "^2");
                assert_eq!(
                    dependencies[0].requirement_prefix,
                    "github:octokit/core.js#semver:"
                );
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "github:octokit/core.js#semver:^2"
                );
                assert_eq!(
                    dependencies[0].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/octokit/core.js/tags")
                );
                assert_eq!(dependencies[1].name, "owner/plain");
                assert_eq!(dependencies[1].requirement, "v1.0.0");
                assert_eq!(dependencies[1].requirement_prefix, "github:owner/plain#");
                assert_eq!(
                    dependencies[1].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/plain/tags")
                );
                assert_eq!(dependencies[2].name, "owner/commit");
                assert_eq!(dependencies[2].requirement, "abcdef1");
                assert_eq!(dependencies[2].requirement_prefix, "github:owner/commit#");
                assert_eq!(
                    dependencies[2].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/commit/commits")
                );
                assert_eq!(dependencies[3].name, "owner/shortcut");
                assert_eq!(dependencies[3].requirement, "v2.0.0");
                assert_eq!(dependencies[3].requirement_prefix, "owner/shortcut#");
                assert_eq!(
                    dependencies[3].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/shortcut/tags")
                );
                assert_eq!(dependencies[4].name, "owner/url");
                assert_eq!(dependencies[4].requirement, "^3");
                assert_eq!(
                    dependencies[4].requirement_prefix,
                    "git+https://github.com/owner/url.git#semver:"
                );
                assert_eq!(
                    dependencies[4].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/url/tags")
                );
                assert_eq!(dependencies[5].name, "owner/ssh");
                assert_eq!(dependencies[5].requirement, "1234567");
                assert_eq!(
                    dependencies[5].requirement_prefix,
                    "git@github.com:owner/ssh.git#"
                );
                assert_eq!(
                    dependencies[5].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/ssh/commits")
                );
                assert_eq!(dependencies[6].name, "owner/git-ssh");
                assert_eq!(dependencies[6].requirement, "7654321");
                assert_eq!(
                    dependencies[6].requirement_prefix,
                    "git+ssh://git@github.com/owner/git-ssh.git#"
                );
                assert_eq!(
                    dependencies[6].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/git-ssh/commits")
                );
                assert_eq!(dependencies[7].name, "owner/bare");
                assert_eq!(dependencies[7].requirement, "");
                assert_eq!(dependencies[7].requirement_prefix, "github:owner/bare#");
                assert_eq!(
                    extract_range(text, dependencies[7].requirement_range),
                    "github:owner/bare"
                );
                assert_eq!(dependencies[7].hosted_url.as_deref(), Some("branch"));
            }

            #[test]
            fn parses_package_json_github_ssh_colon_dependencies() {
                let text =
                    package_file_fixture("parses-package-json-github-ssh-colon-dependencies.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].name, "owner/git-ssh-colon");
                assert_eq!(dependencies[0].requirement, "89abcde");
                assert_eq!(
                    dependencies[0].requirement_prefix,
                    "git+ssh://git@github.com:owner/git-ssh-colon.git#"
                );
                assert_eq!(
                    dependencies[0].hosted_url.as_deref(),
                    Some("https://api.github.com/repos/owner/git-ssh-colon/commits")
                );
            }

            #[test]
            fn parses_github_url_without_ref_as_plain_git_dependency() {
                let text = package_file_fixture(
                    "parses-github-url-without-ref-as-plain-git-dependency.json",
                );
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].name, "git-url");
                assert_eq!(
                    dependencies[0].requirement,
                    "git+https://github.com/owner/url.git"
                );
                assert_eq!(dependencies[0].hosted_url, None);
            }

            #[test]
            fn parses_package_json_github_branch_dependencies_as_branch_references() {
                let text = package_file_fixture(
                    "parses-package-json-github-branch-dependencies-as-branch-references.json",
                );
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].name, "owner/branch");
                assert_eq!(dependencies[0].requirement, "main");
                assert_eq!(dependencies[0].hosted_url.as_deref(), Some("branch"));
            }

            fn package_file_fixture(name: &str) -> &'static str {
                crate::support::tests::fixture(
                    "tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_github",
                    name,
                )
            }
        }
        mod nested {
            use super::{DocumentInput, parse_document_with_dependency_paths};
            use crate::document::test_support::extract_range;
            use versionlens_model::Ecosystem::Npm;

            #[test]
            fn parses_package_json_nested_wildcard_and_scalars() {
                let text =
                    package_file_fixture("parses-package-json-nested-wildcard-and-scalars.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "jsonc",
                );

                assert_eq!(dependencies.len(), 8);
                assert_eq!(dependencies[0].group, "packageManager");
                assert_eq!(dependencies[0].name, "pnpm");
                assert_eq!(dependencies[0].requirement, "9.1.2");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "9.1.2"
                );
                assert_eq!(dependencies[1].name, "react");
                assert_eq!(dependencies[2].name, "@scope/pkg");
                assert_eq!(dependencies[3].group, "overrides");
                assert_eq!(dependencies[3].name, "child");
                assert_eq!(dependencies[4].group, "jspm.dependencies");
                assert_eq!(dependencies[5].group, "pnpm.overrides");
                assert_eq!(dependencies[5].name, "leaf");
                assert_eq!(dependencies[6].group, "workspaces.catalog");
                assert_eq!(dependencies[6].name, "react-dom");
                assert_eq!(dependencies[7].group, "workspaces.catalogs.testing");
                assert_eq!(dependencies[7].name, "jest");
            }

            #[test]
            fn parses_package_json_pnpm_package_extensions_by_default() {
                let text = package_file_fixture(
                    "parses-package-json-pnpm-package-extensions-by-default.txt",
                );
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 2);
                assert_eq!(
                    dependencies[0].group,
                    "pnpm.packageExtensions.react-redux.peerDependencies"
                );
                assert_eq!(dependencies[0].name, "react-dom");
                assert_eq!(dependencies[0].requirement, "*");
                assert_eq!(extract_range(text, dependencies[0].requirement_range), "*");
                assert_eq!(
                    dependencies[1].group,
                    "pnpm.packageExtensions.vite@5.optionalDependencies"
                );
                assert_eq!(dependencies[1].name, "fsevents");
                assert_eq!(dependencies[1].requirement, "^2.3.3");
            }

            #[test]
            fn parses_package_json_bun_catalogs_by_default() {
                let text = package_file_fixture("parses-package-json-bun-catalogs-by-default.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 6);
                assert_eq!(dependencies[0].group, "dependencies");
                assert_eq!(dependencies[0].name, "react");
                assert_eq!(dependencies[1].group, "catalog");
                assert_eq!(dependencies[1].name, "typescript");
                assert_eq!(dependencies[2].group, "catalogs.build");
                assert_eq!(dependencies[2].name, "webpack");
                assert_eq!(dependencies[3].group, "workspaces.catalog");
                assert_eq!(dependencies[3].name, "react");
                assert_eq!(dependencies[4].name, "react-dom");
                assert_eq!(dependencies[5].group, "workspaces.catalogs.testing");
                assert_eq!(dependencies[5].name, "jest");
            }

            #[test]
            fn parses_configured_smoke_npm_custom_dependency_paths() {
                let text =
                    package_file_fixture("parses-configured-smoke-npm-custom-dependency-paths.txt");
                let dependencies = parse_document_with_dependency_paths(
                    &DocumentInput::new(
                        "file:///work/package.json".to_owned(),
                        "json".to_owned(),
                        text.to_owned(),
                        None,
                    ),
                    &["customDependencies".to_owned()],
                );

                assert_eq!(dependencies.len(), 1);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "customDependencies");
                assert_eq!(dependencies[0].name, "@types/hammerjs");
                assert_eq!(dependencies[0].requirement, "2.0.33");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "2.0.33"
                );
            }

            #[test]
            fn ignores_array_elements_under_terminal_wildcard_path() {
                let text = r#"{"overrides":[{"name":"foo","version":"1.0.0"}]}"#;
                let dependencies = parse_document_with_dependency_paths(
                    &DocumentInput::new(
                        "file:///work/package.json".to_owned(),
                        "json".to_owned(),
                        text.to_owned(),
                        None,
                    ),
                    &["overrides.*".to_owned()],
                );

                assert_eq!(dependencies.len(), 0);
            }

            fn package_file_fixture(name: &str) -> &'static str {
                crate::support::tests::fixture(
                    "tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_nested",
                    name,
                )
            }

            #[test]
            fn package_json_version_is_not_a_dependency() {
                let dependencies = crate::support::tests::parse_test_document(
                    r#"{"name":"demo","version":"0.4.4"}"#,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies, vec![]);
            }

            #[test]
            fn configured_package_json_version_path_is_not_a_dependency() {
                let input = DocumentInput::new(
                    "file:///work/package.json".to_owned(),
                    "json".to_owned(),
                    r#"{"name":"demo","version":"0.4.4","dependencies":{"react":"^18.0.0"}}"#
                        .to_owned(),
                    None,
                );

                let dependencies =
                    parse_document_with_dependency_paths(&input, &["version", "dependencies"]);

                let names = dependencies
                    .iter()
                    .map(|dependency| dependency.name.as_str())
                    .collect::<Vec<_>>();
                assert_eq!(names, ["react"]);
            }
        }
        mod smoke {
            use crate::document::test_support::extract_range;
            use versionlens_model::Ecosystem::Npm;

            #[test]
            fn parses_smoke_npm_range_smoke_shapes() {
                let text = package_file_fixture("parses-smoke-npm-range-smoke-shapes.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                crate::support::tests::assert_dependency_group(
                    &dependencies,
                    2,
                    0,
                    Npm,
                    "dependencies",
                );
                assert_eq!(dependencies[0].name, "@faker-js/faker");
                assert_eq!(dependencies[0].requirement, "> 10.0.0 < 10.5.0");
                assert_eq!(
                    extract_range(text, dependencies[0].requirement_range),
                    "> 10.0.0 < 10.5.0"
                );
                assert_eq!(dependencies[1].name, "typescript");
                assert_eq!(dependencies[1].requirement, "> 6.0.0 < 6.0.3");
            }

            #[test]
            fn parses_smoke_jspm_package_json_smoke_shapes() {
                let text = package_file_fixture("parses-smoke-jspm-package-json-smoke-shapes.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 6);
                assert_eq!(dependencies[0].group, "jspm.dependencies");
                assert_eq!(dependencies[0].name, "bluebird");
                assert_eq!(dependencies[0].requirement, "^3.7.2");
                assert_eq!(dependencies[1].requirement, "*");
                assert_eq!(dependencies[2].name, "twbs/bootstrap");
                assert_eq!(dependencies[2].requirement, "v5.3.8");
                assert_eq!(dependencies[2].requirement_prefix, "github:twbs/bootstrap#");
                assert_eq!(dependencies[5].group, "jspm.devDependencies");
                assert_eq!(dependencies[5].name, "core-js");
                assert_eq!(dependencies[5].requirement, "^3.49.0");
            }

            #[test]
            fn parses_smoke_typical_package_json_smoke_shapes() {
                let text =
                    package_file_fixture("parses-smoke-typical-package-json-smoke-shapes.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 14);
                assert_eq!(dependencies[1].name, "typescript");
                assert_eq!(dependencies[1].requirement, "6.0.3");
                assert_eq!(dependencies[2].name, "twbs/bootstrap");
                assert_eq!(dependencies[2].requirement, "v5.3.8");
                assert_eq!(dependencies[3].name, "expressjs/express");
                assert_eq!(dependencies[3].requirement, "v5.2.1");
                assert_eq!(
                    dependencies[3].requirement_prefix,
                    "expressjs/express#semver:"
                );
                assert_eq!(dependencies[6].name, "@types/node");
                assert_eq!(dependencies[6].requirement, "latest");
                assert_eq!(dependencies[12].name, "invalid");
                assert_eq!(dependencies[12].requirement, ">=4.5.");
                assert!(!dependencies.iter().any(|dependency| {
                    dependency.group == "customDependencies" && dependency.name == "@types/hammerjs"
                }));
            }

            #[test]
            fn parses_smoke_npm_workspaces_smoke_shapes() {
                let text = package_file_fixture("parses-smoke-npm-workspaces-smoke-shapes.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 5);
                assert_eq!(dependencies[0].group, "dependencies");
                assert_eq!(dependencies[0].name, "react");
                assert_eq!(dependencies[1].group, "workspaces.catalog");
                assert_eq!(dependencies[1].name, "react");
                assert_eq!(dependencies[2].name, "react-dom");
                assert_eq!(dependencies[3].group, "workspaces.catalogs.testing");
                assert_eq!(dependencies[3].name, "jest");
                assert_eq!(dependencies[4].name, "testing-library");
            }

            #[test]
            fn parses_smoke_npm_overrides_smoke_shapes() {
                let text = package_file_fixture("parses-smoke-npm-overrides-smoke-shapes.txt");
                let dependencies = crate::support::tests::parse_test_document(
                    text,
                    "file:///work/package.json",
                    "json",
                );

                assert_eq!(dependencies.len(), 3);
                assert_eq!(dependencies[0].ecosystem, Npm);
                assert_eq!(dependencies[0].group, "overrides");
                assert_eq!(dependencies[0].name, "semver");
                assert_eq!(dependencies[0].requirement, "7.8.5");
                assert_eq!(dependencies[1].group, "overrides");
                assert_eq!(dependencies[1].name, "typescript");
                assert_eq!(dependencies[2].name, "semver");
            }

            fn package_file_fixture(name: &str) -> &'static str {
                crate::support::tests::fixture(
                    "tests/fixtures/versionlens-parsers/src/json_manifest/tests/npm_smoke",
                    name,
                )
            }
        }
    }
    mod smoke {
        use super::{DocumentInput, parse_document};
        use versionlens_model::Ecosystem::*;

        #[test]
        fn parses_smoke_pnpm_package_json_smoke_shapes() {
            let text = package_file_fixture("smoke-pnpm-package-json-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/package.json",
                "json",
            );

            assert_eq!(dependencies.len(), 8);
            assert_eq!(dependencies[0].ecosystem, Npm);
            assert_eq!(dependencies[0].group, "packageManager");
            assert_eq!(dependencies[0].name, "pnpm");
            assert_eq!(dependencies[0].requirement, "10.34.4");
            assert_eq!(dependencies[1].requirement, "workspace:*");
            assert_eq!(dependencies[2].requirement, "catalog:*");
            assert_eq!(
                dependencies[3].requirement,
                "file:../overrides/package.json"
            );
            assert_eq!(dependencies[4].name, "types-react");
            assert_eq!(dependencies[4].requirement, "");
            assert_eq!(dependencies[4].requirement_prefix, "npm:types-react@");
            assert_eq!(dependencies[5].group, "pnpm.overrides");
            assert_eq!(dependencies[6].name, "axios");
            assert_eq!(dependencies[7].name, "typescript");
        }

        #[test]
        fn parses_smoke_npm_custom_file_smoke_shapes() {
            let text = package_file_fixture("smoke-npm-custom-file-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/web-module.json",
                "json",
            );

            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].ecosystem, Npm);
            assert_eq!(dependencies[0].group, "devDependencies");
            assert_eq!(dependencies[0].name, "typescript");
            assert_eq!(dependencies[0].requirement, "^6.0.3");
        }

        #[test]
        fn parses_smoke_npm_git_smoke_shapes() {
            let text = package_file_fixture("npm-git-smoke-shapes.json");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/package.json",
                "json",
            );

            crate::support::tests::assert_dependency_group(
                &dependencies,
                2,
                0,
                Npm,
                "devDependencies",
            );
            assert_eq!(dependencies[0].name, "gitpkgnotfound1");
            assert_eq!(
                dependencies[0].requirement,
                "git+https://git@github.com/testuser/test.git"
            );
            assert_eq!(dependencies[1].name, "gitpkgnotfound2");
            assert_eq!(
                dependencies[1].requirement,
                "git+ssh://git@some.com/testuser/test.git"
            );
        }

        #[test]
        fn parses_smoke_npm_faq_and_npmrc_smoke_shapes() {
            let faq = r#"{
  "devDependencies": {
    "projectz": "4.2.0",
    "@types/node": "26.0.1",
    "typescript": "^6.0.3"
  }
}"#;
            let faq_dependencies = parse_document(&DocumentInput::new(
                "file:///work/package.json".to_owned(),
                "json".to_owned(),
                faq.to_owned(),
                None,
            ));

            assert_eq!(faq_dependencies.len(), 3);
            assert_eq!(faq_dependencies[0].ecosystem, Npm);
            assert_eq!(faq_dependencies[0].group, "devDependencies");
            assert_eq!(faq_dependencies[0].name, "projectz");
            assert_eq!(faq_dependencies[0].requirement, "4.2.0");
            assert_eq!(faq_dependencies[1].name, "@types/node");
            assert_eq!(faq_dependencies[1].requirement, "26.0.1");
            assert_eq!(faq_dependencies[2].name, "typescript");
            assert_eq!(faq_dependencies[2].requirement, "^6.0.3");

            let npmrc = r#"{
  "name": "smoke-test",
  "title": "smoke test",
  "dependencies": {},
  "devDependencies": {
    "@scope/some-package": "0.1"
  }
}"#;
            let npmrc_dependencies = parse_document(&DocumentInput::new(
                "file:///work/package.json".to_owned(),
                "json".to_owned(),
                npmrc.to_owned(),
                None,
            ));

            assert_eq!(npmrc_dependencies.len(), 1);
            assert_eq!(npmrc_dependencies[0].ecosystem, Npm);
            assert_eq!(npmrc_dependencies[0].group, "devDependencies");
            assert_eq!(npmrc_dependencies[0].name, "@scope/some-package");
            assert_eq!(npmrc_dependencies[0].requirement, "0.1");
        }

        #[test]
        fn parses_smoke_dub_smoke_shapes() {
            let text = package_file_fixture("dub-smoke-shapes.json");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/dub.json", "json");

            assert_eq!(dependencies.len(), 8);
            assert_eq!(dependencies[0].ecosystem, Dub);
            assert_eq!(dependencies[0].name, "gtk-d:gtkd");
            assert_eq!(dependencies[0].requirement, "~>3.11.0");
            assert_eq!(dependencies[4].name, "painlessjson");
            assert_eq!(dependencies[4].requirement, "1.4.0");
        }

        #[test]
        fn parses_smoke_dub_selections_smoke_shapes() {
            let text = package_file_fixture("dub-selections-smoke-shapes.json");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/dub.selections.json",
                "json",
            );

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].ecosystem, Dub);
            assert_eq!(dependencies[0].group, "versions");
            assert_eq!(dependencies[0].name, "gtk-d:gtkd");
            assert_eq!(dependencies[0].requirement, "3.11.0");
            assert_eq!(dependencies[2].name, "derelict-sdl2");
            assert_eq!(dependencies[2].requirement, "2.1.4");
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/json_manifest/tests/smoke",
                name,
            )
        }
    }
}
