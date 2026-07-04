use crate::docker::image::split_image_reference;
use crate::document::test_support::extract_range;
use crate::{DocumentInput, Ecosystem, parse_document};

#[test]
fn dockerfile_image_reference_separates_explicit_registry() {
    let image = split_image_reference("ghcr.io/org/app:1.2.3");

    assert_eq!(image.registry, "ghcr.io");
    assert_eq!(image.name, "org/app");
    assert_eq!(image.tag, "1.2.3");
}

#[test]
fn parses_dockerfile_from_dependencies() {
    let text = "\
# FROM skipped:1
FROM node:20
FROM mcr.microsoft.com/dotnet/sdk:8.0
FROM ghcr.io/org/app:1.2.3
FROM --platform=linux/amd64 alpine AS base
FROM ubuntu@sha256:abc123
FROMAGE:latest
FROMnode:20
FROM :20
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/Dockerfile".to_owned(),
        language_id: "dockerfile".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 5);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Docker);
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
    assert_eq!(dependencies[4].requirement, "");
    assert_eq!(dependencies[4].requirement_prefix, ":");
    assert_eq!(extract_range(text, dependencies[4].requirement_range), "");
}

#[test]
fn dockerfile_ranges_count_utf16_code_units_before_dependencies() {
    let text = "FROM --platform=linux/😀 node:20\n";
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
    let text = "\
services:
  web:
    image: node:20
  api:
    image: ghcr.io/org/app:1.2.3
  mcr:
    image: mcr.microsoft.com/dotnet/runtime:9.0
  data:
    image: postgres
  digest:
    image: ubuntu@sha256:def456
  image-number:
    image: 123456
  worker:
    build: .
  context-default:
    build:
      context: ./ctx
  custom:
    build:
      context: .
      dockerfile: custom.dockerfile
  built-image:
    image: example/app:1.0
    build:
      context: ./app
      dockerfile: Dockerfile.prod
  bare-build:
    build: backend
  bare-context:
    build:
      context: service
  empty:
    image: ''
  malformed:
    image: :20
";
    let dependencies = parse_document(&DocumentInput {
        uri: "file:///work/docker-compose.yaml".to_owned(),
        language_id: "yaml".to_owned(),
        text: text.to_owned(),
        workspace_root: None,
    });

    assert_eq!(dependencies.len(), 12);
    assert_eq!(dependencies[0].ecosystem, Ecosystem::Docker);
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
    assert_eq!(dependencies[4].name, "ubuntu@sha256");
    assert_eq!(dependencies[4].requirement, "def456");
    assert_eq!(extract_range(text, dependencies[4].range), "ubuntu@sha256");
    assert_eq!(
        extract_range(text, dependencies[4].requirement_range),
        "def456"
    );
    assert_eq!(dependencies[5].name, "123456");
    assert_eq!(dependencies[5].requirement, "");
    assert_eq!(dependencies[5].requirement_prefix, ":");
    assert_eq!(dependencies[6].group, "services.build");
    assert_eq!(dependencies[6].name, "./dockerfile");
    assert_eq!(dependencies[6].requirement, "./dockerfile");
    assert_eq!(dependencies[7].name, "./ctx/dockerfile");
    assert_eq!(dependencies[8].name, "./custom.dockerfile");
    assert_eq!(dependencies[9].name, "app");
    assert_eq!(dependencies[9].hosted_url.as_deref(), Some("example"));
    assert_eq!(dependencies[9].requirement, "1.0");
    assert_eq!(extract_range(text, dependencies[9].range), "app");
    assert_eq!(dependencies[10].name, "backend/dockerfile");
    assert_eq!(dependencies[10].requirement, "backend/dockerfile");
    assert_eq!(dependencies[11].name, "service/dockerfile");
    assert_eq!(dependencies[11].requirement, "service/dockerfile");
}

#[test]
fn parses_docker_compose_bare_build_context_without_prefix() {
    let text = "\
services:
  worker:
    build: backend
";
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
fn parses_docker_compose_build_context_slashes_without_normalizing() {
    let text = "\
services:
  worker:
    build: backend/
  mapped:
    build:
      context: ./ctx/
      dockerfile: Dockerfile.prod
";
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
    let text = "\
services:
  worker:
    build: \"\"
";
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
