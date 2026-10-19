use crossbeam_channel as _;
use lsp_server as _;
use lsp_types as _;
use serde as _;
use serde_json as _;
use tokio as _;
use versionlens_core as _;
use versionlens_model as _;
use versionlens_parsers as _;

#[cfg(test)]
#[path = "main/tests.rs"]
mod tests;

use std::io::Write as _;

const PRINT_MANIFEST_TABLE: &str = "--print-manifest-table";

pub fn main() -> anyhow::Result<()> {
    if std::env::args()
        .skip(1)
        .any(|arg| arg == PRINT_MANIFEST_TABLE)
    {
        writeln!(
            std::io::stdout().lock(),
            "{}",
            versionlens_lsp::manifest_table_json()?
        )?;
        return Ok(());
    }
    versionlens_lsp::run_stdio_server()
}
