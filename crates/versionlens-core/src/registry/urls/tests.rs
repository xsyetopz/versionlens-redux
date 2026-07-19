use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::PathBuf;
use std::process::id;
use versionlens_model::DocumentInput;

use std::env;

use crate::{ProviderSettings, RegistryUrlConfig, SessionConfig};
use versionlens_model::Ecosystem::Pub;

include!("tests/hosted.rs");
include!("tests/gradle/catalogs.rs");
include!("tests/gradle/scripts.rs");
include!("tests/jvm.rs");
include!("tests/configured.rs");
include!("tests/dotnet.rs");
fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/core/registry/urls/tests")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
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
