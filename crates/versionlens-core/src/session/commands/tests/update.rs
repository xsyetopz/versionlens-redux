use std::fs::read_to_string;
use std::path::PathBuf;

use super::{
    ApplyCommandRequest, DocumentInput, RegistryResponseInput,
    session_with_vulnerability_visibility, standard_session,
};
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, CocoaPods, Docker, Helm, Nix, Npm, Terraform, Unity,
};

include!("update/platforms.rs");
include!("update/security.rs");
include!("update/selection.rs");
fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/commands/update")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session command update fixture {}: {error}",
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

include!("update/project.rs");
include!("update/jvm.rs");
include!("update/languages.rs");
include!("update/packages.rs");
include!("update/toolchains.rs");
