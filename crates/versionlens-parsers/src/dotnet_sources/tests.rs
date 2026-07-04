use super::{
    filter_dotnet_remote_sources, parse_dotnet_enabled_sources, parse_dotnet_sources,
    parse_nuget_config_auth_entries, parse_nuget_config_source_mappings,
    parse_nuget_config_source_urls,
};

#[test]
fn parses_dotnet_sources() {
    let sources = parse_dotnet_sources(
        "D http://non-ssl/v3/index.json\nE  https://api.nuget.org/v3/index.json  \nEM C:\\Program Files (x86)\\Microsoft SDKs\\NuGetPackages\\\n",
    );

    assert_eq!(sources.len(), 3);
    assert!(!sources[0].enabled);
    assert_eq!(sources[0].protocol, "http:");
    assert!(sources[1].enabled);
    assert_eq!(sources[1].protocol, "https:");
    assert_eq!(sources[1].url, "https://api.nuget.org/v3/index.json");
    assert!(sources[2].enabled);
    assert!(sources[2].machine_wide);
    assert_eq!(
        sources[2].url,
        "C:\\Program Files (x86)\\Microsoft SDKs\\NuGetPackages\\"
    );
    assert_eq!(sources[2].protocol, "file:");
    assert!(parse_dotnet_sources("\n   \n").is_empty());
}

#[test]
fn filters_dotnet_enabled_remote_sources() {
    let sources = parse_dotnet_enabled_sources(
        "D http://disabled/v3/index.json\nE https://api.nuget.org/v3/index.json\nE http://non-ssl/v3/index.json\nEM C:\\NuGetPackages\\\n",
        &[
            "  ".to_owned(),
            " https://feed.test/v3/index.json ".to_owned(),
        ],
    );

    assert_eq!(sources.len(), 4);
    assert_eq!(sources[0].url, "https://feed.test/v3/index.json");
    assert!(sources[0].enabled);
    assert_eq!(sources[1].url, "https://api.nuget.org/v3/index.json");
    assert_eq!(sources[2].url, "http://non-ssl/v3/index.json");
    assert!(sources[3].machine_wide);

    let remote_sources = filter_dotnet_remote_sources(sources);

    assert_eq!(remote_sources.len(), 3);
    assert_eq!(remote_sources[0].protocol, "https:");
    assert_eq!(remote_sources[1].protocol, "https:");
    assert_eq!(remote_sources[2].protocol, "http:");
}

#[test]
fn marks_file_feed_urls_machine_wide() {
    let sources = parse_dotnet_enabled_sources("", &["C:\\NuGetPackages\\".to_owned()]);

    assert_eq!(sources.len(), 1);
    assert!(sources[0].machine_wide);
    assert_eq!(sources[0].protocol, "file:");
}

#[test]
fn parses_nuget_config_package_sources() {
    let urls = parse_nuget_config_source_urls(
        r#"
<configuration>
  <packageSources>
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
    <add key="private" value="https://nuget.example.test/v3/index.json" />
    <add key="local" value="./packages" />
  </packageSources>
  <disabledPackageSources>
    <add key="private" value="true" />
  </disabledPackageSources>
</configuration>
"#,
    );

    assert_eq!(
        urls,
        vec!["https://api.nuget.org/v3/index.json", "./packages"]
    );
}

#[test]
fn parses_nuget_config_clear_elements_per_section() {
    let text = r#"
<configuration>
  <packageSources>
    <add key="old" value="https://old.example.test/v3/index.json" />
    <clear />
    <add key="new" value="https://new.example.test/v3/index.json" />
  </packageSources>
  <disabledPackageSources>
    <add key="new" value="true" />
    <clear />
  </disabledPackageSources>
  <packageSourceMapping>
    <packageSource key="old"><package pattern="Old.*" /></packageSource>
    <clear />
    <packageSource key="new"><package pattern="New.*" /></packageSource>
  </packageSourceMapping>
</configuration>
"#;

    let urls = parse_nuget_config_source_urls(text);
    let mappings = parse_nuget_config_source_mappings(text);

    assert_eq!(urls, vec!["https://new.example.test/v3/index.json"]);
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].source, "new");
    assert_eq!(mappings[0].pattern, "New.*");
}

#[test]
fn parses_nuget_config_remove_elements_per_section() {
    let text = r#"
<configuration>
  <packageSources>
    <add key="old" value="https://old.example.test/v3/index.json" />
    <add key="new" value="https://new.example.test/v3/index.json" />
    <remove key="old" />
  </packageSources>
  <disabledPackageSources>
    <add key="new" value="true" />
    <remove key="new" />
  </disabledPackageSources>
  <packageSourceMapping>
    <packageSource key="old"><package pattern="Old.*" /></packageSource>
    <packageSource key="new"><package pattern="New.*" /></packageSource>
    <remove key="old" />
  </packageSourceMapping>
</configuration>
"#;

    let urls = parse_nuget_config_source_urls(text);
    let mappings = parse_nuget_config_source_mappings(text);

    assert_eq!(urls, vec!["https://new.example.test/v3/index.json"]);
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].source, "new");
}

#[test]
fn parses_nuget_config_clear_text_credentials() {
    let entries = parse_nuget_config_auth_entries(
        r#"
<configuration>
  <packageSources>
    <add key="private" value="https://nuget.example.test/v3/index.json" />
    <add key="disabled" value="https://disabled.example.test/v3/index.json" />
  </packageSources>
  <disabledPackageSources>
    <add key="disabled" value="true" />
  </disabledPackageSources>
  <packageSourceCredentials>
    <private>
      <add key="Username" value="user" />
      <add key="ClearTextPassword" value="pass" />
    </private>
    <disabled>
      <add key="Username" value="ignored" />
      <add key="ClearTextPassword" value="ignored" />
    </disabled>
  </packageSourceCredentials>
</configuration>
"#,
    );

    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].registry,
        "https://nuget.example.test/v3/index.json"
    );
    assert_eq!(entries[0].header_value, "Basic dXNlcjpwYXNz");
}

#[test]
fn parses_nuget_config_package_source_mappings() {
    let mappings = parse_nuget_config_source_mappings(
        r#"
<configuration>
  <packageSourceMapping>
    <packageSource key="nuget.org">
      <package pattern="Newtonsoft.*" />
      <package pattern="Serilog" />
    </packageSource>
    <packageSource key="private">
      <package pattern="Contoso.*" />
    </packageSource>
  </packageSourceMapping>
</configuration>
"#,
    );

    assert_eq!(mappings.len(), 3);
    assert_eq!(mappings[0].source, "nuget.org");
    assert_eq!(mappings[0].pattern, "Newtonsoft.*");
    assert_eq!(mappings[1].source, "nuget.org");
    assert_eq!(mappings[1].pattern, "Serilog");
    assert_eq!(mappings[2].source, "private");
    assert_eq!(mappings[2].pattern, "Contoso.*");
}
