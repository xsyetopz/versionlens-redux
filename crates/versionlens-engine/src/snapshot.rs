use versionlens_model::Dependency;
use versionlens_providers::provider_id;

pub(crate) fn dependency_signature(dependencies: &[Dependency]) -> String {
    let mut entries = dependencies
        .iter()
        .map(|dependency| {
            serde_json::json!([
                provider_id(dependency.ecosystem),
                dependency.name,
                dependency.group,
                dependency.requirement,
                dependency.hosted_url,
                dependency.hosted_name,
                dependency.canonical_reference,
            ])
            .to_string()
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries.join("\n")
}

#[cfg(test)]
mod tests {
use versionlens_model::DocumentInput;
use versionlens_parsers::parse_document;

use super::dependency_signature;

fn signature(text: &str) -> String {
    dependency_signature(&parse_document(&DocumentInput::new(
        "file:///package.json".to_owned(),
        "json".to_owned(),
        text.to_owned(),
        None,
    )))
}

#[test]
fn dependency_signature_tracks_package_manager_versions() {
    assert_ne!(
        signature(r#"{"packageManager":"pnpm@9.0.0"}"#),
        signature(r#"{"packageManager":"pnpm@10.0.0"}"#),
    );
}

#[test]
fn dependency_signature_tracks_workspace_requirements() {
    assert_ne!(
        signature(r#"{"dependencies":{"member":"workspace:*"}}"#),
        signature(r#"{"dependencies":{"member":"workspace:^"}}"#),
    );
}

#[test]
fn dependency_signature_is_independent_of_property_order() {
    assert_eq!(
        signature(r#"{"dependencies":{"one":"1.0.0","two":"2.0.0"}}"#),
        signature(r#"{"dependencies":{"two":"2.0.0","one":"1.0.0"}}"#),
    );
}
}
