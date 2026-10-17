use std::fs;
use std::path::PathBuf;

use serde_json::Value;
use versionlens_model::DocumentInput;

use super::classify_document;
use crate::{ecosystem_config_namespace, ecosystem_for_manifest};

fn repository_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read_json(relative: &str) -> Value {
    let path = repository_path(relative);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("cannot parse {}: {error}", path.display()))
}

fn field<'a>(value: &'a Value, name: &str) -> &'a str {
    value[name]
        .as_str()
        .unwrap_or_else(|| panic!("missing string field {name} in {value}"))
}

/// Every manifest fixture classifies to the kind and ecosystem that the 0.4.4
/// golden output records for it. The golden output names ecosystems by their
/// settings namespace.
#[test]
fn classifies_every_fixture_to_its_golden_kind_and_ecosystem() {
    let cases = read_json("tests/fixtures/checking-coverage/manifest-cases.json");
    let cases = cases.as_array().expect("manifest cases are a list");
    assert!(!cases.is_empty());

    for case in cases {
        let path = field(case, "path");
        let golden = read_json(&format!("tests/golden/0.4.4/{path}.json"));
        let input = DocumentInput::new(
            format!("file:///workspace/{path}"),
            field(case, "language"),
            field(case, "text"),
            Some("file:///workspace".to_owned()),
        );

        let kind = classify_document(&input);
        assert_eq!(
            format!("{kind:?}"),
            field(&golden, "kind"),
            "kind of {path}"
        );

        let ecosystem = ecosystem_for_manifest(kind)
            .unwrap_or_else(|| panic!("{path} classified to {kind:?}, which has no ecosystem"));
        let golden_ecosystems = golden["dependencies"]
            .as_array()
            .expect("golden dependencies are a list")
            .iter()
            .map(|dependency| dependency["ecosystem"].clone())
            .collect::<Vec<_>>();
        assert!(
            !golden_ecosystems.is_empty(),
            "{path} has no golden dependency"
        );
        for golden_ecosystem in golden_ecosystems {
            assert_eq!(
                Some(ecosystem_config_namespace(ecosystem)),
                golden_ecosystem.as_str(),
                "ecosystem of {path}"
            );
        }
    }
}
