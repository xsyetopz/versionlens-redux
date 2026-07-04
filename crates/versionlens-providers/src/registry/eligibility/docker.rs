pub(super) fn is_supported_docker_dependency(name: &str) -> bool {
    !name.contains('$')
}
