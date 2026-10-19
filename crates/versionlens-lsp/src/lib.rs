pub mod protocol;
mod server;
mod state;

#[cfg(test)]
mod test_support;

pub use server::run_stdio_server;
pub use state::{VersionLensLspState, VersionLensTextDocument, into_lsp_range};

/// The manifest table as pretty JSON, for `--print-manifest-table`.
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
