use super::*;
use crate::registry::RegistryFileSnapshot;
use versionlens_parsers::classify_document;

impl RegistryContext {
    pub(crate) fn from_document(input: &DocumentInput) -> Self {
        let kind = classify_document(input);
        let snapshot = RegistryFileSnapshot::default();
        let reader = snapshot.reader(input);
        registry_context_from_document_kind_with_files(input, kind, &reader)
    }
}

#[test]
fn shared_context_propagates_registry_file_bounds() {
    let root = versionlens_test_support::temporary_directory("registry-context-bound").unwrap();
    let config = root.join(".npmrc");
    let file = std::fs::File::create(&config).unwrap();
    file.set_len(1024 * 1024 + 1).unwrap();
    let input = DocumentInput::new(
        root.join("package.json").to_string_lossy(),
        "json",
        r#"{"dependencies":{"demo":"1.0.0"}}"#,
        None,
    );

    let snapshot = RegistryFileSnapshot::default();
    let reader = snapshot.reader(&input);
    let context = registry_context_from_document_kind_with_files(
        &input,
        ManifestKind::NpmPackageJson,
        &reader,
    );
    assert!(
        context
            .failure_message()
            .is_some_and(|message| message.contains("the limit is"))
    );
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn configured_npm_user_file_cannot_fall_back_when_missing() {
    let root =
        versionlens_test_support::temporary_directory("registry-required-userconfig").unwrap();
    let required = root.join("required.npmrc");
    let input = DocumentInput::new(
        root.join("package.json").to_string_lossy(),
        "json",
        r#"{"dependencies":{"demo":"1.0.0"}}"#,
        None,
    );
    let snapshot = RegistryFileSnapshot::default();
    let files = snapshot.reader(&input);
    let texts = npmrc_texts(
        &input,
        None,
        &[(
            "NPM_CONFIG_USERCONFIG".to_owned(),
            required.to_string_lossy().into_owned(),
        )],
        &files,
    );

    assert!(texts.is_empty());
    assert!(
        files
            .failure()
            .is_some_and(|failure| failure.message().contains("required file was not found"))
    );
    std::fs::remove_dir_all(root).unwrap();
}
