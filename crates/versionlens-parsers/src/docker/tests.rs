use crate::docker::image::split_image_reference;
use crate::document::test_support::{extract_range, parse_fixture};
use crate::{DocumentInput, parse_document};
use versionlens_model::Ecosystem::Docker;

#[test]
fn dockerfile_image_reference_separates_explicit_registry() {
    let image = split_image_reference("ghcr.io/org/app:1.2.3");

    assert_eq!(image.registry, "ghcr.io");
    assert_eq!(image.name, "org/app");
    assert_eq!(image.tag, "1.2.3");
}

#[test]
fn parses_dockerfile_from_dependencies() {
    let text = package_file_fixture("parses-dockerfile-from-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/Dockerfile", "dockerfile");

    assert_eq!(dependencies.len(), 5);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(0, Docker, "FROM", "node", "20"),
    );
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "20");
    assert_eq!(dependencies[1].name, "dotnet/sdk");
    assert_eq!(
        dependencies[1].hosted_url.as_deref(),
        Some("mcr.microsoft.com")
    );
    assert_eq!(dependencies[1].requirement, "8.0");
    assert_eq!(dependencies[2].name, "org/app");
    assert_eq!(dependencies[2].hosted_url.as_deref(), Some("ghcr.io"));
    assert_eq!(dependencies[2].requirement, "1.2.3");
    assert_eq!(extract_range(text, dependencies[2].range), "org/app");
    assert_eq!(dependencies[3].name, "alpine");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, ":");
    assert_eq!(dependencies[4].name, "ubuntu");
    assert_eq!(dependencies[4].requirement, "sha256:abc123");
    assert_eq!(dependencies[4].requirement_prefix, "@");
    assert_eq!(
        extract_range(text, dependencies[4].requirement_range),
        "sha256:abc123"
    );
}

#[test]
fn dockerfile_ranges_count_utf16_code_units_before_dependencies() {
    let text =
        package_file_fixture("dockerfile-ranges-count-utf16-code-units-before-dependencies.txt");
    let dependencies = parse_fixture(text, "file:///work/Dockerfile", "dockerfile");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "node");
    assert_eq!(dependencies[0].range.start.character, 25);
    assert_eq!(extract_range(text, dependencies[0].range), "node");
}

#[test]
fn parses_docker_compose_image_dependencies() {
    let text = package_file_fixture("parses-docker-compose-image-dependenciesDockerfile");
    let dependencies = parse_fixture(text, "file:///work/docker-compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 12);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Docker,
            "services.image",
            "node",
            "20",
        ),
    );
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "20");
    assert_eq!(dependencies[1].name, "org/app");
    assert_eq!(dependencies[1].hosted_url.as_deref(), Some("ghcr.io"));
    assert_eq!(dependencies[1].requirement, "1.2.3");
    assert_eq!(extract_range(text, dependencies[1].range), "org/app");
    assert_eq!(dependencies[2].name, "dotnet/runtime");
    assert_eq!(
        dependencies[2].hosted_url.as_deref(),
        Some("mcr.microsoft.com")
    );
    assert_eq!(dependencies[2].requirement, "9.0");
    assert_eq!(extract_range(text, dependencies[2].range), "dotnet/runtime");
    assert_eq!(dependencies[3].name, "postgres");
    assert_eq!(dependencies[3].requirement, "");
    assert_eq!(dependencies[3].requirement_prefix, ":");
    assert_eq!(dependencies[4].name, "ubuntu");
    assert_eq!(dependencies[4].requirement, "sha256:def456");
    assert_eq!(dependencies[4].requirement_prefix, "@");
    assert_eq!(extract_range(text, dependencies[4].range), "ubuntu");
    assert_eq!(
        extract_range(text, dependencies[4].requirement_range),
        "sha256:def456"
    );
    assert_eq!(dependencies[5].name, "123456");
    assert_eq!(dependencies[5].requirement, "");
    assert_eq!(dependencies[5].requirement_prefix, ":");
    assert_eq!(dependencies[6].group, "services.build");
    assert_eq!(dependencies[6].name, "./dockerfile");
    assert_eq!(dependencies[6].requirement, "./dockerfile");
    assert_eq!(dependencies[7].name, "./ctx/dockerfile");
    assert_eq!(dependencies[8].name, "./custom.dockerfile");
    assert_eq!(dependencies[9].name, "example/app");
    assert_eq!(dependencies[9].hosted_url, None);
    assert_eq!(dependencies[9].requirement, "1.0");
    assert_eq!(extract_range(text, dependencies[9].range), "example/app");
    assert_eq!(dependencies[10].name, "backend/dockerfile");
    assert_eq!(dependencies[10].requirement, "backend/dockerfile");
    assert_eq!(dependencies[11].name, "service/dockerfile");
    assert_eq!(dependencies[11].requirement, "service/dockerfile");
}

#[test]
fn parses_docker_compose_namespace_images_without_treating_namespace_as_registry() {
    let text = package_file_fixture(
        "parses-docker-compose-namespace-images-without-treating-namespace-as-registry.txt",
    );
    let dependencies = parse_fixture(text, "file:///work/compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "services.image");
    assert_eq!(dependencies[0].name, "library/nginx");
    assert_eq!(dependencies[0].hosted_url, None);
    assert_eq!(dependencies[0].requirement, "1.25");
    assert_eq!(extract_range(text, dependencies[0].range), "library/nginx");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "1.25"
    );
}

#[test]
fn parses_docker_compose_bare_build_context_without_prefix() {
    let text = package_file_fixture("parses-docker-compose-bare-build-context-without-prefix.yaml");
    let dependencies = parse_fixture(text, "file:///work/docker-compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "services.build");
    assert_eq!(dependencies[0].name, "backend/dockerfile");
    assert_eq!(dependencies[0].requirement, "backend/dockerfile");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "backend"
    );
}

#[test]
fn parses_docker_compose_top_level_extension_image_dependencies() {
    let text =
        package_file_fixture("parses-docker-compose-top-level-extension-image-dependencies.yaml");
    let dependencies = parse_fixture(text, "file:///work/docker-compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 2);
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            0,
            Docker,
            "services.image",
            "node",
            "20",
        ),
    );
    crate::support::tests::assert_dependency(
        &dependencies,
        crate::support::tests::DependencyExpectation::new(
            1,
            Docker,
            "services.image",
            "busybox",
            "1.36",
        ),
    );
    assert_eq!(extract_range(text, dependencies[1].range), "busybox");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "1.36"
    );
}

#[test]
fn parses_docker_compose_build_context_slashes_without_normalizing() {
    let text =
        package_file_fixture("parses-docker-compose-build-context-slashes-without-normalizing.txt");
    let dependencies = parse_fixture(text, "file:///work/docker-compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "backend//dockerfile");
    assert_eq!(dependencies[0].requirement, "backend//dockerfile");
    assert_eq!(
        extract_range(text, dependencies[0].requirement_range),
        "backend/"
    );
    assert_eq!(dependencies[1].name, "./ctx//Dockerfile.prod");
    assert_eq!(dependencies[1].requirement, "./ctx//Dockerfile.prod");
    assert_eq!(
        extract_range(text, dependencies[1].requirement_range),
        "./ctx/"
    );
}

#[test]
fn parses_docker_compose_empty_string_build_context_like_upstream() {
    let text =
        package_file_fixture("parses-docker-compose-empty-string-build-context-like-upstream.txt");
    let dependencies = parse_fixture(text, "file:///work/docker-compose.yaml", "yaml");

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].group, "services.build");
    assert_eq!(dependencies[0].name, "/dockerfile");
    assert_eq!(dependencies[0].requirement, "/dockerfile");
    assert_eq!(extract_range(text, dependencies[0].requirement_range), "");
}

#[test]
fn parses_smoke_docker_smoke_shapes() {
    let dockerfile = "\
FROM mcr.microsoft.com/dotnet/sdk

FROM node:20-alpine
";
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/dockerfile".to_owned(),
        "dockerfile".to_owned(),
        dockerfile.to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].name, "dotnet/sdk");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("mcr.microsoft.com")
    );
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[1].name, "node");
    assert_eq!(dependencies[1].requirement, "20-alpine");

    let compose = "\
services:
  web:
    image: nginx
  backend:
    build:
      context: ./build-folder
      dockerfile: custom.dockerfile
  mongo:
    image: mongo
";
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/compose.yaml".to_owned(),
        "yaml".to_owned(),
        compose.to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "nginx");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[1].group, "services.build");
    assert_eq!(dependencies[1].name, "./build-folder/custom.dockerfile");
    assert_eq!(dependencies[2].name, "mongo");

    let custom_dockerfile = "FROM mcr.microsoft.com/dotnet/sdk:7.0";
    let dependencies = parse_document(&DocumentInput::new(
        "file:///work/build-folder/custom.dockerfile".to_owned(),
        "dockerfile".to_owned(),
        custom_dockerfile.to_owned(),
        None,
    ));

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "dotnet/sdk");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("mcr.microsoft.com")
    );
    assert_eq!(dependencies[0].requirement, "7.0");
}

fn package_file_fixture(name: &str) -> &'static str {
    crate::support::tests::fixture("tests/fixtures/versionlens-parsers/src/docker/tests", name)
}
