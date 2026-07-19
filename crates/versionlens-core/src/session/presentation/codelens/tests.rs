use std::fs::read_to_string;
use std::path::PathBuf;

use versionlens_model::DocumentInput;

use crate::{RegistryResponseInput, SessionConfig, SuggestionIndicators};
use versionlens_model::Ecosystem::Npm;

mod actions;
mod docker;
mod npm;
mod python;
mod ranges;
mod vulnerabilities;

include!("tests/indicators.rs");
include!("tests/updates.rs");
include!("tests/status.rs");
fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("core crate should be under crates/")
        .to_path_buf()
}

fn npm_response(package: &str, latest: &str) -> RegistryResponseInput {
    RegistryResponseInput {
        package: package.to_owned(),
        ecosystem: Npm,
        body: format!(r#"{{"dist-tags":{{"latest":"{latest}"}}}}"#),
    }
}

fn test_indicators() -> SuggestionIndicators {
    SuggestionIndicators {
        latest: "L".to_owned(),
        satisfies_latest: "S".to_owned(),
        directory: "D".to_owned(),
        error: "E".to_owned(),
        no_match: "N".to_owned(),
        matched: "M".to_owned(),
        updateable: "U".to_owned(),
        updateable_vulnerable: "V".to_owned(),
        build: "B".to_owned(),
    }
}
