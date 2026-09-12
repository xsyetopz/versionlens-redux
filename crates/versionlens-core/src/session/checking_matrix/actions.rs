use versionlens_model::Ecosystem::{Cargo, GitHub, Npm};

use super::RegistryResponseInput;

const FIXTURE_ROOT: &str = "tests/fixtures/checking-coverage/github-actions";

fn runtime_responses() -> Vec<RegistryResponseInput> {
    [
        ("actions/setup-node", GitHub, r#"[{"name":"v4"}]"#),
        ("oven-sh/setup-bun", GitHub, r#"[{"name":"v2"}]"#),
        (
            "actions-rust-lang/setup-rust-toolchain",
            GitHub,
            r#"[{"name":"v1"}]"#,
        ),
        (
            "node",
            Npm,
            r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
        ),
        (
            "bun",
            Npm,
            r#"[{"name":"bun-v1.0.0"},{"name":"bun-v1.2.0"}]"#,
        ),
        ("rust", Cargo, r#"[{"name":"1.80.0"},{"name":"1.90.0"}]"#),
    ]
    .into_iter()
    .map(|(package, ecosystem, body)| RegistryResponseInput::new(package, ecosystem, body))
    .collect()
}

#[path = "actions/tests.rs"]
mod tests;
