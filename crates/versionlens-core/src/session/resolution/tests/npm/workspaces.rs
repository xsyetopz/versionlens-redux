use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;
#[test]
fn deno_npm_imports_use_document_npmrc_registry() {
    let root = temp_dir().join(format!("versionlens-deno-npmrc-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "registry=https://registry.example.test/\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("deno.json").display()),
        language_id: "jsonc".to_owned(),
        text: package_file_fixture("deno-npm-imports-use-document-npmrc-registry.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(dependencies[0].ecosystem, Npm);
    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://registry.example.test/chalk"]
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn pnpm_yaml_dependencies_use_document_npmrc_registry() {
    let root = temp_dir().join(format!("versionlens-pnpm-npmrc-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "@scope:registry=https://scope.example.test/npm\nregistry=https://registry.example.test/\n",
    )
    .unwrap();

    let input = DocumentInput {
        uri: format!("file://{}", root.join("pnpm-workspace.yaml").display()),
        language_id: "yaml".to_owned(),
        text: package_file_fixture("pnpm-yaml-dependencies-use-document-npmrc-registry.txt"),
        workspace_root: Some(root.to_string_lossy().into_owned()),
    };
    let context = crate::registry::registry_context_from_document(&input);
    let dependencies = standard_session().dependencies(&input);
    let session = standard_session();

    assert_eq!(
        session.registry_urls_with_context(&dependencies[0], &context),
        vec!["https://scope.example.test/npm/@scope%2fpkg"]
    );
    assert_eq!(
        session.registry_urls_with_context(&dependencies[1], &context),
        vec!["https://registry.example.test/left-pad"]
    );

    remove_dir_all(root).unwrap();
}
