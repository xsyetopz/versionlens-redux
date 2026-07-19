use std::fs::read_to_string;
use std::path::PathBuf;

use super::{DocumentInput, session_with_dependency_properties, standard_session};
use versionlens_model::Ecosystem::{Deno, Maven, Npm};
use versionlens_model::TextEdit;

include!("sort/manifests.rs");
include!("sort/ecosystems.rs");
fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/commands/sort")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session command sort fixture {}: {error}",
            path.display()
        )
    })
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

include!("sort/exclusions.rs");
