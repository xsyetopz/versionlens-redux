use versionlens_model::Ecosystem::Npm;
use versionlens_model::{Position, Range};

use super::{package_json_input, standard_config};
use crate::{RegistryResponseInput, VersionLensSession};

const ASTRO: &str =
    r#"{"dist-tags":{"latest":"8.0.1"},"versions":{"7.2.2":{},"7.3.5":{},"8.0.1":{}}}"#;

async fn resolved(manifest: &str, package: &str, body: &str) -> VersionLensSession {
    let session = crate::version_lens_session(standard_config(false));
    session
        .resolve_document_with_responses(
            package_json_input(manifest),
            &[RegistryResponseInput::new(package, Npm, body)],
        )
        .await;
    session
}

const fn at(character: u32) -> Range {
    let position = Position { line: 0, character };
    Range {
        start: position,
        end: position,
    }
}

fn summary(actions: &[crate::BumpActionView]) -> Vec<(&str, &str, bool, bool)> {
    actions
        .iter()
        .map(|action| {
            (
                action.title.as_str(),
                action.edit.new_text.as_str(),
                action.preferred,
                action.diagnostic.is_some(),
            )
        })
        .collect()
}

#[tokio::test]
async fn each_offered_bump_is_an_action_with_latest_preferred() {
    let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
    let session = resolved(manifest, "astro", ASTRO).await;

    let actions = session.bump_actions(&package_json_input(manifest), at(28));

    assert_eq!(
        summary(&actions),
        [
            ("Bump to ^7.3.5", "^7.3.5", false, true),
            ("Bump to ^8.0.1", "^8.0.1", true, true),
        ]
    );
    let requirement = session
        .dependencies(&package_json_input(manifest))
        .remove(0)
        .requirement_range;
    assert!(
        actions
            .iter()
            .all(|action| action.edit.range == requirement)
    );
}

#[tokio::test]
async fn a_patch_floor_bump_is_offered_without_a_diagnostic() {
    let manifest = r#"{"devDependencies":{"@biomejs/biome":"^2.5.8"}}"#;
    let session = resolved(
        manifest,
        "@biomejs/biome",
        r#"{"dist-tags":{"latest":"2.5.14"},"versions":{"2.5.8":{},"2.5.14":{}}}"#,
    )
    .await;

    let actions = session.bump_actions(&package_json_input(manifest), at(40));

    assert_eq!(
        summary(&actions),
        [("Bump to ^2.5.14", "^2.5.14", true, false)]
    );
}

#[tokio::test]
async fn ranges_outside_a_requirement_and_unresolved_documents_get_no_actions() {
    let manifest = r#"{"dependencies":{"astro":"^7.2.2"}}"#;
    let session = resolved(manifest, "astro", ASTRO).await;
    assert!(
        session
            .bump_actions(&package_json_input(manifest), at(3))
            .is_empty()
    );

    let unresolved = crate::version_lens_session(standard_config(false));
    assert!(
        unresolved
            .bump_actions(&package_json_input(manifest), at(28))
            .is_empty()
    );
}
