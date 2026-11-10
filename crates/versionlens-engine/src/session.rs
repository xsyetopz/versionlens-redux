use std::collections::{HashMap, hash_map::RandomState};
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use crate::vulnerability::VulnerabilityCheck;
use versionlens_fetch::{CacheKey, MemoryCache};
use versionlens_model::{Dependency, DocumentInput, Ecosystem, ManifestKind, TextEdit};
use versionlens_update::Suggestion;

use crate::concurrency::Priority;
use crate::config::SessionConfig;
use crate::contract;
use crate::contract::{
    AuthorizationRequestPayload, ResolveDocumentOutput, ResolveDocumentOutputParts,
    resolve_document_output,
};
use crate::presentation::suggestion::into_suggestion_views;
use cache::CachedLatest;

mod authorization;
mod cache;
mod cached;
mod checking;
#[cfg(test)]
mod checking_matrix {
    use crate::ProviderSettings;
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::Path;

    use serde::Deserialize;
    use versionlens_model::{DocumentInput, Ecosystem, TextEdit, VersionableKind};

    use super::{VersionLensSession, WorkspaceDiscoveryOptions};
    use crate::{EnabledProviderConfig, RegistryResponseInput};

    mod support {
        use crate::ProviderSettings;
        use std::cmp::Reverse;
        use std::collections::BTreeSet;

        use versionlens_model::{Position, TextEdit};

        use super::super::VersionLensSession;
        use super::{CategoryCase, ManifestCase};
        use crate::test_support::session_config;

        pub(super) fn response_body(case: &ManifestCase) -> String {
            match case.dependency.ecosystem.as_str() {
        "Cargo" if case.dependency.package == "rust" => runtime_response("rust"),
        "Cargo" => r#"{"versions":[{"num":"2.0.0","yanked":false}]}"#.to_owned(),
        "Composer" => format!(
            r#"{{"packages":{{"{}":[{{"version":"1.0.0"}},{{"version":"2.0.0"}}]}}}}"#,
            case.dependency.package
        ),
        "Deno" => r#"{"versions":{"2.0.0":{}}}"#.to_owned(),
        "Dotnet" | "Bazel" => r#"{"versions":["1.0.0","2.0.0"]}"#.to_owned(),
        "Docker" => r#"{"tags":["1.0.0","2.0.0"]}"#.to_owned(),
        "Dub" => r#"{"versions":[{"version":"2.0.0"}]}"#.to_owned(),
        "Ruby" => r#"[{"number":"2.0.0"}]"#.to_owned(),
        "Go" => "v1.0.0\nv2.0.0\n".to_owned(),
        "Maven" => "<metadata><versioning><versions><version>1.0.0</version><version>2.0.0</version></versions></versioning></metadata>".to_owned(),
        "Hex" => r#"{"releases":[{"version":"1.0.0"},{"version":"2.0.0"}]}"#.to_owned(),
        "Opam" => format!("<h2>{} version</h2><p>2.0.0 (latest)</p>", case.dependency.package),
        "Hackage" => r#"{"1.0.0":"normal","2.0.0":"normal"}"#.to_owned(),
        "Julia" => "[1.0.0]\ngit-tree-sha1 = \"a\"\n[2.0.0]\ngit-tree-sha1 = \"b\"\n".to_owned(),
        "Cran" => format!("Package: {}\nVersion: 2.0.0\n", case.dependency.package),
        "Conan" => format!(r#"{{"results":["{}/1.0.0","{}/2.0.0"]}}"#, case.dependency.package, case.dependency.package),
        "Vcpkg" | "Terraform" => {
            r#"{"versions":[{"version":"1.0.0"},{"version":"2.0.0"}]}"#.to_owned()
        }
        "Cpp" | "Swift" | "Zig" | "Nim" => r#"[{"name":"2.0.0"}]"#.to_owned(),
        "Nix" => r#"[{"name":"nixos-23.11"},{"name":"nixos-24.05"}]"#.to_owned(),
        "GitHub" => r#"[{"name":"v1.0.0"},{"name":"v2.0.0"}]"#.to_owned(),
        "CocoaPods" => r#"{"versions":["2.0.0"]}"#.to_owned(),
        "LuaRocks" => format!("repository = {{ [\"{}\"] = {{ [\"1.0.0-1\"] = {{}}, [\"2.0.0-1\"] = {{}} }} }}", case.dependency.package),
        "Cpan" => r#"{"status":"latest","version":"2.0.0"}"#.to_owned(),
        "Haxelib" => format!("<code>haxelib install {} 1.0.0</code><code>haxelib install {} 2.0.0</code>", case.dependency.package, case.dependency.package),
        "Helm" => format!("apiVersion: v1\nentries:\n  {}:\n    - version: 2.0.0\n", case.dependency.package),
        "AnsibleGalaxy" => r#"{"data":[{"version":"1.0.0"},{"version":"2.0.0"}]}"#.to_owned(),
        "Unity" | "Npm" if case.dependency.package != "node" && case.dependency.package != "bun" => npm_response(&case.dependency.package),
        "Npm" => runtime_response(&case.dependency.package),
        "Python" => r#"{"info":{"version":"2.0.0"},"releases":{"1.0.0":[],"2.0.0":[{"yanked":false}]}}"#.to_owned(),
        "Pub" => r#"{"latest":{"version":"2.0.0"}}"#.to_owned(),
        ecosystem => panic!("missing deterministic response for {ecosystem}"),
    }
        }

        pub(super) fn npm_response(package: &str) -> String {
            format!(r#"{{"name":"{package}","dist-tags":{{"latest":"2.0.0"}}}}"#)
        }

        fn runtime_response(package: &str) -> String {
            match package {
                "node" => r#"[{"version":"v1.0.0"},{"version":"v2.0.0"}]"#.to_owned(),
                "bun" => r#"[{"name":"bun-v1.0.0"},{"name":"bun-v2.0.0"}]"#.to_owned(),
                "rust" => r#"[{"name":"1.0.0"},{"name":"2.0.0"}]"#.to_owned(),
                _ => npm_response(package),
            }
        }

        pub(super) fn suggestion_for<'a>(
            output: &'a crate::contract::ResolveDocumentOutput,
            package: &str,
        ) -> &'a crate::SuggestionView {
            output
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == package)
                .unwrap_or_else(|| panic!("no suggestion for {package}: {output:?}"))
        }

        pub(super) fn manifest_cases() -> Vec<ManifestCase> {
            serde_json::from_str(include_str!(
                "../../../tests/fixtures/checking-coverage/manifest-cases.json"
            ))
            .unwrap()
        }

        pub(super) fn category_cases() -> Vec<CategoryCase> {
            serde_json::from_str(include_str!(
                "../../../tests/fixtures/checking-coverage/versionable-categories.json"
            ))
            .unwrap()
        }

        pub(super) fn slice_range<'a>(text: &'a str, edit: &TextEdit) -> &'a str {
            &text[position_offset(text, edit.range.start)..position_offset(text, edit.range.end)]
        }

        pub(super) fn apply_edits(text: &str, edits: &[TextEdit]) -> String {
            let mut edits = edits
                .iter()
                .map(|edit| {
                    (
                        position_offset(text, edit.range.start),
                        position_offset(text, edit.range.end),
                        &edit.new_text,
                    )
                })
                .collect::<Vec<_>>();
            edits.sort_by_key(|(start, _, _)| Reverse(*start));
            let mut result = text.to_owned();
            for (start, end, replacement) in edits {
                result.replace_range(start..end, replacement);
            }
            result
        }

        fn position_offset(text: &str, position: Position) -> usize {
            let line_start = text
                .match_indices('\n')
                .take(position.line as usize)
                .last()
                .map_or(0, |(offset, _)| offset + 1);
            let line = &text[line_start..];
            let mut utf16 = 0_u32;
            for (offset, character) in line.char_indices() {
                if utf16 == position.character {
                    return line_start + offset;
                }
                utf16 += if character.len_utf16() == 1 { 1 } else { 2 };
                assert!(
                    utf16 <= position.character,
                    "position splits a UTF-16 code point"
                );
            }
            assert_eq!(utf16, position.character, "position is outside its line");
            text.len()
        }

        pub(super) fn test_session() -> VersionLensSession {
            VersionLensSession::new(session_config(ProviderSettings::default(), false))
        }

        /// Each manifest table row carries its parser, so the rows are the dispatch set.
        pub(super) fn parser_dispatch_kind_names() -> BTreeSet<String> {
            versionlens_parsers::MANIFESTS
                .iter()
                .map(|row| format!("{:?}", row.kind))
                .collect()
        }

        pub(super) fn versionable_kind_names() -> BTreeSet<String> {
            let source = include_str!("../../versionlens-model/src/document.rs");
            let variants = source
                .split_once("pub enum VersionableKind {")
                .expect("VersionableKind must remain declared in the model")
                .1
                .split_once('}')
                .expect("VersionableKind declaration must be complete")
                .0;
            variants
                .lines()
                .map(|line| line.trim().trim_end_matches(','))
                .filter(|variant| !variant.is_empty() && !variant.starts_with("//"))
                .map(str::to_owned)
                .collect()
        }
    }

    use support::*;

    mod actions {
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

        mod tests {
            use super::{FIXTURE_ROOT, runtime_responses};
            use crate::ApplyCommandRequest;
            use crate::session::checking_matrix::{
                WorkspaceDiscoveryOptions, apply_edits, test_session,
            };
            use crate::test_support::fixture;
            use crate::workspace::tests::support::TestWorkspace;

            #[tokio::test]
            async fn github_action_runtime_pins_complete_the_workspace_checking_pipeline() {
                let pinned = fixture(FIXTURE_ROOT, "runtime-pins.yml");
                let workspace = TestWorkspace::new("actions-matrix");
                workspace.write(".github/workflows/ci.yml", &pinned);
                let session = test_session();
                let inputs = session
                    .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
                        workspace.path().to_owned(),
                    ]))
                    .collect::<Result<Vec<_>, _>>()
                    .expect("the workflow must be discovered without failures");
                assert_eq!(inputs.len(), 1);
                let input = &inputs[0];
                assert_eq!(input.text, pinned);

                let dependencies = session.dependencies(input);
                for (name, requirement, group) in [
                    ("node", "20.0.0", "with.node-version"),
                    ("bun", "1.0.0", "with.bun-version"),
                    ("rust", "1.80.0", "with.toolchain"),
                ] {
                    assert!(
                        dependencies.iter().any(|dependency| {
                            dependency.name == name
                                && dependency.requirement == requirement
                                && dependency.group == group
                        }),
                        "missing {name} {requirement} from {dependencies:?}"
                    );
                }

                let responses = runtime_responses();
                let resolved = session
                    .resolve_document_with_responses(input.clone(), &responses)
                    .await;
                for name in ["node", "bun", "rust"] {
                    let suggestion = resolved
                        .suggestions
                        .iter()
                        .find(|suggestion| suggestion.dependency.name == name)
                        .unwrap_or_else(|| panic!("missing resolved {name}: {resolved:?}"));
                    assert_eq!(suggestion.status, "updateAvailable", "{name}");
                }
                assert_eq!(
                    apply_edits(&pinned, &resolved.edits),
                    fixture(FIXTURE_ROOT, "runtime-pins-updated.yml")
                );
                assert!(session.document_is_fresh(input));

                let cached = session.analyze_document(input.clone());
                assert!(cached.is_supported_manifest);
                assert!(cached.status.visible);
                assert!(cached.status.update_count >= 3);
                assert!(
                    cached
                        .code_lenses
                        .iter()
                        .filter(|lens| !lens.action_kind().is_empty())
                        .count()
                        >= 3
                );
            }

            #[tokio::test]
            async fn github_action_runtime_constraints_never_raise_the_declared_minimum() {
                let constrained = fixture(FIXTURE_ROOT, "runtime-constraints.yml");
                let workspace = TestWorkspace::new("actions-constraints");
                workspace.write(".github/workflows/constraints.yml", &constrained);
                let session = test_session();
                let input = session
                    .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
                        workspace.path().to_owned(),
                    ]))
                    .next()
                    .expect("the workflow must be discovered")
                    .expect("the workflow must be readable");
                let responses = runtime_responses();
                let resolved = session
                    .resolve_document_with_responses(input.clone(), &responses)
                    .await;

                assert!(resolved.edits.is_empty(), "{resolved:?}");
                for (name, minimum) in [("node", ">=20"), ("bun", "^1.0"), ("rust", ">=1.80")] {
                    let suggestion = resolved
                        .suggestions
                        .iter()
                        .find(|suggestion| {
                            suggestion.dependency.name == name
                                && suggestion.dependency.requirement == minimum
                        })
                        .unwrap_or_else(|| panic!("missing constrained {name}: {resolved:?}"));
                    assert_eq!(suggestion.status, "satisfiesLatest", "{name}");
                    let forced = session
                        .apply_command_with_selected_version(ApplyCommandRequest {
                            input: input.clone(),
                            command: Some("update"),
                            dependency_name: Some(name),
                            selected_version: Some("99.0.0"),
                            responses: &responses,
                        })
                        .await;
                    assert!(forced.edits.is_empty(), "{name}: {forced:?}");
                }
                assert!(session.document_is_fresh(&input));
                let cached = session.analyze_document(input);
                assert!(cached.status.visible);
                assert!(
                    cached
                        .code_lenses
                        .iter()
                        .all(|lens| lens.action_kind().is_empty())
                );
            }
        }
    }

    mod regressions {
        use versionlens_model::{DocumentInput, Ecosystem, VersionableKind};

        use super::{apply_edits, test_session};
        use crate::RegistryResponseInput;
        use crate::workspace::tests::support::TestWorkspace;

        #[tokio::test]
        async fn nimble_equality_edit_replaces_the_complete_constraint() {
            let session = test_session();
            let text = "requires \"foo == 1.0.0\"\n";
            let input = DocumentInput::new("file:///coverage/demo.nimble", "nim", text, None);
            let response =
                RegistryResponseInput::new("foo", Ecosystem::Nim, r#"[{"name":"2.0.0"}]"#);
            let output = session
                .resolve_document_with_responses(input, &[response])
                .await;
            assert_eq!(
                apply_edits(text, &output.edits),
                "requires \"foo == 2.0.0\"\n"
            );
        }

        #[tokio::test]
        async fn luarocks_equality_edit_replaces_the_complete_constraint() {
            let session = test_session();
            let text =
                "package = \"demo\"\nversion = \"1.0.0-1\"\ndependencies = { \"foo == 1.0.0\" }\n";
            let input = DocumentInput::new("file:///coverage/demo.rockspec", "lua", text, None);
            let response = RegistryResponseInput::new(
                "foo",
                Ecosystem::LuaRocks,
                r#"repository = { ["foo"] = { ["1.0.0-1"] = {}, ["2.0.0-1"] = {} } }"#,
            );
            let output = session
                .apply_command_with_selected_version(super::super::ApplyCommandRequest {
                    input,
                    command: Some("update"),
                    dependency_name: Some("foo"),
                    selected_version: Some("2.0.0-1"),
                    responses: &[response],
                })
                .await;
            assert_eq!(output.edits.len(), 1);
            assert_eq!(
                apply_edits(text, &output.edits),
                "package = \"demo\"\nversion = \"1.0.0-1\"\ndependencies = { \"foo == 2.0.0-1\" }\n"
            );
        }

        #[tokio::test]
        async fn dune_equality_edits_preserve_operator_and_quotes() {
            let cases = [
                (
                    "(lang dune 3.17)\n(package (name demo) (depends (foo (= 1.0.0))))\n",
                    "(lang dune 3.17)\n(package (name demo) (depends (foo (= 2.0.0))))\n",
                ),
                (
                    "(lang dune 3.17)\n(package (name demo) (depends (foo (= \"1.0.0\"))))\n",
                    "(lang dune 3.17)\n(package (name demo) (depends (foo (= \"2.0.0\"))))\n",
                ),
            ];
            for (text, expected) in cases {
                let session = test_session();
                let input =
                    DocumentInput::new("file:///coverage/dune-project", "plaintext", text, None);
                let response = RegistryResponseInput::new(
                    "foo",
                    Ecosystem::Opam,
                    "<h2>foo version</h2><p>2.0.0 (latest)</p>",
                );
                let output = session
                    .resolve_document_with_responses(input, &[response])
                    .await;
                assert_eq!(output.edits.len(), 1);
                assert_eq!(apply_edits(text, &output.edits), expected);
            }
        }

        #[tokio::test]
        async fn gradle_catalog_inline_version_edit_replaces_the_value() {
            let session = test_session();
            let text =
                "[libraries]\nfoo = { module = \"example.test:foo\", version = \"1.0.0\" }\n";
            let input = DocumentInput::new(
                "file:///coverage/gradle/libs.versions.toml",
                "toml",
                text,
                None,
            );
            let response = RegistryResponseInput::new(
                "example.test:foo",
                Ecosystem::Maven,
                "<metadata><versioning><versions><version>1.0.0</version><version>2.0.0</version></versions></versioning></metadata>",
            );
            let output = session
                .apply_command_with_selected_version(super::super::ApplyCommandRequest {
                    input,
                    command: Some("update"),
                    dependency_name: Some("example.test:foo"),
                    selected_version: Some("2.0.0"),
                    responses: &[response],
                })
                .await;
            assert_eq!(output.edits.len(), 1);
            assert_eq!(
                apply_edits(text, &output.edits),
                "[libraries]\nfoo = { module = \"example.test:foo\", version = \"2.0.0\" }\n"
            );
        }

        #[tokio::test]
        async fn leiningen_project_version_is_local() {
            let session = test_session();
            let text = "(defproject demo \"0.1.0\" :dependencies [[example.test/foo \"1.0.0\"]])\n";
            let input = DocumentInput::new("file:///coverage/project.clj", "clojure", text, None);
            let project = session
                .dependencies(&input)
                .into_iter()
                .find(|dependency| dependency.name == "demo")
                .expect("Leiningen project version must parse");
            assert_eq!(project.versionable_kind(), VersionableKind::ProjectVersion);

            let output = session.resolve_document(input).await;
            let suggestion = output
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == "demo")
                .expect("Leiningen project version must resolve locally");
            assert_eq!(suggestion.status, "updateAvailable");
            assert_eq!(suggestion.latest.as_deref(), Some("0.1.1"));
        }

        #[tokio::test]
        async fn cargo_inherited_version_edits_only_the_workspace_root() {
            let workspace = TestWorkspace::new("cargo-inheritance");
            let root_text =
                "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nversion = \"1.0.0\"\n";
            let member_text = "[package]\nname = \"member\"\nversion.workspace = true\n";
            let input = |relative: &str, text: &str, version| {
                DocumentInput::new(
                    crate::workspace_file_uri(&workspace.path().join(relative)).unwrap(),
                    "toml",
                    text,
                    Some(workspace.path().to_string_lossy().into_owned()),
                )
                .with_version(version)
            };
            let root = input("Cargo.toml", root_text, 1);
            let member = input("member/Cargo.toml", member_text, 2);
            let session = test_session();
            assert!(session.set_workspace_documents(vec![root.clone(), member.clone()]));

            let root_dependency = session.command_dependencies(&root).remove(0);
            assert_eq!(
                root_dependency.versionable_kind(),
                VersionableKind::ProjectVersion
            );
            let root_output = session
                .apply_command(root, Some("updatePatch"), Some("version"), &[])
                .await;
            assert_eq!(root_output.edits.len(), 1);
            assert_eq!(
                apply_edits(root_text, &root_output.edits),
                "[workspace]\nmembers = [\"member\"]\n[workspace.package]\nversion = \"1.0.1\"\n"
            );

            let member_dependency = session.command_dependencies(&member).remove(0);
            assert_eq!(member_dependency.requirement, "workspace:true");
            assert_eq!(
                member_dependency.versionable_kind(),
                VersionableKind::WorkspaceReference
            );
            let member_output = session
                .apply_command(member, Some("updatePatch"), Some("version"), &[])
                .await;
            assert_eq!(member_output.edits.len(), 0);
        }
    }

    mod golden {
        use std::fs;
        use std::path::PathBuf;

        use serde_json::{Value, json};
        use versionlens_model::DocumentInput;

        use super::super::ApplyCommandRequest;
        use super::{ManifestCase, manifest_cases, response_body, test_session};
        use crate::workspace::tests::support::TestWorkspace;
        use crate::{
            Action, DiagnosticView, LensView, RegistryResponseInput, Severity, UpdateMode,
        };

        /// Writes the lens, diagnostic and edit output of every manifest case to
        /// `tests/golden/<release>/`, where the release is the case's `since`, or
        /// 0.4.4 for kinds that release supported. Run with
        /// `cargo test -p versionlens-engine capture_golden_output -- --ignored`.
        #[tokio::test]
        #[ignore = "writes tests/golden"]
        async fn capture_golden_output() {
            let golden = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
            for case in manifest_cases() {
                let release = case.since.as_deref().unwrap_or("0.4.4");
                let output = golden.join(release).join(format!("{}.json", case.path));
                fs::create_dir_all(output.parent().unwrap()).unwrap();
                fs::write(output, capture_case(&case).await).unwrap();
            }
        }

        /// One manifest case checked against its canned registry response, with the
        /// command of every lens run against the same response.
        struct CaseRun {
            root_uri: String,
            root: String,
            text: String,
            analysis: crate::AnalyzeDocumentOutput,
            resolved: crate::ResolveDocumentOutput,
            actions: Vec<LensRun>,
        }

        struct LensRun {
            lens: String,
            command: String,
            dependency: String,
            selected_version: Option<String>,
            output: crate::ResolveDocumentOutput,
        }

        async fn run_case(case: &ManifestCase) -> CaseRun {
            let workspace = TestWorkspace::new("golden");
            let root_uri = crate::workspace_file_uri(workspace.path()).unwrap();
            let input = DocumentInput::new(
                crate::workspace_file_uri(&workspace.write(&case.path, &case.text)).unwrap(),
                &case.language,
                &case.text,
                Some(root_uri.clone()),
            );
            let session = test_session();
            let ecosystem = session
                .dependencies(&input)
                .into_iter()
                .find(|dependency| dependency.name == case.dependency.package)
                .unwrap_or_else(|| {
                    panic!("{} did not parse {}", case.kind, case.dependency.package)
                })
                .ecosystem;
            let responses = [RegistryResponseInput::new(
                case.dependency
                    .response_package
                    .as_deref()
                    .unwrap_or(&case.dependency.package),
                ecosystem,
                response_body(case),
            )];
            let resolved = session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            let analysis = session.analyze_document(input.clone());
            let mut actions = Vec::new();
            for lens in &analysis.code_lenses {
                for (command, dependency_name, selected_version) in lens_actions(lens) {
                    let output = session
                        .apply_command_with_selected_version(ApplyCommandRequest {
                            input: input.clone(),
                            command: Some(command),
                            dependency_name: Some(dependency_name),
                            selected_version,
                            responses: &responses,
                        })
                        .await;
                    actions.push(LensRun {
                        lens: lens.title.clone(),
                        command: command.to_owned(),
                        dependency: dependency_name.to_owned(),
                        selected_version: selected_version.map(ToOwned::to_owned),
                        output,
                    });
                }
            }
            let root = workspace.path().to_string_lossy().into_owned();
            fs::remove_dir_all(workspace.path()).unwrap();
            CaseRun {
                root_uri,
                root,
                text: case.text.clone(),
                analysis,
                resolved,
                actions,
            }
        }

        async fn capture_case(case: &ManifestCase) -> String {
            let run = run_case(case).await;
            let actions = run
                .actions
                .iter()
                .map(|action| {
                    json!({
                        "lens": action.lens,
                        "command": action.command,
                        "dependency": action.dependency,
                        "selectedVersion": action.selected_version,
                        "edits": action.output.edits,
                    })
                })
                .collect::<Vec<_>>();
            let golden = json!({
                "kind": case.kind,
                "path": case.path,
                "dependencies": run.analysis.dependencies,
                "codeLenses": run.analysis.code_lenses.iter().map(golden_code_lens).collect::<Vec<_>>(),
                // Update diagnostics have their own tests. Leaving them out means a
                // golden file changes only when lens, edit or vulnerability output does.
                "diagnostics": run
                    .analysis
                    .diagnostics
                    .iter()
                    .filter(|diagnostic| diagnostic.code.as_deref() != Some(crate::UPDATE_AVAILABLE))
                    .map(golden_diagnostic)
                    .collect::<Vec<_>>(),
                "updateAllEdits": run.resolved.edits,
                "actions": actions,
            });
            let text = serde_json::to_string_pretty(&golden).unwrap() + "\n";
            let text = text.replace(&run.root_uri, "file:///workspace");
            text.replace(&run.root, "/workspace")
        }

        /// Every lens that offers an update must change the document when clicked.
        /// A lens whose command has no edit, or only edits that keep the text, is a
        /// no-op and must not be shown.
        #[tokio::test]
        async fn every_update_lens_changes_the_document() {
            let mut noops = Vec::new();
            for case in manifest_cases() {
                let run = run_case(&case).await;
                for action in &run.actions {
                    let planned = action.output.edit_plan.as_ref().is_some_and(|plan| {
                        plan.documents
                            .iter()
                            .any(|document| !document.edits.is_empty())
                    });
                    let changes = action
                        .output
                        .edits
                        .iter()
                        .any(|edit| text_at(&run.text, edit.range) != edit.new_text);
                    if !planned && !changes {
                        noops.push(format!(
                            "{}: {:?} ({})",
                            case.path, action.lens, action.command
                        ));
                    }
                }
            }
            assert!(noops.is_empty(), "no-op lenses:\n{}", noops.join("\n"));
        }

        /// The text of `range`, counting characters as UTF-16 units like LSP does.
        fn text_at(text: &str, range: versionlens_model::Range) -> String {
            let offset = |position: versionlens_model::Position| {
                let line_start: usize = text
                    .split_inclusive('\n')
                    .take(position.line as usize)
                    .map(str::len)
                    .sum();
                let mut units = 0;
                let line = &text[line_start..];
                let within = line
                    .char_indices()
                    .find(|(_, character)| {
                        let reached = units >= position.character as usize;
                        units += character.len_utf16();
                        reached
                    })
                    .map_or(line.len(), |(index, _)| index);
                line_start + within
            };
            text[offset(range.start)..offset(range.end)].to_owned()
        }

        /// Converts a lens to the golden `CodeLensPayload` shape: a command ID,
        /// positional arguments, and a point range moved `order` characters right,
        /// because VS Code sorts the lenses of a line by position.
        fn golden_code_lens(lens: &LensView) -> Value {
            let mut range = lens.range;
            range.start.character = range
                .start
                .character
                .saturating_add(lens.order)
                .min(lens.range.end.character);
            range.end = range.start;
            json!({
                "range": range,
                "title": lens.title,
                "command": golden_command(lens),
                "arguments": lens.arguments(),
            })
        }

        /// The VS Code command ID of the golden format for this lens action.
        fn golden_command(lens: &LensView) -> &'static str {
            match lens.action_kind() {
                "UpdateDependency" => "versionlens.suggestion.onUpdateDependency",
                "ChooseBuild" => "versionlens.suggestion.onChooseBuild",
                "OpenFile" => "versionlens.suggestion.onFileLink",
                _ => "",
            }
        }

        /// Converts a diagnostic to the golden `DiagnosticPayload` shape, whose severity is a number.
        fn golden_diagnostic(diagnostic: &DiagnosticView) -> Value {
            let severity = match diagnostic.severity {
                Severity::Error => 0,
                Severity::Warning => 1,
                Severity::Information => 2,
                Severity::Hint => 3,
            };
            json!({
                "range": diagnostic.range,
                "message": diagnostic.message,
                "severity": severity,
                "source": diagnostic.source,
                "code": diagnostic.code,
                "codeDescriptionUrl": diagnostic.code_description_url,
            })
        }

        pub(super) fn lens_actions(lens: &LensView) -> Vec<(&str, &str, Option<&str>)> {
            match &lens.action {
                Some(Action::UpdateDependency {
                    name,
                    mode,
                    version,
                    ..
                }) => vec![(
                    mode.map_or("update", UpdateMode::as_str),
                    name.as_str(),
                    version.as_deref(),
                )],
                Some(Action::ChooseBuild { name, builds, .. }) => builds
                    .iter()
                    .map(|build| ("update", name.as_str(), Some(build.as_str())))
                    .collect(),
                Some(Action::OpenFile { .. }) | None => Vec::new(),
            }
        }

        /// Commit that first captured `tests/golden/0.4.4` from the napi build.
        const BASELINE_COMMIT: &str = "67f3b1d4";

        fn golden_path(case: &ManifestCase) -> PathBuf {
            let release = case.since.as_deref().unwrap_or("0.4.4");
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/golden")
                .join(release)
                .join(format!("{}.json", case.path))
        }

        /// The engine output of every manifest case equals its file in `tests/golden`.
        #[tokio::test]
        async fn engine_output_matches_the_golden_folder() {
            let mut stale = Vec::new();
            for case in manifest_cases() {
                let expected = fs::read_to_string(golden_path(&case)).unwrap_or_default();
                if capture_case(&case).await != expected {
                    stale.push(case.path.clone());
                }
            }
            assert!(
                stale.is_empty(),
                "golden files out of date:\n{}",
                stale.join("\n")
            );
        }

        /// `tests/golden/0.4.4` differs from the 67f3b1d4 capture exactly in the files
        /// that `tests/golden/DIFFERENCES.md` lists under a `### <path>` heading: no
        /// unlisted difference, no stale entry, no file missing from the baseline. The
        /// check panics under `CI` and is skipped locally where git or that commit is
        /// unavailable.
        #[test]
        fn golden_differences_with_the_baseline_are_documented() {
            let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
            let documented = fs::read_to_string(root.join("DIFFERENCES.md")).unwrap();
            let listed: Vec<&str> = documented
                .lines()
                .filter_map(|line| line.strip_prefix("### `")?.strip_suffix('`'))
                .collect();
            let git = |args: &[&str]| {
                std::process::Command::new("git")
                    .current_dir(&root)
                    .args(args)
                    .output()
                    .ok()
                    .filter(|output| output.status.success())
            };
            let Some(files) = git(&["ls-tree", "-r", "--name-only", BASELINE_COMMIT, "0.4.4"])
            else {
                assert!(
                    std::env::var_os("CI").is_none(),
                    "git history for {BASELINE_COMMIT} is unavailable; fetch with depth 0"
                );
                eprintln!("skipping baseline comparison: {BASELINE_COMMIT} is unavailable");
                return;
            };
            let prefix =
                git(&["rev-parse", "--show-prefix"]).map(|o| String::from_utf8(o.stdout).unwrap());
            let prefix = prefix.unwrap_or_default().trim().to_owned();
            let baseline_files: Vec<String> = String::from_utf8(files.stdout)
                .unwrap()
                .lines()
                .map(|file| file.trim_start_matches("0.4.4/").to_owned())
                .collect();
            let mut problems = Vec::new();
            for name in &baseline_files {
                let baseline = git(&["show", &format!("{BASELINE_COMMIT}:{prefix}0.4.4/{name}")]);
                let current = fs::read(root.join("0.4.4").join(name)).ok();
                let same = baseline.map(|o| o.stdout) == current;
                let is_listed = listed.contains(&name.as_str());
                if !same && !is_listed {
                    problems.push(format!("undocumented difference: {name}"));
                }
                if (same || current.is_none()) && is_listed {
                    problems.push(format!("stale entry (identical or missing): {name}"));
                }
            }
            for entry in &listed {
                if !baseline_files.iter().any(|name| name == entry) {
                    problems.push(format!("listed but not in the baseline: {entry}"));
                }
            }
            let mut pending = vec![root.join("0.4.4")];
            while let Some(dir) = pending.pop() {
                for entry in fs::read_dir(dir)
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                {
                    if entry.is_dir() {
                        pending.push(entry);
                        continue;
                    }
                    let name = entry
                        .strip_prefix(root.join("0.4.4"))
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/");
                    if !baseline_files.contains(&name) {
                        problems.push(format!("not in the baseline: {name}"));
                    }
                }
            }
            assert!(
                problems.is_empty(),
                "0.4.4 golden files disagree with {BASELINE_COMMIT} and DIFFERENCES.md:\n{}",
                problems.join("\n")
            );
        }
    }

    mod fixtures {
        //! Regression fixtures for reported suggestion bugs. Each test states the
        //! behavior its spec scenario requires.

        use versionlens_model::{DocumentInput, Ecosystem};
        use versionlens_test_support::regression;

        use super::super::{ApplyCommandRequest, VersionLensSession};
        use super::golden::lens_actions;
        use super::test_session;
        use crate::contract::{AnalyzeDocumentOutput, ResolveDocumentOutput};
        use crate::{ProviderSettings, RegistryResponseInput, RegistryUrlConfig};

        struct Checked {
            resolved: ResolveDocumentOutput,
            analysis: AnalyzeDocumentOutput,
            bumps: Vec<String>,
        }

        async fn check(
            session: &VersionLensSession,
            uri: &str,
            language: &str,
            text: &str,
            responses: &[RegistryResponseInput],
        ) -> Checked {
            let input = DocumentInput::new(uri, language, text, None);
            let resolved = session
                .resolve_document_with_responses(input.clone(), responses)
                .await;
            let analysis = session.analyze_document(input.clone());
            let mut bumps = Vec::new();
            for lens in &analysis.code_lenses {
                for (command, dependency_name, selected_version) in lens_actions(lens) {
                    let output = session
                        .apply_command_with_selected_version(ApplyCommandRequest {
                            input: input.clone(),
                            command: Some(command),
                            dependency_name: Some(dependency_name),
                            selected_version,
                            responses,
                        })
                        .await;
                    bumps.extend(output.edits.into_iter().map(|edit| edit.new_text));
                }
            }
            bumps.sort();
            bumps.dedup();
            Checked {
                resolved,
                analysis,
                bumps,
            }
        }

        /// A session whose registries refuse connections, so any lookup surfaces as
        /// an error instead of reaching the network.
        fn offline_session() -> VersionLensSession {
            let unreachable = |ecosystem| RegistryUrlConfig {
                ecosystem,
                url: "http://127.0.0.1:9/".to_owned(),
            };
            crate::test_support::session_with_provider_settings(
                ProviderSettings {
                    registry_urls: vec![
                        unreachable(Ecosystem::GitHub),
                        unreachable(Ecosystem::Npm),
                        unreachable(Ecosystem::Dotnet),
                    ],
                    ..Default::default()
                },
                false,
            )
        }

        fn titles(checked: &Checked) -> Vec<&str> {
            checked
                .analysis
                .code_lenses
                .iter()
                .map(|lens| lens.title.as_str())
                .collect()
        }

        fn assert_status(checked: &Checked, status: &str) {
            assert!(
                titles(checked)
                    .iter()
                    .any(|title| title.ends_with(status) && !title.starts_with('↑')),
                "expected status `{status}`, lenses: {:?}",
                titles(checked)
            );
        }

        fn assert_no_bump(checked: &Checked) {
            assert!(
                titles(checked).iter().all(|title| !title.starts_with('↑')),
                "bump lens offered: {:?}",
                titles(checked)
            );
            assert_eq!(
                checked.bumps,
                Vec::<String>::new(),
                "lenses: {:?}",
                titles(checked)
            );
            assert_eq!(checked.resolved.edits, vec![]);
        }

        /// No error, and no diagnostic other than an available update.
        fn assert_silent(checked: &Checked) {
            let noise = checked
                .analysis
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code.as_deref() != Some(crate::UPDATE_AVAILABLE))
                .collect::<Vec<_>>();
            assert_eq!(noise, Vec::<&crate::DiagnosticView>::new());
            assert!(
                checked
                    .resolved
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.status != "error"),
                "suggestions: {:?}",
                checked.resolved.suggestions
            );
        }

        /// Checks a shared regression fixture, with the registry answers it names.
        async fn check_case(session: &VersionLensSession, case: &regression::Case) -> Checked {
            let responses = case
                .registry
                .iter()
                .map(|stub| {
                    let ecosystem = serde_json::from_value(serde_json::json!(stub.ecosystem))
                        .unwrap_or_else(|error| panic!("{}: {error}", stub.ecosystem));
                    RegistryResponseInput::new(stub.package, ecosystem, stub.body.clone())
                })
                .collect::<Vec<_>>();
            check(session, case.uri, case.language, &case.text, &responses).await
        }

        #[tokio::test]
        async fn cargo_exact_pin_at_latest_is_current() {
            let checked = check_case(&test_session(), &regression::cargo_exact_pin()).await;
            assert_status(&checked, "latest 0.7.0");
            assert_no_bump(&checked);
        }

        #[tokio::test]
        async fn python_exact_pin_at_latest_is_current() {
            let checked = check_case(&test_session(), &regression::python_exact_pin()).await;
            assert_status(&checked, "latest 2.34.2");
            assert_no_bump(&checked);
        }

        #[tokio::test]
        async fn caret_range_behind_latest_offers_a_floor_bump_without_a_diagnostic() {
            let checked =
                check_case(&test_session(), &regression::caret_range_behind_latest()).await;
            assert_status(&checked, "satisfies latest 2.5.14");
            assert_eq!(checked.bumps, ["^2.5.14"]);
            assert_eq!(checked.analysis.diagnostics, vec![]);
        }

        #[tokio::test]
        async fn intermediate_releases_are_not_listed_individually() {
            let checked = check_case(&test_session(), &regression::intermediate_releases()).await;
            assert_status(&checked, "satisfies 0.41.11");
            assert_eq!(checked.bumps, ["^0.41.11", "^0.42.6"]);
            for skipped in regression::STARLIGHT_SKIPPED {
                assert!(
                    titles(&checked)
                        .iter()
                        .all(|title| !title.contains(skipped)),
                    "{skipped} listed: {:?}",
                    titles(&checked)
                );
            }
        }

        #[tokio::test]
        async fn sha_pinned_action_with_floating_and_exact_tags_is_current() {
            let checked = check_case(&test_session(), &regression::sha_pinned_action()).await;
            assert_status(&checked, "latest v9.0.0");
            assert_no_bump(&checked);
        }

        #[tokio::test]
        async fn branch_action_ref_is_fixed_without_a_lookup() {
            let checked = check_case(&offline_session(), &regression::branch_action_ref()).await;
            assert_silent(&checked);
            assert_eq!(checked.resolved.suggestions[0].status, "fixed");
            assert_no_bump(&checked);
        }

        #[tokio::test]
        async fn vscode_engine_field_is_skipped_without_an_error() {
            let checked = check_case(&offline_session(), &regression::vscode_engine_field()).await;
            assert_silent(&checked);
        }

        #[tokio::test]
        async fn unsupported_setup_dotnet_syntax_is_not_supported_rather_than_an_error() {
            let checked =
                check_case(&offline_session(), &regression::unsupported_setup_dotnet()).await;
            assert_silent(&checked);
            let dotnet = checked
                .resolved
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == "dotnet")
                .unwrap_or_else(|| {
                    panic!("dotnet not checked: {:?}", checked.resolved.suggestions)
                });
            assert_eq!(dotnet.status, "notSupported");
        }

        #[tokio::test]
        async fn workflow_env_reference_resolves_the_runtime() {
            let checked = check_case(&test_session(), &regression::workflow_env_reference()).await;
            assert_silent(&checked);
            let node = checked
                .resolved
                .suggestions
                .iter()
                .find(|suggestion| suggestion.dependency.name == "node")
                .unwrap_or_else(|| panic!("node not resolved: {:?}", checked.resolved.suggestions));
            assert_eq!(node.dependency.requirement, "20");
        }

        #[tokio::test]
        async fn unresolvable_step_output_expression_is_skipped_silently() {
            let checked =
                check_case(&offline_session(), &regression::step_output_expression()).await;
            assert_silent(&checked);
            assert!(
                checked
                    .resolved
                    .suggestions
                    .iter()
                    .all(|suggestion| suggestion.dependency.name == "actions/setup-dotnet"),
                "expression was looked up: {:?}",
                checked.resolved.suggestions
            );
        }

        #[tokio::test]
        async fn gradle_catalog_version_alias_resolves_through_its_library() {
            let checked = check_case(&test_session(), &regression::gradle_version_alias()).await;
            assert_silent(&checked);
            let edit = checked
                .resolved
                .edits
                .iter()
                .find(|edit| edit.new_text == "2.1.0")
                .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
            assert_eq!(edit.range.start.line, 1);
        }

        #[tokio::test]
        async fn gradle_catalog_plugin_alias_uses_the_plugin_marker() {
            let checked = check_case(&test_session(), &regression::gradle_plugin_alias()).await;
            assert_silent(&checked);
            let edit = checked
                .resolved
                .edits
                .iter()
                .find(|edit| edit.new_text == "2.0.0")
                .unwrap_or_else(|| panic!("no bump: {:?}", checked.resolved));
            assert_eq!(edit.range.start.line, 1);
        }

        #[tokio::test]
        async fn cargo_bare_version_is_a_caret_requirement() {
            let checked = check_case(&test_session(), &regression::cargo_bare_version()).await;
            assert_status(&checked, "satisfies latest 1.0.229");
            assert_eq!(checked.bumps, ["1.0.229"]);
        }

        #[tokio::test]
        async fn package_version_fields_are_not_dependencies() {
            for case in regression::package_version_fields() {
                let uri = case.uri;
                let checked = check_case(&offline_session(), &case).await;
                assert_eq!(checked.analysis.dependencies, vec![], "{uri}");
                assert_eq!(titles(&checked), Vec::<&str>::new(), "{uri}");
                assert_eq!(checked.analysis.diagnostics, vec![], "{uri}");
            }
        }
    }

    #[derive(Debug, Deserialize)]
    struct ManifestCase {
        kind: String,
        path: String,
        /// The release that first supported this manifest kind. It names the
        /// golden folder, so kinds 0.4.4 never read do not enter its baseline.
        since: Option<String>,
        language: String,
        text: String,
        #[serde(flatten)]
        dependency: ManifestDependency,
        #[serde(flatten)]
        resolution: ManifestResolution,
    }

    #[derive(Debug, Deserialize)]
    struct ManifestDependency {
        package: String,
        response_package: Option<String>,
        ecosystem: String,
        requirement: String,
        range_text: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct ManifestResolution {
        latest: Option<String>,
        selected: Option<String>,
        edit: Option<String>,
        expected_text: Option<String>,
        status: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct CategoryCase {
        name: String,
        path: String,
        text: String,
        package: String,
        kind: String,
        status: Option<String>,
        edit: Option<String>,
    }

    #[tokio::test]
    async fn supported_manifests_complete_the_checking_pipeline() {
        let cases = manifest_cases();
        assert_authoritative_case_coverage(&cases);
        assert_editor_selector_coverage(&cases);
        let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-coverage");
        for case in &cases {
            workspace.write(&case.path, &case.text);
        }
        let session = test_session();
        let discovered = session
            .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
                workspace.path().to_owned(),
            ]))
            .collect::<Result<Vec<_>, _>>()
            .expect("coverage workspace discovery must have no failures");
        assert_eq!(
            discovered.len(),
            cases.len(),
            "every case must be discovered"
        );

        let by_path = cases
            .iter()
            .map(|case| (case.path.as_str(), case))
            .collect::<BTreeMap<_, _>>();
        for input in discovered {
            let relative = crate::workspace_path(&input.uri)
                .and_then(|path| path.strip_prefix(workspace.path()).ok().map(Path::to_owned))
                .expect("discovered URI must remain below its root");
            let relative = relative.to_string_lossy().replace('\\', "/");
            let case = by_path
                .get(relative.as_str())
                .unwrap_or_else(|| panic!("unexpected discovered manifest {relative}"));
            assert_manifest_pipeline(&session, &input, case).await;
        }
    }

    #[tokio::test]
    async fn versionable_categories_remain_distinct_and_safe() {
        let cases = category_cases();
        let actual_kinds = cases
            .iter()
            .map(|case| case.kind.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(actual_kinds, versionable_kind_names());
        let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-categories");
        fs::create_dir_all(workspace.root.join("foo")).unwrap();
        let session = test_session();
        let responses = [
            RegistryResponseInput::new("foo", Ecosystem::Npm, npm_response("foo")),
            RegistryResponseInput::new(
                "node",
                Ecosystem::Npm,
                r#"[{"version":"v20.0.0"},{"version":"v22.0.0"}]"#,
            ),
            RegistryResponseInput::new(
                "npm",
                Ecosystem::Npm,
                r#"{"versions":{"1.0.0":{},"2.0.0":{}},"dist-tags":{"latest":"2.0.0"}}"#,
            ),
        ];

        for case in cases {
            let input = DocumentInput::new(
                crate::workspace_file_uri(&workspace.write(&case.path, &case.text)).unwrap(),
                "json",
                &case.text,
                Some(crate::workspace_file_uri(workspace.path()).unwrap()),
            );
            let dependency = session
                .dependencies(&input)
                .into_iter()
                .find(|dependency| dependency.name == case.package)
                .unwrap_or_else(|| panic!("{} did not parse {}", case.name, case.package));
            assert_eq!(
                format!("{:?}", dependency.versionable_kind()),
                case.kind,
                "{}",
                case.name
            );
            let output = session
                .resolve_document_with_responses(input.clone(), &responses)
                .await;
            if let Some(status) = case.status.as_deref() {
                let suggestion = suggestion_for(&output, &case.package);
                assert_eq!(suggestion.status, status, "{}", case.name);
            }
            match case.edit.as_deref() {
                Some(edit) => assert_eq!(
                    output.edits.as_slice(),
                    [TextEdit {
                        range: dependency.requirement_range,
                        new_text: edit.to_owned(),
                    }],
                    "{}",
                    case.name
                ),
                None => assert!(
                    output.edits.is_empty(),
                    "{} changed its declared value: {output:?}",
                    case.name
                ),
            }
            let rendered = session.analyze_document(input);
            if case.status.is_some() {
                let suggestion = suggestion_for(&output, &case.package);
                assert_semantic_target_uniqueness(&rendered, suggestion, &case.name);
            }
            assert!(
                !rendered.code_lenses.is_empty(),
                "{} did not render",
                case.name
            );
            if case.kind == "RuntimeConstraint" {
                assert!(
                    rendered
                        .code_lenses
                        .iter()
                        .all(|lens| lens.action_kind().is_empty())
                );
            }
        }
    }

    #[tokio::test]
    async fn disabled_provider_is_excluded_from_discovery_and_analysis() {
        let workspace = crate::workspace::tests::support::TestWorkspace::new("disabled-coverage");
        workspace.write("package.json", r#"{"dependencies":{"foo":"1.0.0"}}"#);
        workspace.write("Cargo.toml", "[dependencies]\nfoo = \"1.0.0\"\n");
        let mut config = crate::test_support::session_config(ProviderSettings::default(), false);
        config.enabled_providers = vec![EnabledProviderConfig {
            ecosystem: Ecosystem::Cargo,
            manifest_kind: None,
        }];
        let session = VersionLensSession::new(config);
        let discovered = session
            .discover_workspace_documents(WorkspaceDiscoveryOptions::new(vec![
                workspace.path().to_owned(),
            ]))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(discovered.len(), 1);
        assert!(discovered[0].uri.ends_with("/Cargo.toml"));

        let npm = DocumentInput::new(
            crate::workspace_file_uri(&workspace.root.join("package.json")).unwrap(),
            "json",
            r#"{"dependencies":{"foo":"1.0.0"}}"#,
            None,
        );
        let analyzed = session.analyze_document(npm.clone());
        assert!(!analyzed.is_supported_manifest);
        assert_eq!(analyzed.dependencies.len(), 0);
        assert_eq!(session.resolve_document(npm).await.suggestions.len(), 0);
    }

    #[tokio::test]
    async fn response_failure_is_rendered_and_never_edited() {
        let session = test_session();
        let input = DocumentInput::new(
            "file:///coverage/package.json",
            "json",
            r#"{"dependencies":{"foo":"1.0.0"}}"#,
            None,
        );
        let response = RegistryResponseInput::new("foo", Ecosystem::Npm, r#"{"status":"E404"}"#);
        let output = session
            .resolve_document_with_responses(input.clone(), &[response])
            .await;
        assert_eq!(suggestion_for(&output, "foo").status, "error");
        assert_eq!(output.edits.len(), 0);
        assert!(session.document_is_fresh(&input));
        let rendered = session.analyze_document(input);
        assert_eq!(rendered.status.error_count, 1);
        assert_eq!(rendered.code_lenses.len(), 1);
    }

    #[tokio::test]
    async fn dotnet_fixed_status_survives_memory_cache() {
        let session = test_session();
        let input = DocumentInput::new(
            "file:///coverage/app.csproj",
            "xml",
            "<Project><ItemGroup><PackageReference Include=\"Foo\" Version=\"1.0.0\" /></ItemGroup></Project>\n",
            None,
        );
        let response = RegistryResponseInput::new(
            "Foo",
            Ecosystem::Dotnet,
            r#"{"versions":["1.0.0","2.0.0"]}"#,
        );
        let first = session
            .resolve_document_with_responses(input.clone(), &[response])
            .await;
        let cached = session.resolve_document(input).await;
        assert_eq!(suggestion_for(&first, "Foo").status, "fixed");
        assert_eq!(
            suggestion_for(&cached, "Foo").status,
            suggestion_for(&first, "Foo").status
        );
        assert_eq!(
            suggestion_for(&cached, "Foo").latest,
            suggestion_for(&first, "Foo").latest
        );
    }

    #[tokio::test]
    async fn successful_result_survives_memory_clear_and_session_restart() {
        let workspace = crate::workspace::tests::support::TestWorkspace::new("checking-cache");
        let directory = workspace.root.join("cache");
        let input = DocumentInput::new(
            "file:///coverage/package.json",
            "json",
            r#"{"dependencies":{"foo":"1.0.0"}}"#,
            None,
        );
        let first = test_session().with_persistent_cache(&directory).unwrap();
        assert!(!first.document_is_fresh(&input));
        let response = RegistryResponseInput::new("foo", Ecosystem::Npm, npm_response("foo"));
        let checked = first
            .resolve_document_with_responses(input.clone(), &[response])
            .await;
        assert_eq!(
            suggestion_for(&checked, "foo").latest.as_deref(),
            Some("2.0.0")
        );
        assert!(first.document_is_fresh(&input));
        first.clear_memory_cache();
        assert!(first.document_is_fresh(&input));

        let restarted = test_session().with_persistent_cache(&directory).unwrap();
        assert!(restarted.document_is_fresh(&input));
        let restored = restarted.resolve_document(input.clone()).await;
        assert_eq!(
            suggestion_for(&restored, "foo").latest.as_deref(),
            Some("2.0.0")
        );
        assert_eq!(
            apply_edits(&input.text, &restored.edits),
            r#"{"dependencies":{"foo":"2.0.0"}}"#
        );
        fs::remove_dir_all(directory).unwrap();
    }

    async fn assert_manifest_pipeline(
        session: &VersionLensSession,
        input: &DocumentInput,
        case: &ManifestCase,
    ) {
        let dependency = assert_manifest_input(session, input, case);
        let response = RegistryResponseInput::new(
            case.dependency
                .response_package
                .as_deref()
                .unwrap_or(&case.dependency.package),
            dependency.ecosystem,
            response_body(case),
        );
        let resolved = session
            .resolve_document_with_responses(input.clone(), std::slice::from_ref(&response))
            .await;
        let suggestion = suggestion_for(&resolved, &case.dependency.package);
        assert_eq!(
            suggestion.status,
            case.resolution
                .status
                .as_deref()
                .unwrap_or("updateAvailable"),
            "{} suggestion: {suggestion:?}",
            case.kind,
        );
        assert_eq!(
            suggestion.latest.as_deref(),
            case.resolution.latest.as_deref(),
            "{} suggestion: {suggestion:?}",
            case.kind,
        );
        assert!(
            resolved
                .suggestions
                .iter()
                .all(|suggestion| suggestion.status != "error"),
            "{} produced an unchecked error: {resolved:?}",
            case.kind
        );
        assert!(
            session.document_is_fresh(input),
            "{} did not cache its result",
            case.kind
        );
        if case.resolution.selected.is_none() {
            let cached = session.resolve_document(input.clone()).await;
            let cached_suggestion = suggestion_for(&cached, &case.dependency.package);
            assert_eq!(
                cached_suggestion.status, suggestion.status,
                "{} cache status",
                case.kind
            );
            assert_eq!(
                cached_suggestion.latest, suggestion.latest,
                "{} cache latest",
                case.kind
            );
        }
        assert_manifest_edits(ManifestEditCheck {
            session,
            input,
            case,
            dependency: &dependency,
            response: &response,
            resolved: &resolved,
        })
        .await;
        let rendered = session.analyze_document(input.clone());
        assert_semantic_target_uniqueness(&rendered, suggestion, &case.kind);
        assert!(
            !rendered.code_lenses.is_empty(),
            "{} had no rendered result",
            case.kind
        );
        assert!(
            rendered.status.visible,
            "{} had no rendered status",
            case.kind
        );
    }

    fn assert_semantic_target_uniqueness(
        rendered: &crate::AnalyzeDocumentOutput,
        suggestion: &crate::SuggestionView,
        case: &str,
    ) {
        let targets = rendered
            .code_lenses
            .iter()
            .filter_map(|lens| match &lens.action {
                Some(crate::Action::UpdateDependency {
                    name,
                    version: Some(version),
                    ..
                }) if *name == suggestion.dependency.name => {
                    Some(versionlens_update::semantic_update_target(version))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            targets.iter().collect::<BTreeSet<_>>().len(),
            targets.len(),
            "{case} emitted semantically duplicate update targets: {suggestion:?}"
        );
        if suggestion.status == "satisfiesLatest"
            && let Some(latest) = suggestion.latest.as_deref()
        {
            let latest = versionlens_update::semantic_update_target(latest);
            assert!(
                !targets.contains(&latest),
                "{case} emitted a status and update action for the same target: {suggestion:?}"
            );
        }
    }

    fn assert_manifest_input(
        session: &VersionLensSession,
        input: &DocumentInput,
        case: &ManifestCase,
    ) -> versionlens_model::Dependency {
        assert_eq!(
            input.text, case.text,
            "{} discovery changed text",
            case.kind
        );
        let kind = session.classify_document(input);
        assert!(
            versionlens_parsers::provider_name_for_manifest(kind).is_some(),
            "{} classified unsupported",
            case.kind
        );
        assert_eq!(format!("{kind:?}"), case.kind);
        let dependencies = session.dependencies(input);
        let dependency = dependencies
            .iter()
            .find(|dependency| {
                (dependency.name.as_str(), dependency.requirement.as_str())
                    == (
                        case.dependency.package.as_str(),
                        case.dependency.requirement.as_str(),
                    )
            })
            .unwrap_or_else(|| {
                panic!(
                    "{} did not parse {} {}: {dependencies:?}",
                    case.kind, case.dependency.package, case.dependency.requirement,
                )
            });
        assert_eq!(
            format!("{:?}", dependency.ecosystem),
            case.dependency.ecosystem
        );
        if dependency.versionable_kind() == VersionableKind::ProjectVersion {
            assert!(
                session.document_is_fresh(input),
                "{} project result was not immediately fresh",
                case.kind
            );
        } else {
            assert!(
                !session.document_is_fresh(input),
                "{} was fresh before checking",
                case.kind
            );
        }
        let analysis = session.analyze_document(input.clone());
        assert!(analysis.is_supported_manifest, "{} unsupported", case.kind);
        assert_eq!(
            analysis.active_provider_name.as_deref(),
            versionlens_parsers::provider_name_for_manifest(kind)
        );
        dependency.clone()
    }

    struct ManifestEditCheck<'a> {
        session: &'a VersionLensSession,
        input: &'a DocumentInput,
        case: &'a ManifestCase,
        dependency: &'a versionlens_model::Dependency,
        response: &'a RegistryResponseInput,
        resolved: &'a crate::contract::ResolveDocumentOutput,
    }

    async fn assert_manifest_edits(check: ManifestEditCheck<'_>) {
        let ManifestEditCheck {
            session,
            input,
            case,
            dependency,
            response,
            resolved,
        } = check;
        let selected_output = if let Some(selected) = case.resolution.selected.as_deref() {
            Some(
                session
                    .apply_command_with_selected_version(super::ApplyCommandRequest {
                        input: input.clone(),
                        command: Some("update"),
                        dependency_name: Some(case.dependency.package.as_str()),
                        selected_version: Some(selected),
                        responses: std::slice::from_ref(response),
                    })
                    .await,
            )
        } else {
            None
        };
        let edits = selected_output
            .as_ref()
            .map_or(&resolved.edits, |output| &output.edits);
        if case.resolution.selected.is_some() {
            assert!(
                resolved
                    .edits
                    .iter()
                    .all(|edit| edit.range != dependency.requirement_range),
                "{} silently edited a fixed pin",
                case.kind
            );
        }
        match case.resolution.edit.as_deref() {
            Some(expected) => {
                let edit = edits
                    .iter()
                    .find(|edit| edit.range == dependency.requirement_range)
                    .unwrap_or_else(|| panic!("{} omitted its update edit", case.kind));
                assert_eq!(edit.new_text, expected, "{}", case.kind);
                assert_eq!(
                    slice_range(&input.text, edit),
                    case.dependency
                        .range_text
                        .as_deref()
                        .unwrap_or(&case.dependency.requirement),
                    "{}",
                    case.kind
                );
                let expected_document =
                    case.resolution.expected_text.clone().unwrap_or_else(|| {
                        apply_edits(
                            &input.text,
                            &[TextEdit {
                                range: dependency.requirement_range,
                                new_text: expected.to_owned(),
                            }],
                        )
                    });
                assert_eq!(
                    apply_edits(&input.text, edits),
                    expected_document,
                    "{} final document",
                    case.kind
                );
            }
            None => assert!(edits.is_empty(), "{} emitted an unsafe edit", case.kind),
        }
    }

    fn assert_authoritative_case_coverage(cases: &[ManifestCase]) {
        let expected = parser_dispatch_kind_names();
        let actual = cases
            .iter()
            .map(|case| case.kind.clone())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual.len(),
            cases.len(),
            "manifest coverage kinds must be unique"
        );
        assert_eq!(
            actual, expected,
            "coverage must follow the exhaustive parser/model dispatch set"
        );
    }

    fn assert_editor_selector_coverage(cases: &[ManifestCase]) {
        let matrix: serde_json::Value = serde_json::from_str(include_str!(
            "../../../tests/fixtures/manifest-support-matrix.json"
        ))
        .unwrap();
        let selected = matrix["entries"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|entry| entry["manifests"].as_array().unwrap())
            .flat_map(|manifest| manifest["manifestKinds"].as_array().unwrap())
            .filter_map(serde_json::Value::as_str)
            .collect::<BTreeSet<_>>();
        for case in cases {
            assert!(
                selected.contains(case.kind.as_str()),
                "{} has no editor selector",
                case.kind
            );
            let has_language = matrix["entries"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|entry| entry["manifests"].as_array().unwrap())
                .filter(|manifest| {
                    manifest["manifestKinds"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|kind| kind.as_str() == Some(case.kind.as_str()))
                })
                .flat_map(|manifest| manifest["languages"].as_array().unwrap())
                .any(|language| language.as_str() == Some(case.language.as_str()));
            assert!(
                has_language,
                "{} has no {} editor selector",
                case.kind, case.language
            );
        }
    }
}
mod classify;
mod commands;
mod dependencies;
mod discovery;
mod documents;
pub(crate) mod operation;
mod resolution;
mod storage;
mod tasks;
mod workspace;

pub use authorization::{AuthorizationFuture, AuthorizationSource};
pub use checking::{
    WorkspaceCheckEvent, WorkspaceCheckResult, WorkspaceChecking, WorkspaceCheckingOptions,
};
pub(crate) use commands::ApplyCommandRequest;
pub(crate) use discovery::WorkspaceDiscovery;
#[cfg(test)]
pub(crate) use discovery::WorkspaceDiscoveryFileSize;
pub use discovery::{
    WorkspaceDiscoveryFailure, WorkspaceDiscoveryFailureKind, WorkspaceDiscoveryOptions,
    workspace_exclusion_matches, workspace_exclusion_path,
};
pub(crate) use storage::CacheScope;
pub use tasks::{CHECKING_CANCELLED, SessionTask};

#[derive(Debug, Clone)]
/// One engine. Cloning it shares the caches.
pub struct VersionLensSession {
    pub(crate) config: SessionConfig,
    pub(crate) storage_state: SessionStorage,
    pub(crate) request_state: SessionRequests,
    pub(crate) result_state: SessionResults,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionStorage {
    pub(crate) persistent_cache: Option<Arc<versionlens_fetch::PersistentCache>>,
    pub(crate) observed_persistent_epoch: Arc<Mutex<Option<u64>>>,
    pub(crate) cache_epoch: Arc<AtomicU64>,
    pub(crate) task_epoch: Option<u64>,
    pub(crate) task_cancellation: Option<Arc<std::sync::atomic::AtomicBool>>,
    pub(crate) task_priority: Priority,
    pub(crate) workspace: Arc<Mutex<workspace::WorkspaceState>>,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionRequests {
    pub(crate) request_body_cache: Arc<Mutex<MemoryCache<Arc<str>>>>,
    pub(crate) request_locks: Arc<Mutex<HashMap<CacheKey, Weak<tokio::sync::Mutex<()>>>>>,
    pub(crate) request_context_hashers: [RandomState; 2],
    pub(crate) dotnet_registry_sources: Arc<Mutex<Option<Vec<String>>>>,
    pub(crate) credentials: versionlens_fetch::CredentialChain,
    pub(crate) authorization: authorization::AuthorizationHook,
}

#[derive(Debug, Clone)]
pub(crate) struct SessionResults {
    pub(crate) parsed_dependencies: Arc<Mutex<MemoryCache<Vec<Dependency>>>>,
    pub(crate) latest: Arc<Mutex<MemoryCache<CachedLatest>>>,
    pub(crate) suggestions: Arc<Mutex<MemoryCache<Suggestion>>>,
    pub(crate) vulnerabilities: Arc<Mutex<MemoryCache<VulnerabilityCheck>>>,
}

impl VersionLensSession {
    /// The session's credential sources. A server fills in the client-supplied
    /// credential here.
    pub fn credentials(&self) -> &versionlens_fetch::CredentialChain {
        &self.request_state.credentials
    }

    /// Stops the resolutions in flight. Their results are dropped.
    pub fn cancel_pending_resolutions(&self) {
        self.storage_state
            .cache_epoch
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }

    /// Creates a session from `config`.
    pub fn new(config: SessionConfig) -> Self {
        version_lens_session(config)
    }

    pub(crate) fn provider_enabled_for_manifest(
        &self,
        kind: ManifestKind,
        ecosystem: Ecosystem,
    ) -> bool {
        self.config.enabled_providers.is_empty()
            || self
                .config
                .enabled_providers
                .iter()
                .any(|provider| provider.applies_to_manifest(kind, ecosystem))
    }
}

/// Creates a session from `config`.
pub fn version_lens_session(config: SessionConfig) -> VersionLensSession {
    let config = SessionConfig {
        suggestion_indicators: config
            .suggestion_indicators
            .with_standard_indicators_for_blanks(),
        ..config
    };
    let cache_ttl = Duration::from_millis(config.cache_ttl_ms);
    VersionLensSession {
        config,
        storage_state: SessionStorage {
            persistent_cache: None,
            observed_persistent_epoch: Arc::new(Mutex::new(None)),
            cache_epoch: Arc::new(AtomicU64::new(0)),
            task_cancellation: None,
            task_epoch: None,
            task_priority: Priority::Interactive,
            workspace: Arc::new(Mutex::new(workspace::WorkspaceState::default())),
        },
        request_state: SessionRequests {
            request_body_cache: Arc::new(Mutex::new(
                MemoryCache::new(cache_ttl)
                    .with_byte_capacity(32 * 1024 * 1024, |body: &Arc<str>| body.len()),
            )),
            request_locks: Arc::new(Mutex::new(HashMap::default())),
            request_context_hashers: [<RandomState>::new(), <RandomState>::new()],
            dotnet_registry_sources: Arc::new(Mutex::new(None)),
            credentials: versionlens_fetch::CredentialChain::new(),
            authorization: authorization::AuthorizationHook::default(),
        },
        result_state: SessionResults {
            parsed_dependencies: Arc::new(Mutex::new(
                MemoryCache::new(Duration::from_secs(300))
                    .with_capacity(64)
                    .with_byte_capacity(16 * 1024 * 1024, cache::weights::dependencies),
            )),
            latest: Arc::new(Mutex::new(
                MemoryCache::new(cache_ttl)
                    .with_byte_capacity(16 * 1024 * 1024, cache::weights::latest),
            )),
            suggestions: Arc::new(Mutex::new(
                MemoryCache::new(cache_ttl)
                    .with_byte_capacity(8 * 1024 * 1024, cache::weights::suggestion),
            )),
            vulnerabilities: Arc::new(Mutex::new(
                MemoryCache::new(cache_ttl)
                    .with_byte_capacity(8 * 1024 * 1024, cache::weights::vulnerability),
            )),
        },
    }
}

fn cancelled_resolution(suggestions: Vec<Suggestion>) -> ResolveDocumentOutput {
    let suggestions = suggestions
        .into_iter()
        .map(|suggestion| {
            versionlens_update::error(suggestion.dependency, "checking cancelled".to_owned())
        })
        .collect();
    finish_resolve_output(suggestions, resolve_output_parts(vec![], 0, vec![], 0))
}

pub(crate) fn finish_resolve_output(
    suggestions: Vec<Suggestion>,
    mut parts: ResolveDocumentOutputParts,
) -> ResolveDocumentOutput {
    parts.suggestions = into_suggestion_views(suggestions);
    resolve_document_output(parts)
}

pub(crate) fn resolve_output_parts(
    edits: Vec<TextEdit>,
    authorization_required_count: u32,
    authorization_required_requests: Vec<AuthorizationRequestPayload>,
    vulnerable_update_count: u32,
) -> ResolveDocumentOutputParts {
    ResolveDocumentOutputParts {
        suggestions: Vec::new(),
        edits,
        authorization_required_count,
        authorization_required_requests,
        vulnerable_update_count,
        vulnerable_update_package: None,
        vulnerable_update_version: None,
        edit_plan: None,
    }
}

pub(crate) fn resolve_output_parts_with_plan(
    input: &DocumentInput,
    edits: &[TextEdit],
    authorization_required_count: u32,
    authorization_required_requests: Vec<AuthorizationRequestPayload>,
    vulnerable_update_count: u32,
) -> ResolveDocumentOutputParts {
    let mut parts = resolve_output_parts(
        edits.to_owned(),
        authorization_required_count,
        authorization_required_requests,
        vulnerable_update_count,
    );
    parts.edit_plan = contract::document_edit_plan(input, edits);
    if parts.edit_plan.is_none() {
        // A raw edit without its snapshot is unsafe; fail closed.
        parts.edits.clear();
    }
    parts
}
