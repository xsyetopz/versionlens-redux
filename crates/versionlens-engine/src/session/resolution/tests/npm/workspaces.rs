#[test]
fn deno_npm_imports_use_document_npmrc_registry() {
    let root = temp_dir().join(format!("versionlens-deno-npmrc-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join(".npmrc"),
        "registry=https://registry.example.test/\n",
    )
    .unwrap();

    let input = DocumentInput::new(format!("file://{}", root.join("deno.json").display()), "jsonc".to_owned(), package_file_fixture("deno-npm-imports-use-document-npmrc-registry.txt"), Some(root.to_string_lossy().into_owned()));
    let (session, context, dependencies) = crate::session::resolution::tests::registry_case(&input);

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

    let input = DocumentInput::new(format!("file://{}", root.join("pnpm-workspace.yaml").display()), "yaml".to_owned(), package_file_fixture("pnpm-yaml-dependencies-use-document-npmrc-registry.txt"), Some(root.to_string_lossy().into_owned()));
    let (_session, _context, _dependencies) =
        crate::session::resolution::tests::registry_case_with_expected_urls(
            &input,
            &[
                &["https://scope.example.test/npm/@scope%2fpkg"],
                &["https://registry.example.test/left-pad"],
            ],
        );

    remove_dir_all(root).unwrap();
}
