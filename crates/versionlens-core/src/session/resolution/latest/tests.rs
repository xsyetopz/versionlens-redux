use versionlens_model::DocumentInput;
use versionlens_model::Ecosystem::Npm;

use super::LatestResolutionRequest;
use crate::RegistryResponseInput;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

#[tokio::test]
async fn registry_response_lookup_carries_every_release_version() {
    let session = crate::support::tests::test_session(false);
    let input = DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        r#"{"dependencies":{"left-pad":"1.0.0"}}"#.to_owned(),
        None,
    );
    let context = RegistryContext::from_document(&input);
    let dependencies = session.dependencies(&input);
    let body = r#"{"dist-tags":{"latest":"1.2.0"},"versions":{"1.0.0":{},"1.1.0":{},"1.2.0":{}}}"#;
    let responses = [RegistryResponseInput::new("left-pad", Npm, body)];

    let lookup = session
        .resolve_latest(LatestResolutionRequest {
            dependency: &dependencies[0],
            responses: &responses,
            has_registry_response: true,
            context: &context,
            operation: &OperationContext::default(),
        })
        .await;

    let mut versions = lookup.versions;
    versions.sort();
    assert_eq!(versions, ["1.0.0", "1.1.0", "1.2.0"]);
}
