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
