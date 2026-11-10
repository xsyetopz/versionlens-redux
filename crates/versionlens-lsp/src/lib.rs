//! Editor-neutral `VersionLens` language server.

pub mod protocol;
mod server;
mod state;

#[cfg(test)]
mod test_support {
    use lsp_types::CodeLens;

    pub(crate) fn assert_update_code_lenses(lenses: &[CodeLens]) {
        assert!(!lenses.is_empty(), "no code lenses");
        assert!(lenses.iter().all(|lens| {
            lens.command.as_ref().is_some_and(|command| {
                !command.title.is_empty()
                    && [
                        crate::state::UPDATE_COMMAND,
                        crate::state::CHOOSE_BUILD_COMMAND,
                    ]
                    .contains(&command.command.as_str())
                    && command.arguments.as_ref().is_some_and(|arguments| {
                        arguments.len() == 1
                            && arguments[0].get("uri").is_some()
                            && arguments[0].get("dependency").is_some()
                    })
            })
        }));
    }
}

pub use server::run_stdio_server;
pub use state::{VersionLensLspState, VersionLensTextDocument, into_lsp_range};

/// The manifest table as pretty JSON, for `--print-manifest-table`.
///
/// # Errors
///
/// Returns an error when the table cannot be serialized.
pub fn manifest_table_json() -> anyhow::Result<String> {
    Ok(serde_json::to_string_pretty(
        &versionlens_parsers::manifest_table(),
    )?)
}

#[cfg(test)]
mod manifest_table_tests {
    use super::manifest_table_json;

    #[test]
    fn manifest_table_json_lists_manifests() {
        let json: serde_json::Value =
            serde_json::from_str(&manifest_table_json().expect("json")).expect("parses");
        assert!(
            json["manifests"]
                .as_array()
                .is_some_and(|rows| !rows.is_empty())
        );
    }
}
