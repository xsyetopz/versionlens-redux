use super::release_versions_from_response_for_package;
use versionlens_model::Ecosystem::Dotnet;

#[test]
fn extracts_dotnet_versions_for_update_choices() {
    assert_eq!(
        release_versions_from_response_for_package(
            Dotnet,
            "Example.Package",
            r#"{"versions":["1.4.1","1.4.2","1.4.3-beta.1","1.5.0","invalid","1.6.2"]}"#,
        ),
        vec![
            "1.4.1".to_owned(),
            "1.4.2".to_owned(),
            "1.4.3-beta.1".to_owned(),
            "1.5.0".to_owned(),
            "1.6.2".to_owned(),
        ]
    );
}
