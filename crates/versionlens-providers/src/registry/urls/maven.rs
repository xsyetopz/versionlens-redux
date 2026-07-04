use super::trim_end_slash;

pub(super) fn maven_registry_url(name: &str) -> String {
    let (group, artifact) = maven_parts(name);
    format!(
        "https://repo.maven.apache.org/maven2/{}/{artifact}/maven-metadata.xml",
        maven_group_path(group)
    )
}

pub(super) fn maven_registry_url_with_base(base_url: &str, name: &str) -> String {
    let (group, artifact) = maven_parts(name);
    format!(
        "{}/{}/{artifact}/maven-metadata.xml",
        trim_end_slash(base_url),
        maven_group_path(group)
    )
}

fn maven_parts(name: &str) -> (&str, &str) {
    name.split_once(':').unwrap_or((name, "undefined"))
}

fn maven_group_path(group: &str) -> String {
    group.replace('.', "/")
}
