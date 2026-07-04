use crate::model::ManifestKind;

use super::uri::file_name;

pub(super) const EARLY_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("Cargo.toml", ManifestKind::CargoToml),
    ("composer.json", ManifestKind::ComposerJson),
    ("deno.json", ManifestKind::DenoJson),
    ("deno.jsonc", ManifestKind::DenoJson),
    ("project.json", ManifestKind::DotnetProjectJson),
    ("pom.xml", ManifestKind::MavenPomXml),
];

pub(super) const LATE_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("dub.json", ManifestKind::DubJson),
    ("dub.selections.json", ManifestKind::DubJson),
    ("Gemfile", ManifestKind::Gemfile),
    ("go.mod", ManifestKind::GoMod),
    ("package.json", ManifestKind::NpmPackageJson),
];

pub(super) const PUBSPEC_FILE_MANIFESTS: &[(&str, ManifestKind)] = &[
    ("pubspec.yaml", ManifestKind::PubspecYaml),
    ("pubspec.yml", ManifestKind::PubspecYaml),
];

pub(super) fn exact_file_manifest(
    uri: &str,
    manifests: &[(&str, ManifestKind)],
) -> Option<ManifestKind> {
    let name = file_name(uri)?;
    manifests
        .iter()
        .find_map(|(expected, kind)| name.eq_ignore_ascii_case(expected).then_some(*kind))
}
