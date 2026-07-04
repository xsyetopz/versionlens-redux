use super::super::uri::{file_name, has_extension};

pub(in crate::classify) fn is_dockerfile_uri(uri: &str) -> bool {
    let Some(name) = file_name(uri) else {
        return false;
    };
    name.eq_ignore_ascii_case("Dockerfile") || ends_with_ignore_ascii_case(name, ".dockerfile")
}

pub(super) fn is_docker_compose_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if is_docker_compose_name(name))
}

fn is_docker_compose_name(name: &str) -> bool {
    [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yaml",
        "docker-compose.yml",
    ]
    .iter()
    .any(|item| name.eq_ignore_ascii_case(item))
        || is_docker_compose_variant_name(name)
        || is_dot_compose_yaml_name(name)
}

fn is_docker_compose_variant_name(name: &str) -> bool {
    (starts_with_ignore_ascii_case(name, "docker-compose.")
        || starts_with_ignore_ascii_case(name, "compose."))
        && has_extension(name, ["yaml", "yml"])
}

fn is_dot_compose_yaml_name(name: &str) -> bool {
    has_extension(name, ["yaml", "yml"])
        && name
            .rsplit_once('.')
            .is_some_and(|(stem, _)| ends_with_ignore_ascii_case(stem, ".compose"))
}

fn starts_with_ignore_ascii_case(value: &str, prefix: &str) -> bool {
    value
        .get(..prefix.len())
        .is_some_and(|start| start.eq_ignore_ascii_case(prefix))
}

fn ends_with_ignore_ascii_case(value: &str, suffix: &str) -> bool {
    value
        .get(value.len().saturating_sub(suffix.len())..)
        .is_some_and(|end| end.eq_ignore_ascii_case(suffix))
}
