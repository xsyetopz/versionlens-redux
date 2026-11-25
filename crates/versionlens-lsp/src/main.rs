//! `versionlens-lsp`: the `VersionLens` language server. Editors start it over stdio;
//! `--file-types` prints the manifest globs that editor packages select files with.

mod catalog;
mod config;
mod data;
mod extract;
mod flatten;
mod http;
mod osv;
mod project;
mod registry;
mod render;
mod server;
mod sort;
mod version;

use std::io::Write;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arg = std::env::args().nth(1);
    let result = match arg.as_deref() {
        Some("--version") => writeln!(
            std::io::stdout(),
            "versionlens-lsp {}",
            env!("CARGO_PKG_VERSION")
        )
        .map_err(|e| e.to_string()),
        Some("--file-types") => {
            writeln!(std::io::stdout(), "{}", catalog::globs_json()).map_err(|e| e.to_string())
        }
        Some("--stdio") | None => server::run(),
        Some(other) => Err(format!(
            "unknown argument {other}; expected --stdio, --version, or --file-types"
        )),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            drop(writeln!(std::io::stderr(), "versionlens-lsp: {error}"));
            ExitCode::FAILURE
        }
    }
}
