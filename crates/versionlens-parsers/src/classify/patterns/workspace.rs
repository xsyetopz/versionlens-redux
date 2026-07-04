use super::super::uri::file_name;

pub(super) fn is_pnpm_yaml_uri(uri: &str) -> bool {
    matches!(file_name(uri), Some(name) if is_pnpm_yaml_name(name))
}

fn is_pnpm_yaml_name(name: &str) -> bool {
    [
        "pnpm-workspace.yaml",
        "pnpm-workspace.yml",
        ".yarnrc.yml",
        ".yarnrc.yaml",
    ]
    .iter()
    .any(|item| name.eq_ignore_ascii_case(item))
}
