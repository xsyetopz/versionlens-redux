use super::*;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;
use versionlens_model::{Position, Range};

#[test]
fn update_choices_are_suppressed_only_for_equal_exact_runtime_pins() {
    let session = crate::support::tests::test_session(false);
    let responses = [RegistryResponseInput::new(
        "bun",
        Ecosystem::Npm,
        r#"[{"name":"bun-v1.4.2"}]"#,
    )];
    let context = RegistryContext::default();
    let operation = OperationContext::default();

    let exact = session
        .fetch_runtime_latest(&bun_dependency("1.4.2"), &responses, &context, &operation)
        .unwrap();
    assert_eq!(exact.latest.as_deref(), Some("1.4.2"));
    assert!(exact.choices.is_empty());

    let selector = session
        .fetch_runtime_latest(&bun_dependency("1.4"), &responses, &context, &operation)
        .unwrap();
    assert_eq!(selector.latest.as_deref(), Some("1.4.2"));
    assert_eq!(selector.choices.len(), 1);
    assert_eq!(selector.choices[0].version, "1.4.2");
    assert_eq!(selector.choices[0].replacement, None);
}

fn bun_dependency(requirement: &str) -> Dependency {
    let position = Position {
        line: 0,
        character: 0,
    };
    let range = Range {
        start: position,
        end: position,
    };
    Dependency {
        name: "bun".to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ecosystem::Npm,
        group: "with.bun-version".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}
