use super::{DocumentInput, standard_session};
use crate::RegistryResponseInput;
use crate::support::tests::{assert_no_edits, assert_suggestion};
use crate::support::tests::{file_uri, local_test_root};
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;
use std::process::id;

use versionlens_model::Ecosystem::*;

mod dotnet;
mod npm;
mod registry_sources;

fn package_file_fixture(name: &str) -> String {
    crate::support::tests::fixture("tests/fixtures/session/resolution/tests/fixed", name)
}

fn assert_ruby_path_dependency_fixture(fixture: &str, package: &str, root_name: &str) {
    let root = local_test_root(root_name);
    let app = root.join("app");
    create_dir_all(app.join("vendor/local")).unwrap();
    let output = standard_session().resolve_document_with_responses(
        DocumentInput::new(
            file_uri(&app.join("Gemfile")),
            "ruby".to_owned(),
            package_file_fixture(fixture),
            None,
        ),
        &[RegistryResponseInput::new(
            package.to_owned(),
            Ruby,
            r#"[{"number":"9.9.9"}]"#.to_owned(),
        )],
    );
    assert_eq!(output.suggestions[0].status, "directory");
    assert_eq!(
        output.suggestions[0].latest.as_deref(),
        Some("vendor/local")
    );
    assert!(output.edits.is_empty());
    remove_dir_all(root).unwrap();
}

macro_rules! resolve_fixture {
    ($uri:expr, $language:expr, $fixture:expr, $responses:expr $(,)?) => {{
        standard_session().resolve_document_with_responses(
            DocumentInput::new(
                $uri.to_owned(),
                $language.to_owned(),
                package_file_fixture($fixture),
                None,
            ),
            $responses,
        )
    }};
}

include!("fixed/local.rs");
include!("fixed/jvm.rs");
include!("fixed/manifests.rs");
include!("fixed/composer.rs");
include!("fixed/sources.rs");
include!("fixed/repositories.rs");
