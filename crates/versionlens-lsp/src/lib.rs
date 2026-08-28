mod server;
mod state;

#[cfg(test)]
mod test_support;

pub use server::run_stdio_server;
pub use state::{VersionLensLspState, VersionLensTextDocument, into_lsp_range};
