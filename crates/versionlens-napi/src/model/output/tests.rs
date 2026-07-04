use versionlens_core::{AuthorizationRequestPayload, ResolveDocumentOutput};

use super::NativeResolveDocumentOutput;

#[test]
fn maps_authorization_requests_from_core_output() {
    let output = NativeResolveDocumentOutput::from_core(ResolveDocumentOutput {
        suggestions: Vec::new(),
        edits: Vec::new(),
        authorization_required_count: 1,
        authorization_required_requests: vec![AuthorizationRequestPayload {
            auth_url: "https://registry.example.test".to_owned(),
            request_url: "https://registry.example.test/left-pad".to_owned(),
        }],
        vulnerable_update_count: 0,
        vulnerable_update_package: None,
        vulnerable_update_version: None,
    });

    assert_eq!(output.authorization_required_count, 1);
    assert_eq!(output.authorization_required_requests.len(), 1);
    assert_eq!(
        output.authorization_required_requests[0].auth_url,
        "https://registry.example.test"
    );
    assert_eq!(
        output.authorization_required_requests[0].request_url,
        "https://registry.example.test/left-pad"
    );
}
