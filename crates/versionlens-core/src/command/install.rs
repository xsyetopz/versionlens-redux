use versionlens_model::ManifestKind::{DockerComposeYaml, Dockerfile, PnpmYaml};
use versionlens_model::{
    Ecosystem, ManifestKind, ecosystem_config_namespace, ecosystem_for_manifest,
};

pub(crate) fn install_task_config_key(ecosystem: Ecosystem) -> String {
    format!("{}.onSaveChanges", ecosystem_config_namespace(ecosystem))
}

pub(crate) fn install_task_config_key_for_manifest(kind: ManifestKind) -> Option<String> {
    if matches!(kind, DockerComposeYaml | Dockerfile | PnpmYaml) {
        return None;
    }

    ecosystem_for_manifest(kind).map(install_task_config_key)
}

#[cfg(test)]
mod tests;
