use crate::document::test_support::extract_range;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn parses_smoke_dotnet_project_smoke_shapes() {
    let text = package_file_fixture("parses-smoke-dotnet-project-smoke-shapes.txt");
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/project.csproj", "xml");

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
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/default.props", "xml");

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
    let dependencies =
        crate::support::tests::parse_test_document(text, "file:///work/default.targets", "xml");

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

    crate::support::tests::assert_dependency_group(&dependencies, 4, 0, Dotnet, "Project.Sdk");
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
