use super::{
    http_status_message_from_code, latest_version_for_requirement, npm_error_status_from_response,
};
use crate::response::RegistryErrorStatus::{
    Error as RegistryStatusError, Invalid as RegistryStatusInvalid,
    InvalidWithLatest as RegistryStatusInvalidWithLatest,
    NotSupported as RegistryStatusNotSupported,
};
use versionlens_model::Ecosystem::Npm;

#[test]
fn resolves_npm_dist_tag_requirements() {
    assert_eq!(
        latest_version_for_requirement(
            Npm,
            "typescript",
            "next",
            r#"{"dist-tags":{"latest":"6.0.3","next":"7.0.0-beta.1"},"versions":{"6.0.3":{},"7.0.0-beta.1":{}}}"#,
        ),
        Some("7.0.0-beta.1".to_owned())
    );
}

#[test]
fn reads_npm_error_statuses_from_responses() {
    assert_eq!(
        npm_error_status_from_response(r#"{"status":"ECONNREFUSED"}"#),
        Some(RegistryStatusError("connection refused".to_owned()))
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"code":"EUNSUPPORTEDPROTOCOL"}"#),
        Some(RegistryStatusNotSupported)
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"error":{"code":"EINVALIDTAGNAME"}}"#),
        Some(RegistryStatusInvalidWithLatest(
            "invalid version".to_owned()
        ))
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"error":{"code":"EINVALIDPACKAGENAME"}}"#),
        Some(RegistryStatusInvalid("invalid version".to_owned()))
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"status":"E404"}"#),
        Some(RegistryStatusError("not found".to_owned()))
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"status":128}"#),
        Some(RegistryStatusError("not found".to_owned()))
    );
    assert_eq!(
        npm_error_status_from_response(r#"{"status":"ENOTFOUND"}"#),
        Some(RegistryStatusError("ENOTFOUND".to_owned()))
    );
}

#[test]
fn maps_known_http_status_messages() {
    assert_eq!(http_status_message_from_code(400), Some("400 bad request"));
    assert_eq!(
        http_status_message_from_code(401),
        Some("401 not authorized")
    );
    assert_eq!(http_status_message_from_code(403), Some("403 forbidden"));
    assert_eq!(http_status_message_from_code(404), Some("not found"));
    assert_eq!(
        http_status_message_from_code(500),
        Some("500 internal server error")
    );
    assert_eq!(http_status_message_from_code(418), None);
}
