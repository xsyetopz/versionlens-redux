use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::read_to_string;
use std::fs::remove_dir_all;
use std::fs::write;
use std::path::{Path, PathBuf};
use std::process::id;
use std::time::UNIX_EPOCH;

use super::{DocumentInput, RegistryResponseInput, standard_session};
use versionlens_model::Ecosystem::{Cran, Go, Helm, Maven, Npm, Ruby, Terraform};

mod dotnet;
mod npm;
mod registry_sources;

include!("fixed/local.rs");
include!("fixed/jvm.rs");
include!("fixed/manifests.rs");
include!("fixed/composer.rs");
include!("fixed/sources.rs");
include!("fixed/repositories.rs");
fn package_file_fixture(name: &str) -> String {
    let path = repo_root()
        .join("tests/fixtures/session/resolution/tests/fixed")
        .join(name);
    read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read session resolution fixture {}: {error}",
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
