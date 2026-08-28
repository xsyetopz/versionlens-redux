use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_smoke_dotnet_fsproj_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-fsproj-smoke-shapes.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/project.fsproj", "xml");

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
    let text = package_file_fixture("parses-smoke-dotnet-central-package-props-smoke-shapes.txt");
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
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/project.bom.csproj", "xml");

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
