use crate::docker::image::split_image_reference;
use crate::document::test_support::extract_range;
use crate::{DocumentInput, parse_document};
use std::fs::read_to_string;
use std::path::PathBuf;
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Docker);
    assert_eq!(dependencies[0].group, "FROM");
    assert_eq!(dependencies[0].name, "node");
    assert_eq!(dependencies[0].requirement, "20");
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "node");
    assert_eq!(dependencies[0].range.start.character, 25);
    assert_eq!(extract_range(text, dependencies[0].range), "node");
}

#[test]
fn parses_docker_compose_image_dependencies() {
    let text = package_file_fixture("parses-docker-compose-image-dependenciesDockerfile");
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Docker);
    assert_eq!(dependencies[0].group, "services.image");
    assert_eq!(dependencies[0].name, "node");
    assert_eq!(dependencies[0].requirement, "20");
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 2);
    assert_eq!(dependencies[0].ecosystem, Docker);
    assert_eq!(dependencies[0].group, "services.image");
    assert_eq!(dependencies[0].name, "node");
    assert_eq!(dependencies[0].requirement, "20");
    assert_eq!(dependencies[1].ecosystem, Docker);
    assert_eq!(dependencies[1].group, "services.image");
    assert_eq!(dependencies[1].name, "busybox");
    assert_eq!(dependencies[1].requirement, "1.36");
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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: dockerfile.to_owned(),
        workspace_root: None,
    });

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
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: compose.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 3);
    assert_eq!(dependencies[0].name, "nginx");
    assert_eq!(dependencies[0].requirement, "");
    assert_eq!(dependencies[1].group, "services.build");
    assert_eq!(dependencies[1].name, "./build-folder/custom.dockerfile");
    assert_eq!(dependencies[2].name, "mongo");

    let custom_dockerfile = "FROM mcr.microsoft.com/dotnet/sdk:7.0";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/build-folder/custom.dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: custom_dockerfile.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].name, "dotnet/sdk");
    assert_eq!(
        dependencies[0].hosted_url.as_deref(),
        Some("mcr.microsoft.com")
    );
    assert_eq!(dependencies[0].requirement, "7.0");
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-parsers/src/docker/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    <PathBuf as From<&str>>::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
