use versionlens_model::Ecosystem::Dotnet;
use versionlens_model::{Dependency, Ecosystem};

use crate::config::DependencyPropertyConfig;

use versionlens_parsers::path_matches as property_matches;

pub(crate) fn is_enabled<'a>(
    dependency: &Dependency,
    manifest_ecosystem: Ecosystem,
    configs: impl IntoIterator<Item = &'a DependencyPropertyConfig>,
) -> bool {
    let group = dependency_property_group(dependency);
    let member = format!("{group}.{}", dependency.name);
    let mut has_ecosystem_config = false;

    for config in configs
        .into_iter()
        .filter(|config| config.ecosystem == manifest_ecosystem)
    {
        has_ecosystem_config = true;
        if config.properties.iter().any(|property| {
            property_matches(property, &group) || property_matches(property, &member)
        }) {
            return true;
        }
    }

    !has_ecosystem_config
}

fn dependency_property_group(dependency: &Dependency) -> String {
    match dependency.ecosystem {
        Dotnet => dotnet_dependency_property_group(dependency),
        _ => dependency.group.as_str().to_owned(),
    }
}

fn dotnet_dependency_property_group(dependency: &Dependency) -> String {
    match dependency.group.as_str() {
        "dependencies" => dependency.group.as_str().to_owned(),
        group if group.starts_with("frameworks.") || group.starts_with("runtimes.") => {
            group.to_owned()
        }
        group if group.starts_with("paket.") => group.to_owned(),
        "packages.package" => "packages.package".to_owned(),
        "PropertyGroup" => format!("Project.PropertyGroup.{}", dependency.name),
        "Sdk" | "Project.Sdk" => "Project.Sdk".to_owned(),
        group => format!("Project.ItemGroup.{group}"),
    }
}

#[cfg(test)]
mod tests {
    use versionlens_model::{DocumentInput, Ecosystem, ManifestKind};

    use crate::{DependencyPropertyConfig, ProviderSettings, VersionLensSession};

    fn session_with_properties(ecosystem: Ecosystem, properties: &[&str]) -> VersionLensSession {
        session_with_property_configs(&[(ecosystem, properties)])
    }

    fn session_with_property_configs(configs: &[(Ecosystem, &[&str])]) -> VersionLensSession {
        session_with_scoped_property_configs(
            &configs
                .iter()
                .map(|(ecosystem, properties)| (*ecosystem, None, *properties))
                .collect::<Vec<_>>(),
        )
    }

    fn session_with_scoped_property_configs(
        configs: &[(Ecosystem, Option<ManifestKind>, &[&str])],
    ) -> VersionLensSession {
        crate::version_lens_session(crate::test_support::session_config(
            ProviderSettings {
                dependency_properties: configs
                    .iter()
                    .map(
                        |(ecosystem, manifest_kind, properties)| DependencyPropertyConfig {
                            ecosystem: *ecosystem,
                            manifest_kind: *manifest_kind,
                            properties: properties
                                .iter()
                                .map(|property| (*property).to_owned())
                                .collect(),
                        },
                    )
                    .collect(),
                ..Default::default()
            },
            true,
        ))
    }

    fn package_file_fixture(name: &str) -> String {
        crate::test_support::fixture("tests/fixtures/dependency-properties", name)
    }

    mod cargo {
        use super::{DocumentInput, package_file_fixture, session_with_properties};
        use versionlens_model::Ecosystem::Cargo;

        #[test]
        fn cargo_target_dependency_properties_match_wildcards() {
            let session = session_with_properties(Cargo, &["target.*.dependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///Cargo.toml".to_owned(),
                "toml".to_owned(),
                package_file_fixture("cargo-target-dependencies.toml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "nix");
        }

        #[test]
        fn dependency_properties_allow_custom_cargo_paths() {
            let session =
                session_with_properties(Cargo, &["workspace.metadata.versionlens.dependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///Cargo.toml".to_owned(),
                "toml".to_owned(),
                package_file_fixture("cargo-custom-workspace-metadata.toml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "custom");
        }
    }
    mod npm {
        use super::{
            DocumentInput, package_file_fixture, session_with_properties,
            session_with_property_configs, session_with_scoped_property_configs,
        };
        use versionlens_model::Ecosystem::*;
        use versionlens_model::ManifestKind::{NpmPackageJson, PnpmYaml};

        fn assert_single_dependency(output: &crate::AnalyzeDocumentOutput, name: &str) {
            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, name);
        }

        #[test]
        fn dependency_properties_are_filtered_in_rust() {
            let session = session_with_properties(Npm, &["devDependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("dev-dependencies.json"),
                None,
            ));

            assert_single_dependency(&output, "is-odd");
        }

        #[test]
        fn dependency_properties_allow_custom_json_paths() {
            let session = session_with_properties(Npm, &["resolutions"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("resolutions.json"),
                None,
            ));

            assert_single_dependency(&output, "is-even");
        }

        #[test]
        fn dependency_properties_match_wildcard_groups() {
            let session = session_with_properties(Npm, &["packageExtensions.*.peerDependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pnpm-workspace.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("package-extensions-peer.yaml"),
                None,
            ));

            assert_single_dependency(&output, "@types/react");
        }

        #[test]
        fn deno_dependency_properties_allow_scopes() {
            let session = session_with_properties(Deno, &["scopes"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///import_map.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("deno-import-map-scopes.json"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(
                output.dependencies[0].group,
                "scopes.https://deno.land/x/app/"
            );
            assert_eq!(output.dependencies[0].name, "@scope/pkg");
        }

        #[test]
        fn deno_dependency_properties_filter_before_extraction() {
            let session = session_with_properties(Deno, &["imports"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///deno.json".to_owned(),
                "jsonc".to_owned(),
                package_file_fixture("deno-import-map-scopes.json"),
                None,
            ));

            assert_single_dependency(&output, "@std/assert");
        }

        #[test]
        fn dependency_properties_merge_same_ecosystem_configs() {
            let session = session_with_property_configs(&[
                (Npm, &["devDependencies"][..]),
                (Npm, &["packageExtensions.*.peerDependencies"][..]),
            ]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pnpm-workspace.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("package-extensions-scheduler.yaml"),
                None,
            ));

            assert_single_dependency(&output, "scheduler");
        }

        #[test]
        fn dependency_properties_allow_custom_pnpm_paths() {
            let session = session_with_properties(Npm, &["customCatalog"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pnpm-workspace.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("custom-catalog.yaml"),
                None,
            ));

            assert_single_dependency(&output, "scheduler");
        }

        #[test]
        fn scoped_npm_dependency_properties_apply_to_package_json5() {
            let session = session_with_scoped_property_configs(&[(
                Npm,
                Some(NpmPackageJson),
                &["devDependencies"][..],
            )]);
            let input = DocumentInput::new(
                "file:///package.json5".to_owned(),
                "json5".to_owned(),
                package_file_fixture("package-dev-dependencies.json5"),
                None,
            );

            let dependencies = session.dependencies(&input);

            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].name, "typescript");
            assert_eq!(dependencies[0].group, "devDependencies");
        }

        #[test]
        fn scoped_npm_dependency_properties_apply_to_package_yaml() {
            let session = session_with_scoped_property_configs(&[(
                Npm,
                Some(NpmPackageJson),
                &["devDependencies"],
            )]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///package.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("package-dev-dependencies.yaml"),
                None,
            ));

            assert_single_dependency(&output, "typescript");
        }

        #[test]
        fn npm_dependency_properties_do_not_disable_pnpm_yaml_defaults() {
            let session = session_with_scoped_property_configs(&[(
                Npm,
                Some(NpmPackageJson),
                &["devDependencies"],
            )]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pnpm-workspace.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("catalog.yaml"),
                None,
            ));

            assert_single_dependency(&output, "react");
        }

        #[test]
        fn pnpm_dependency_properties_do_not_disable_package_json_defaults() {
            let session =
                session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///package.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("dependencies.json"),
                None,
            ));

            assert_single_dependency(&output, "left-pad");
        }

        #[test]
        fn scoped_pnpm_dependency_properties_still_apply_to_pnpm_yaml() {
            let session =
                session_with_scoped_property_configs(&[(Npm, Some(PnpmYaml), &["customCatalog"])]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pnpm-workspace.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("custom-catalog.yaml"),
                None,
            ));

            assert_single_dependency(&output, "scheduler");
        }
    }
    mod pub_manifest {
        use super::{DocumentInput, package_file_fixture, session_with_properties};
        use versionlens_model::Ecosystem::Pub;

        #[test]
        fn dependency_properties_allow_custom_pub_paths() {
            let session = session_with_properties(Pub, &["custom_dependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pubspec.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("custom-dependencies.yaml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "retry");
        }

        #[test]
        fn pub_dependency_properties_allow_member_paths() {
            let session =
                session_with_properties(Pub, &["dependencies.http", "dev_dependencies.*"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pubspec.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("member-paths.yaml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 2);
            assert_eq!(output.dependencies[0].name, "http");
            assert_eq!(output.dependencies[1].name, "test");
        }

        #[test]
        fn pub_dependency_properties_allow_workspace_paths() {
            let session = session_with_properties(Pub, &["workspace"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pubspec.yaml".to_owned(),
                "yaml".to_owned(),
                package_file_fixture("workspace.yaml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].group, "workspace");
            assert_eq!(output.dependencies[0].name, "packages/shared");
        }
    }
    mod python {
        use super::{DocumentInput, package_file_fixture, session_with_properties};
        use versionlens_model::Ecosystem::Python;

        #[test]
        fn dependency_properties_allow_custom_python_toml_paths() {
            let session = session_with_properties(Python, &["tool.uv.sources"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pyproject.toml".to_owned(),
                "toml".to_owned(),
                package_file_fixture("pyproject-uv-sources.toml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "custom");
        }
    }
    mod xml {
        use super::{DocumentInput, package_file_fixture, session_with_properties};
        use versionlens_model::Ecosystem::*;

        #[test]
        fn maven_dependency_properties_filter_before_extraction() {
            let session = session_with_properties(Maven, &["project.parent"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                package_file_fixture("maven-parent-filter.pom.xml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(
                output.dependencies[0].name,
                "org.springframework.boot:spring-boot-starter-parent"
            );
        }

        #[test]
        fn dotnet_project_sdk_matches_dependency_properties() {
            let session = session_with_properties(Dotnet, &["Project.Sdk"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///app.csproj".to_owned(),
                "xml".to_owned(),
                package_file_fixture("dotnet-sdk.csproj"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "Microsoft.NET.Sdk");
        }

        #[test]
        fn dotnet_dependency_properties_filter_before_extraction() {
            let session = session_with_properties(Dotnet, &["Project.ItemGroup.PackageReference"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///app.csproj".to_owned(),
                "xml".to_owned(),
                package_file_fixture("dotnet-package-reference.csproj"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "Newtonsoft.Json");
        }

        #[test]
        fn dotnet_project_json_dependency_properties_match_json_paths() {
            let session = session_with_properties(Dotnet, &["frameworks.*.dependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///project.json".to_owned(),
                "json".to_owned(),
                package_file_fixture("dotnet-project.json"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].name, "System.Text.Json");
            assert_eq!(
                output.dependencies[0].group,
                "frameworks.net472.dependencies"
            );
        }

        #[test]
        fn dotnet_packages_config_dependency_properties_match_package_entries() {
            let session = session_with_properties(Dotnet, &["packages.package"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///packages.config".to_owned(),
                "xml".to_owned(),
                package_file_fixture("packages.config"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].group, "packages.package");
            assert_eq!(output.dependencies[0].name, "jQuery");
        }

        #[test]
        fn dotnet_paket_dependency_properties_match_paket_groups() {
            let session = session_with_properties(Dotnet, &["paket.dependencies"]);

            let output = session.analyze_document(DocumentInput::new(
                "file:///paket.dependencies".to_owned(),
                "plaintext".to_owned(),
                package_file_fixture("paket.dependencies"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 1);
            assert_eq!(output.dependencies[0].group, "paket.dependencies");
            assert_eq!(output.dependencies[0].name, "Newtonsoft.Json");
        }

        #[test]
        fn maven_plugin_dependency_properties_match_plugin_paths() {
            let session = session_with_properties(
                Maven,
                &[
                    "project.build.plugins.plugin",
                    "project.build.pluginManagement.plugins.plugin",
                ],
            );

            let output = session.analyze_document(DocumentInput::new(
                "file:///pom.xml".to_owned(),
                "xml".to_owned(),
                package_file_fixture("maven-plugins.pom.xml"),
                None,
            ));

            assert_eq!(output.dependencies.len(), 2);
            assert_eq!(output.dependencies[0].group, "project.build.plugins.plugin");
            assert_eq!(
                output.dependencies[0].name,
                "org.apache.maven.plugins:maven-compiler-plugin"
            );
            assert_eq!(
                output.dependencies[1].group,
                "project.build.pluginManagement.plugins.plugin"
            );
            assert_eq!(
                output.dependencies[1].name,
                "org.codehaus.mojo:versions-maven-plugin"
            );
        }
    }
}
