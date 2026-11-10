use quick_xml::events::BytesStart;
use std::str;
use versionlens_model::Dependency;

mod attributes;
mod collect;
mod dependency;

use collect::collect_dotnet_xml_dependencies;

pub(crate) fn parse_dotnet_xml_with_paths(
    text: &str,
    dependency_paths: &[&str],
) -> Vec<Dependency> {
    collect_dotnet_xml_dependencies(text, dependency_paths.to_vec())
}

pub(super) struct DotnetEventContext<'a> {
    pub(super) text: &'a str,
    pub(super) span: DotnetTagSpan,
}

pub(super) struct DotnetTagSpan {
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DotnetTagKind {
    Empty,
    Start,
}

pub(super) struct OpenProjectVersion {
    pub(super) text_start: usize,
    pub(super) value: String,
}

pub(super) fn event_name(event: &BytesStart<'_>) -> Option<String> {
    event_name_from_bytes(event.name().as_ref().as_bytes())
}

pub(super) fn event_name_from_bytes(bytes: &[u8]) -> Option<String> {
    str::from_utf8(bytes).ok().map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    mod parser {
        use crate::document::test_support::extract_range;
        use crate::{DocumentInput, parse_document};
        use versionlens_model::Ecosystem::Dotnet;

        #[test]
        fn parses_dotnet_xml_dependencies() {
            let text = package_file_fixture("parses-dotnet-xml-dependencies.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            assert_eq!(dependencies.len(), 8);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            assert_eq!(dependencies[0].group, "Project.Sdk");
            assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
            assert_eq!(dependencies[0].requirement, "8.0.100");
            assert_eq!(dependencies[1].group, "PropertyGroup");
            assert_eq!(dependencies[1].name, "Version");
            assert_eq!(dependencies[1].requirement, "1.2.3");
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "1.2.3"
            );
            assert_eq!(dependencies[2].group, "PropertyGroup");
            assert_eq!(dependencies[2].name, "AssemblyVersion");
            assert_eq!(dependencies[2].requirement, "");
            assert_eq!(extract_range(text, dependencies[2].requirement_range), "");
            assert_eq!(dependencies[3].group, "PackageReference");
            assert_eq!(dependencies[3].name, "Newtonsoft.Json");
            assert_eq!(dependencies[3].requirement, "13.0.3");
            assert_eq!(
                extract_range(text, dependencies[3].range),
                r#"<PackageReference Include="Newtonsoft.Json" Version="13.0.3" />"#
            );
            assert_eq!(
                extract_range(text, dependencies[3].requirement_range),
                "13.0.3"
            );
            assert_eq!(dependencies[4].name, "Serilog");
            assert_eq!(dependencies[4].requirement, "3.1.0");
            assert_eq!(dependencies[5].name, "NoVersionAttribute");
            assert_eq!(dependencies[5].requirement, "*");
            assert_eq!(dependencies[5].requirement_prefix, " Version=\"");
            assert_eq!(dependencies[5].requirement_suffix, "\"");
            assert_eq!(extract_range(text, dependencies[5].requirement_range), "");
            assert_eq!(dependencies[6].group, "PackageVersion");
            assert_eq!(dependencies[6].name, "Microsoft.Extensions.Logging");
            assert_eq!(dependencies[7].group, "DotNetCliToolReference");
            assert_eq!(dependencies[7].name, "dotnet-ef");
            assert_eq!(dependencies[7].requirement, "8.0.1");
        }

        #[test]
        fn parses_packages_config_dependencies() {
            let text = package_file_fixture("parses-packages-config-dependencies.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/packages.config",
                "xml",
            );

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            assert_eq!(dependencies[0].group, "packages.package");
            assert_eq!(dependencies[0].name, "jQuery");
            assert_eq!(dependencies[0].requirement, "3.1.1");
            assert_eq!(extract_range(text, dependencies[0].range), "jQuery");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "3.1.1"
            );
            assert_eq!(dependencies[1].group, "packages.package");
            assert_eq!(dependencies[1].name, "NLog");
            assert_eq!(dependencies[1].requirement, "4.3.10");
        }

        #[test]
        fn parses_dotnet_xml_non_empty_versionless_package_reference() {
            let text = package_file_fixture(
                "parses-dotnet-xml-non-empty-versionless-package-reference.txt",
            );
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            assert_eq!(dependencies.len(), 1);
            crate::support::tests::assert_named_dependency(
                &dependencies,
                0,
                "ChildVersionNoAttribute",
                "*",
            );
            assert_eq!(dependencies[0].requirement_prefix, " Version=\"");
            assert_eq!(dependencies[0].requirement_suffix, "\"");
            assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
        }

        #[test]
        fn parses_package_reference_child_version() {
            let text = package_file_fixture("parses-package-reference-child-version.csproj");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].name, "ChildVersionNoAttribute");
            assert_eq!(dependencies[0].requirement, "18.7.0");
            assert_eq!(dependencies[0].requirement_prefix, "");
            assert_eq!(dependencies[0].requirement_suffix, "");
            assert_eq!(
                extract_range(text, dependencies[0].range),
                "ChildVersionNoAttribute"
            );
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "18.7.0"
            );
        }

        #[test]
        fn dotnet_invalid_xml_returns_no_dependencies() {
            let dependencies = parse_document(&DocumentInput::new(
                "file:///work/app.csproj".to_owned(),
                "xml".to_owned(),
                package_file_fixture("dotnet-invalid-xml-returns-no-dependencies.csproj")
                    .to_owned(),
                None,
            ));

            assert_eq!(dependencies.len(), 0);
        }

        #[test]
        fn dotnet_dependency_order_follows_dependency_properties() {
            let text = package_file_fixture(
                "dotnet-dependency-order-follows-dependency-properties.csproj",
            );
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            let names = dependencies
                .iter()
                .map(|dependency| dependency.name.as_str())
                .collect::<Vec<_>>();

            assert_eq!(
                names,
                [
                    "Microsoft.Build.CentralPackageVersions",
                    "Global.Package",
                    "Newtonsoft.Json",
                    "Central.Package",
                    "dotnet-ef"
                ]
            );
        }

        #[test]
        fn dotnet_project_sdk_attribute_is_parsed_like_upstream() {
            let text =
                package_file_fixture("dotnet-project-sdk-attribute-is-parsed-like-upstream.csproj");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].group, "Project.Sdk");
            assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
            assert_eq!(dependencies[0].requirement, "8.0.100");
            assert_eq!(
                extract_range(text, dependencies[0].range),
                "Microsoft.NET.Sdk"
            );
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "8.0.100"
            );
        }

        #[test]
        fn parses_dotnet_xml_attributes_case_insensitively() {
            let text =
                package_file_fixture("parses-dotnet-xml-attributes-case-insensitively.csproj");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/app.csproj", "xml");

            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            assert_eq!(dependencies[0].name, "Newtonsoft.Json");
            assert_eq!(dependencies[0].requirement, "13.0.3");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "13.0.3"
            );
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/current",
                name,
            )
        }
    }
    mod project_smoke {
        use crate::document::test_support::extract_range;
        use versionlens_model::Ecosystem::Dotnet;

        #[test]
        fn parses_smoke_dotnet_project_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-project-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.csproj",
                "xml",
            );

            assert_eq!(dependencies.len(), 12);
            crate::support::tests::assert_dependency(
                &dependencies,
                crate::support::tests::DependencyExpectation::new(
                    0,
                    Dotnet,
                    "Project.Sdk",
                    "Microsoft.NET.Sdk",
                    "*",
                ),
            );
            assert_eq!(dependencies[1].name, "Version");
            assert_eq!(dependencies[1].requirement, "1.2.3");
            assert_eq!(dependencies[2].name, "AssemblyVersion");
            assert_eq!(dependencies[2].requirement, "");
            assert_eq!(dependencies[3].name, "jQuery");
            assert_eq!(dependencies[3].requirement, "3.7");
            assert_eq!(dependencies[4].requirement, "(5.0,)");
            assert_eq!(dependencies[5].requirement, "[2.22]");
            assert_eq!(dependencies[6].requirement, "(,10.9]");
            assert_eq!(dependencies[7].requirement, "[12,13)");
            assert_eq!(dependencies[9].requirement, "1.*");
            assert_eq!(dependencies[10].name, "AngularJS.Core");
            assert_eq!(dependencies[10].requirement, "1.*");
            assert_eq!(dependencies[11].requirement, "1.0.112.2");
        }

        #[test]
        fn parses_smoke_dotnet_props_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-props-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/default.props",
                "xml",
            );

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            assert_eq!(dependencies[0].group, "PackageReference");
            assert_eq!(dependencies[0].name, "Microsoft.NET.Test.Sdk");
            assert_eq!(dependencies[0].requirement, "15.6.2");
            assert_eq!(dependencies[2].name, "MSTest.TestFramework");
        }

        #[test]
        fn parses_smoke_dotnet_targets_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-targets-smoke-shapes.props");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/default.targets",
                "xml",
            );

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].group, "PackageReference");
            assert_eq!(
                dependencies[0].name,
                "Microsoft.Extensions.DependencyInjection.Abstractions"
            );
            assert_eq!(dependencies[0].requirement, "10.0.9");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "10.0.9"
            );
            assert_eq!(
                dependencies[1].name,
                "Microsoft.Extensions.Logging.Abstractions"
            );
        }

        #[test]
        fn parses_smoke_dotnet_versionless_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-versionless-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.no-version.csproj",
                "xml",
            );

            crate::support::tests::assert_dependency_group(
                &dependencies,
                4,
                0,
                Dotnet,
                "Project.Sdk",
            );
            crate::support::tests::assert_dependency_metadata(
                &dependencies,
                0,
                "Project.Sdk",
                "Microsoft.NET.Sdk",
                "*",
            );
            crate::support::tests::assert_dependency_metadata(
                &dependencies,
                1,
                "PackageReference",
                "jQuery",
                "*",
            );
            assert_eq!(dependencies[2].name, "Nerdbank.GitVersioning");
            assert_eq!(dependencies[2].requirement, "*");
            assert_eq!(dependencies[3].name, "Microsoft.NET.Test.Sdk");
            assert_eq!(dependencies[3].requirement, "18.7.0");
            assert_eq!(
                extract_range(text, dependencies[3].requirement_range),
                "18.7.0"
            );
        }

        #[test]
        fn parses_smoke_dotnet_auth_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-auth-smoke-shapes.txt");
            let dependencies =
                crate::support::tests::parse_test_document(text, "file:///work/auth.csproj", "xml");

            assert_eq!(dependencies.len(), 2);
            crate::support::tests::assert_dependency(
                &dependencies,
                crate::support::tests::DependencyExpectation::new(
                    0,
                    Dotnet,
                    "Project.Sdk",
                    "Microsoft.NET.Sdk",
                    "*",
                ),
            );
            crate::support::tests::assert_dependency(
                &dependencies,
                crate::support::tests::DependencyExpectation::new(
                    1,
                    Dotnet,
                    "PackageReference",
                    "Private.VersionLens.Package",
                    "*",
                ),
            );
            assert_eq!(dependencies[1].requirement_prefix, " Version=\"");
            assert_eq!(dependencies[1].requirement_suffix, "\"");
            assert_eq!(extract_range(text, dependencies[1].requirement_range), "");
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/project_smoke",
                name,
            )
        }
    }
    mod sdk_smoke {
        use crate::document::test_support::extract_range;
        use versionlens_model::Ecosystem::Dotnet;

        #[test]
        fn parses_smoke_dotnet_fsproj_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-fsproj-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.fsproj",
                "xml",
            );

            assert_eq!(dependencies.len(), 4);
            assert_eq!(dependencies[0].group, "Project.Sdk");
            assert_eq!(dependencies[0].name, "FSharp.NET.Sdk");
            assert_eq!(dependencies[0].requirement, "*");
            assert_eq!(dependencies[1].group, "Project.Sdk");
            assert_eq!(dependencies[1].name, "Microsoft.NET.Sdk");
            assert_eq!(dependencies[1].requirement, "*");
            assert_eq!(dependencies[2].name, "FSharp.Core");
            assert_eq!(dependencies[2].requirement, "4.1.2");
            assert_eq!(dependencies[3].name, "FSharp.Net.Sdk");
            assert_eq!(dependencies[3].requirement, "1.0.1");
        }

        #[test]
        fn parses_smoke_dotnet_override_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-override-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.override.csproj",
                "xml",
            );

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].group, "Project.Sdk");
            assert_eq!(dependencies[0].name, "Microsoft.NET.Sdk");
            assert_eq!(dependencies[0].requirement, "*");
            assert_eq!(dependencies[1].group, "PackageReference");
            assert_eq!(dependencies[1].name, "jQuery");
            assert_eq!(dependencies[1].requirement, "3.7.*");
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "3.7.*"
            );
        }

        #[test]
        fn parses_smoke_dotnet_central_package_props_smoke_shapes() {
            let text =
                package_file_fixture("parses-smoke-dotnet-central-package-props-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/Directory.Packages.props",
                "xml",
            );

            assert_eq!(dependencies.len(), 3);
            assert_eq!(dependencies[0].group, "Sdk");
            assert_eq!(
                dependencies[0].name,
                "Microsoft.Build.CentralPackageVersions"
            );
            assert_eq!(dependencies[0].requirement, "2.1.3");
            assert_eq!(
                extract_range(text, dependencies[0].requirement_range),
                "2.1.3"
            );
            assert_eq!(dependencies[1].group, "GlobalPackageReference");
            assert_eq!(dependencies[1].name, "Microsoft.Azure.ServiceBus");
            assert_eq!(dependencies[1].requirement, "(3.0,)");
            assert_eq!(dependencies[2].group, "PackageVersion");
            assert_eq!(dependencies[2].name, "System.Text.Json");
            assert_eq!(dependencies[2].requirement, "4.7.2");
        }

        #[test]
        fn parses_smoke_dotnet_bom_smoke_shapes() {
            let text = package_file_fixture("parses-smoke-dotnet-bom-smoke-shapes.txt");
            let dependencies = crate::support::tests::parse_test_document(
                text,
                "file:///work/project.bom.csproj",
                "xml",
            );

            assert_eq!(dependencies.len(), 2);
            assert_eq!(dependencies[0].ecosystem, Dotnet);
            crate::support::tests::assert_dependency_metadata(
                &dependencies,
                0,
                "Project.Sdk",
                "Microsoft.NET.Sdk",
                "*",
            );
            crate::support::tests::assert_dependency_metadata(
                &dependencies,
                1,
                "PackageReference",
                "jQuery",
                "3.7.1",
            );
            assert_eq!(
                extract_range(text, dependencies[1].requirement_range),
                "3.7.1"
            );
        }

        fn package_file_fixture(name: &str) -> &'static str {
            crate::support::tests::fixture(
                "tests/fixtures/versionlens-parsers/src/dotnet_xml/tests/sdk_smoke",
                name,
            )
        }
    }
}
