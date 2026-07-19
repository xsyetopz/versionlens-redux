use lsp_server as _;
use lsp_types as _;
use serde as _;
use serde_json as _;
use versionlens_core as _;
use versionlens_model as _;
use versionlens_vscode_model as _;

pub fn main() -> anyhow::Result<()> {
    versionlens_lsp::run_stdio_server()
}
