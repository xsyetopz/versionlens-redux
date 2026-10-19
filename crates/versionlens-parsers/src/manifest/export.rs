//! The manifest table as plain data, for generators of editor file lists.

use super::{MANIFESTS, ManifestFormat, ManifestRow, Rule, SettingsDefault};
use serde::Serialize;

/// The JSON printed by `versionlens-lsp --print-manifest-table`. Rows keep
/// table order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestTable {
    pub manifests: Vec<ManifestTableEntry>,
}

/// One file type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ManifestTableEntry {
    /// The `ManifestKind` variant name.
    pub kind: String,
    /// The provider family and settings namespace, such as `golang` in
    /// `versionlens.golang.*`.
    pub provider: String,
    /// `json`, `toml`, `yaml`, `xml` or `custom`.
    pub format: String,
    /// Exact file names, in pattern order.
    pub names: Vec<String>,
    /// File extensions without the dot, in pattern order.
    pub extensions: Vec<String>,
    /// Globs for patterns that a name or an extension cannot express.
    pub globs: Vec<String>,
    /// Editor language IDs that host the file type.
    pub languages: Vec<String>,
    /// Exact file names for editors that filter by name and extension.
    pub editor_names: Vec<String>,
    /// The globs that `versionlens.<provider>.files` lists for the file type.
    pub files: Vec<String>,
    /// What the file type adds to `versionlens.<provider>.dependencyProperties`.
    pub settings_properties: Vec<String>,
}

pub fn manifest_table() -> ManifestTable {
    ManifestTable {
        manifests: MANIFESTS.iter().map(entry).collect(),
    }
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

fn entry(row: &ManifestRow) -> ManifestTableEntry {
    let mut names = Vec::new();
    let mut extensions = Vec::new();
    let mut languages = strings(row.languages);
    for pattern in row.patterns {
        let (list, value) = match pattern.rule {
            Rule::Name(name) => (&mut names, name),
            Rule::Extension(extension) => (&mut extensions, extension),
            Rule::Language(language) => (&mut languages, language),
            Rule::Uri(_) | Rule::Content(_) => continue,
        };
        if !list.iter().any(|item| item == value) {
            list.push(value.to_owned());
        }
    }
    for language in default_languages(row.format) {
        if !languages.iter().any(|item| item == language) {
            languages.push((*language).to_owned());
        }
    }
    let mut files = strings(row.file_globs);
    if files.is_empty() {
        files.extend(names.iter().cloned());
        files.extend(extensions.iter().map(|extension| format!("*.{extension}")));
        for glob in row.globs {
            if !files.iter().any(|item| item == glob) {
                files.push((*glob).to_owned());
            }
        }
    }
    let settings_properties = match row.settings {
        SettingsDefault::Hidden => Vec::new(),
        SettingsDefault::Paths => strings(row.dependency_paths),
        SettingsDefault::Listed(entries) => strings(entries),
    };
    ManifestTableEntry {
        kind: format!("{:?}", row.kind),
        provider: row.provider.to_owned(),
        format: format_name(row.format).to_owned(),
        names,
        extensions,
        globs: strings(row.globs),
        languages,
        editor_names: strings(row.editor_names),
        files,
        settings_properties,
    }
}

fn format_name(format: ManifestFormat) -> &'static str {
    match format {
        ManifestFormat::Json => "json",
        ManifestFormat::Toml => "toml",
        ManifestFormat::Yaml => "yaml",
        ManifestFormat::Xml => "xml",
        ManifestFormat::Custom => "custom",
    }
}

/// The editor languages that a data format implies.
fn default_languages(format: ManifestFormat) -> &'static [&'static str] {
    match format {
        ManifestFormat::Json => &["json", "jsonc"],
        ManifestFormat::Toml => &["toml"],
        ManifestFormat::Yaml => &["yaml"],
        ManifestFormat::Xml => &["xml"],
        ManifestFormat::Custom => &[],
    }
}

#[cfg(test)]
mod tests {
    use super::manifest_table;

    #[test]
    fn table_lists_every_row_with_its_names() {
        let table = manifest_table();
        assert_eq!(table.manifests.len(), super::MANIFESTS.len());
        let cargo = table
            .manifests
            .iter()
            .find(|entry| entry.kind == "CargoToml")
            .expect("Cargo.toml row");
        assert_eq!(cargo.provider, "cargo");
        assert_eq!(cargo.names, ["Cargo.toml"]);
        assert!(cargo.settings_properties.contains(&"patch.*".to_owned()));
        assert_eq!(cargo.files, ["Cargo.toml"]);
    }

    #[test]
    fn table_serializes_with_camel_case_keys() {
        let json = serde_json::to_value(manifest_table()).expect("serializes");
        assert!(json["manifests"][0]["settingsProperties"].is_array());
    }
}
