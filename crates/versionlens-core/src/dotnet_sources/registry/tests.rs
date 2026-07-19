use crate::RegistryUrlConfig;

use super::dotnet_registry_source_urls;
use versionlens_model::Ecosystem::{Cargo, Dotnet};

#[test]
fn dotnet_registry_urls_are_enriched_from_rust_owned_source_listing() {
    let configured = vec![
        RegistryUrlConfig {
            ecosystem: Cargo,
            url: "https://mirror.test/crates".to_owned(),
        },
        RegistryUrlConfig {
            ecosystem: Dotnet,
            url: " https://configured.nuget/v3/index.json ".to_owned(),
        },
    ];

    let urls = dotnet_registry_source_urls(
        &configured,
        Some(
            "E  https://enabled.nuget/v3/index.json\nD  https://disabled.nuget/v3/index.json\nEM file:///local\n",
        ),
    );

    assert_eq!(
        urls,
        [
            "https://configured.nuget/v3/index.json",
            "https://enabled.nuget/v3/index.json",
            "file:///local"
        ]
    );
}
