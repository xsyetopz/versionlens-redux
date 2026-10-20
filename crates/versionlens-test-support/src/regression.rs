//! Manifests and registry responses of the group 2.2 regression fixtures,
//! shared by the engine and server tests so both check the same input.
//!
//! Ecosystems are named as the model serializes them (`"cargo"`, `"gitHub"`).

/// A canned registry response for one package.
pub struct Registry {
    pub package: &'static str,
    pub ecosystem: &'static str,
    pub body: String,
}

/// One manifest and the registry answers its check receives.
pub struct Case {
    pub uri: &'static str,
    pub language: &'static str,
    pub text: String,
    pub registry: Vec<Registry>,
}

fn registry(package: &'static str, ecosystem: &'static str, body: impl Into<String>) -> Registry {
    Registry {
        package,
        ecosystem,
        body: body.into(),
    }
}

fn case(uri: &'static str, language: &'static str, text: &str, registry: Vec<Registry>) -> Case {
    Case {
        uri,
        language,
        text: text.to_owned(),
        registry,
    }
}

fn npm_versions(versions: &[&str], latest: &str) -> String {
    let versions = versions
        .iter()
        .map(|version| format!(r#""{version}":{{}}"#))
        .collect::<Vec<_>>()
        .join(",");
    format!(r#"{{"versions":{{{versions}}},"dist-tags":{{"latest":"{latest}"}}}}"#)
}

const WORKFLOW: &str = "file:///work/.github/workflows/ci.yml";
const MAVEN_KOTLIN: &str = "<metadata><versioning><versions><version>2.0.0</version><version>2.1.0</version></versions></versioning></metadata>";

pub const GITHUB_SCRIPT_SHA: &str = "3a2844b7e9c422d3c10d287c895573f7108da1b3";

pub fn cargo_exact_pin() -> Case {
    case(
        "file:///work/Cargo.toml",
        "toml",
        "[dependencies]\nzed_extension_api = \"=0.7.0\"\n",
        vec![registry(
            "zed_extension_api",
            "cargo",
            r#"{"versions":[{"num":"0.7.0","yanked":false},{"num":"0.6.0","yanked":false}]}"#,
        )],
    )
}

pub fn python_exact_pin() -> Case {
    case(
        "file:///work/requirements.txt",
        "pip-requirements",
        "requests==2.34.2\n",
        vec![registry(
            "requests",
            "python",
            r#"{"info":{"version":"2.34.2"},"releases":{"2.34.1":[{"yanked":false}],"2.34.2":[{"yanked":false}]}}"#,
        )],
    )
}

pub fn caret_range_behind_latest() -> Case {
    case(
        "file:///work/package.json",
        "json",
        "{\"devDependencies\":{\"@biomejs/biome\":\"^2.5.8\"}}\n",
        vec![registry(
            "@biomejs/biome",
            "npm",
            npm_versions(&["2.5.8", "2.5.9", "2.5.10", "2.5.14"], "2.5.14"),
        )],
    )
}

pub const STARLIGHT_SKIPPED: [&str; 6] =
    ["0.42.0", "0.42.1", "0.42.2", "0.42.3", "0.42.4", "0.42.5"];

pub fn intermediate_releases() -> Case {
    case(
        "file:///work/package.json",
        "json",
        "{\"dependencies\":{\"@astrojs/starlight\":\"^0.41.7\"}}\n",
        vec![registry(
            "@astrojs/starlight",
            "npm",
            npm_versions(
                &[
                    "0.41.7", "0.41.11", "0.42.0", "0.42.1", "0.42.2", "0.42.3", "0.42.4",
                    "0.42.5", "0.42.6",
                ],
                "0.42.6",
            ),
        )],
    )
}

pub fn sha_pinned_action() -> Case {
    let sha = GITHUB_SCRIPT_SHA;
    case(
        WORKFLOW,
        "yaml",
        &format!("steps:\n  - uses: actions/github-script@{sha} # v9.0.0\n"),
        vec![registry(
            "actions/github-script",
            "gitHub",
            format!(
                r#"[{{"name":"v9.0.0","commit":{{"sha":"{sha}"}}}},{{"name":"v9","commit":{{"sha":"{sha}"}}}},{{"name":"v8.0.0","commit":{{"sha":"8888888888888888888888888888888888888888"}}}}]"#
            ),
        )],
    )
}

pub fn branch_action_ref() -> Case {
    case(
        WORKFLOW,
        "yaml",
        "steps:\n  - uses: taiki-e/install-action@just\n",
        vec![],
    )
}

pub fn vscode_engine_field() -> Case {
    case(
        "file:///work/package.json",
        "json",
        "{\"engines\":{\"vscode\":\"^1.75.0\"}}\n",
        vec![],
    )
}

pub fn unsupported_setup_dotnet() -> Case {
    case(
        WORKFLOW,
        "yaml",
        "steps:\n  - uses: actions/setup-dotnet@v4\n    with:\n      dotnet-version: 8.0.100.1\n",
        vec![registry(
            "actions/setup-dotnet",
            "gitHub",
            r#"[{"name":"v4"}]"#,
        )],
    )
}

pub fn workflow_env_reference() -> Case {
    case(
        WORKFLOW,
        "yaml",
        "env:\n  NODE_VERSION: \"20\"\njobs:\n  build:\n    steps:\n      - uses: actions/setup-node@v4\n        with:\n          node-version: ${{ env.NODE_VERSION }}\n",
        vec![
            registry("actions/setup-node", "gitHub", r#"[{"name":"v4"}]"#),
            registry(
                "node",
                "npm",
                r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            ),
        ],
    )
}

pub fn step_output_expression() -> Case {
    case(
        WORKFLOW,
        "yaml",
        "steps:\n  - uses: actions/setup-dotnet@v4\n    with:\n      dotnet-version: ${{ steps.read.outputs.version }}\n",
        vec![registry(
            "actions/setup-dotnet",
            "gitHub",
            r#"[{"name":"v4"}]"#,
        )],
    )
}

pub fn gradle_version_alias() -> Case {
    case(
        "file:///work/gradle/libs.versions.toml",
        "toml",
        "[versions]\nkotlin = \"2.0.0\"\n\n[libraries]\nkotlin-stdlib = { module = \"org.jetbrains.kotlin:kotlin-stdlib\", version.ref = \"kotlin\" }\n",
        vec![registry(
            "org.jetbrains.kotlin:kotlin-stdlib",
            "maven",
            MAVEN_KOTLIN,
        )],
    )
}

pub fn gradle_plugin_alias() -> Case {
    case(
        "file:///work/gradle/libs.versions.toml",
        "toml",
        "[versions]\nksp = \"1.0.0\"\n\n[plugins]\nksp = { id = \"com.google.devtools.ksp\", version.ref = \"ksp\" }\n",
        vec![registry(
            "com.google.devtools.ksp:com.google.devtools.ksp.gradle.plugin",
            "maven",
            "<metadata><versioning><versions><version>1.0.0</version><version>2.0.0</version></versions></versioning></metadata>",
        )],
    )
}

pub fn cargo_bare_version() -> Case {
    case(
        "file:///work/Cargo.toml",
        "toml",
        "[dependencies]\nserde = \"1.0.200\"\n",
        vec![registry(
            "serde",
            "cargo",
            r#"{"versions":[{"num":"1.0.229","yanked":false},{"num":"1.0.200","yanked":false}]}"#,
        )],
    )
}

/// Manifests whose own `version` field is not a dependency.
pub fn package_version_fields() -> Vec<Case> {
    vec![
        case(
            "file:///work/package.json",
            "json",
            "{\"name\":\"demo\",\"version\":\"0.4.4\"}\n",
            vec![],
        ),
        case(
            "file:///work/Cargo.toml",
            "toml",
            "[package]\nname = \"demo\"\nversion = \"0.4.4\"\n",
            vec![],
        ),
    ]
}
