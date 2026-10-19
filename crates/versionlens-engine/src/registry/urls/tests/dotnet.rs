use versionlens_model::Ecosystem::Dotnet;

fn write_nuget_config(root: &std::path::Path, package_source_policy: &str) {
    create_dir_all(root).unwrap();
    write(
        root.join("NuGet.config"),
        format!(
            "<configuration>\n  <packageSources>\n    <add key=\"nuget.org\" value=\"https://api.nuget.org/v3/index.json\" />\n    <add key=\"private\" value=\"https://nuget.example.test/v3/index.json\" />\n  </packageSources>\n{package_source_policy}\n</configuration>\n"
        ),
    )
    .unwrap();
}

const DISABLED_NUGET_SOURCE_POLICY: &str = "  <disabledPackageSources>\n    <add key=\"nuget.org\" value=\"true\" />\n  </disabledPackageSources>";

const MAPPED_NUGET_SOURCE_POLICY: &str = "  <packageSourceMapping>\n    <packageSource key=\"nuget.org\">\n      <package pattern=\"Newtonsoft.*\" />\n    </packageSource>\n    <packageSource key=\"private\">\n      <package pattern=\"Contoso.*\" />\n    </packageSource>\n  </packageSourceMapping>";
#[test]
fn dotnet_sources_are_service_indexes_not_package_urls() {
    let session = crate::support::tests::session_with_provider_settings(
        ProviderSettings {
            registry_urls: vec![RegistryUrlConfig {
                ecosystem: Dotnet,
                url: "https://nuget.example.test/v3/index.json".to_owned(),
            }],
            ..crate::default()
        },
        false,
    );
    let input = registry_input("file:///app.csproj", "xml", "dotnet-sources-are-service-indexes-not-package-urls.csproj");
    let dependencies = session.dependencies(&input);

    let urls = session.registry_urls(&dependencies[0]);
    assert_eq!(urls[0], "https://nuget.example.test/v3/index.json");
    assert!(
        urls.iter()
            .all(|url| !url.contains("newtonsoft.json/index.json"))
    );
}

#[test]
fn dotnet_documents_use_workspace_nuget_config_sources() {
    let root = temp_dir().join(format!("versionlens-nuget-config-{}", id()));
    write_nuget_config(&root, DISABLED_NUGET_SOURCE_POLICY);

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", root.join("app.csproj").display()), "xml".to_owned(), package_file_fixture("dotnet-documents-use-workspace-nuget-config-sources.txt"), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://nuget.example.test/v3/index.json"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn paket_dependencies_use_declared_source_urls() {
    let session = crate::support::tests::test_session(false);
    let input = registry_input("file:///paket.dependencies", "plaintext", "paket-dependencies-use-declared-source-urls.dependencies");
    assert_first_registry_urls(&session, &input, &["https://nuget.example.test/v3/index.json"]);
}

#[test]
fn dotnet_child_nuget_config_clear_removes_workspace_sources() {
    let root = temp_dir().join(format!("versionlens-nuget-config-clear-{}", id()));
    let app = root.join("src");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><clear /><add key="child" value="https://child.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", app.join("app.csproj").display()), "xml".to_owned(), package_file_fixture("dotnet-child-nuget-config-clear-removes-workspace-sources.txt"), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://child.example.test/v3/index.json"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_child_nuget_config_remove_does_not_delete_inherited_cli_sources() {
    let root = temp_dir().join(format!("versionlens-nuget-config-remove-{}", id()));
    let app = root.join("src");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /><add key="keep" value="https://keep.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><remove key="root" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", app.join("app.csproj").display()), "xml".to_owned(), package_file_fixture(
            "dotnet-child-nuget-config-remove-does-not-delete-inherited-cli-sources.txt",
        ), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://root.example.test/v3/index.json",
            "https://keep.example.test/v3/index.json"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_reads_intermediate_ancestors_nearest_first() {
    let root = temp_dir().join(format!("versionlens-nuget-config-ancestors-{}", id()));
    let src = root.join("src");
    let app = src.join("app");
    create_dir_all(&app).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="root" value="https://root.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        src.join("NuGet.config"),
        r#"<configuration><packageSources><add key="src" value="https://src.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();
    write(
        app.join("NuGet.config"),
        r#"<configuration><packageSources><add key="app" value="https://app.example.test/v3/index.json" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", app.join("app.csproj").display()), "xml".to_owned(), package_file_fixture(
            "dotnet-nuget-config-reads-intermediate-ancestors-nearest-first.txt",
        ), Some(root.to_string_lossy().into_owned()));
    assert_first_registry_urls(&session, &input, &["https://app.example.test/v3/index.json",
            "https://src.example.test/v3/index.json",
            "https://root.example.test/v3/index.json"]);

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_ignores_local_file_sources_for_suggestions() {
    let root = temp_dir().join(format!("versionlens-nuget-config-local-source-{}", id()));
    create_dir_all(root.join("packages")).unwrap();
    write(
        root.join("NuGet.config"),
        r#"<configuration><packageSources><add key="local" value="./packages" /></packageSources></configuration>"#,
    )
    .unwrap();

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", root.join("app.csproj").display()), "xml".to_owned(), package_file_fixture(
            "dotnet-nuget-config-ignores-local-file-sources-for-suggestions.txt",
        ), Some(root.to_string_lossy().into_owned()));
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);

    assert!(
        session
            .registry_urls_with_context(&dependencies[0], &context)
            .is_empty()
    );

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_supplies_request_scoped_auth_headers() {
    let root = temp_dir().join(format!("versionlens-nuget-config-auth-{}", id()));
    create_dir_all(&root).unwrap();
    write(
        root.join("NuGet.config"),
        r#"
<configuration>
  <packageSources>
    <add key="private" value="https://nuget.example.test/v3/index.json" />
  </packageSources>
  <packageSourceCredentials>
    <private>
      <add key="Username" value="user" />
      <add key="ClearTextPassword" value="pass" />
    </private>
  </packageSourceCredentials>
</configuration>
"#,
    )
    .unwrap();

    let input = DocumentInput::new(format!("file://{}", root.join("app.csproj").display()), "xml".to_owned(), package_file_fixture("dotnet-nuget-config-supplies-request-scoped-auth-headers.txt"), Some(root.to_string_lossy().into_owned()));
    let context = crate::registry::RegistryContext::from_document(&input);
    let service_index_headers =
        context.auth_headers_for_url(Dotnet, "https://nuget.example.test/v3/index.json");
    let package_headers = context.auth_headers_for_url(
        Dotnet,
        "https://nuget.example.test/v3-flatcontainer/newtonsoft.json/index.json",
    );
    let other_headers =
        context.auth_headers_for_url(Dotnet, "https://other.example.test/v3/index.json");

    assert_eq!(service_index_headers.len(), 1);
    assert_eq!(service_index_headers[0].value, "Basic dXNlcjpwYXNz");
    assert_eq!(package_headers.len(), 1);
    assert_eq!(package_headers[0].value, "Basic dXNlcjpwYXNz");
    assert!(other_headers.is_empty());

    remove_dir_all(root).unwrap();
}

#[test]
fn dotnet_nuget_config_package_source_mapping_filters_sources() {
    let root = temp_dir().join(format!("versionlens-nuget-config-mapping-{}", id()));
    write_nuget_config(&root, MAPPED_NUGET_SOURCE_POLICY);

    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(format!("file://{}", root.join("app.csproj").display()), "xml".to_owned(), package_file_fixture(
            "dotnet-nuget-config-package-source-mapping-filters-sources.txt",
        ), Some(root.to_string_lossy().into_owned()));
    let (context, dependencies) = registry_context_and_dependencies(&session, &input);
    for (index, expected) in [
        (0, "https://api.nuget.org/v3/index.json"),
        (1, "https://nuget.example.test/v3/index.json"),
    ] {
        assert_eq!(
            session.registry_urls_with_context(&dependencies[index], &context),
            vec![expected]
        );
    }

    remove_dir_all(root).unwrap();
}
