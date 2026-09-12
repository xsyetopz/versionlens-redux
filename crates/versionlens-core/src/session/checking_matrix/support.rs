use std::cmp::Reverse;
use std::collections::BTreeSet;

use versionlens_model::{Position, TextEdit};

use super::super::VersionLensSession;
use super::{CategoryCase, ManifestCase};
use crate::support::tests::session_config;

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
) -> &'a versionlens_vscode_model::SuggestionPayload {
    output
        .suggestions
        .iter()
        .find(|suggestion| suggestion.dependency.name == package)
        .unwrap_or_else(|| panic!("no suggestion for {package}: {output:?}"))
}

pub(super) fn manifest_cases() -> Vec<ManifestCase> {
    serde_json::from_str(include_str!(
        "../../../../../tests/fixtures/checking-coverage/manifest-cases.json"
    ))
    .unwrap()
}

pub(super) fn category_cases() -> Vec<CategoryCase> {
    serde_json::from_str(include_str!(
        "../../../../../tests/fixtures/checking-coverage/versionable-categories.json"
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
    VersionLensSession::new(session_config(crate::default(), false))
}

pub(super) fn parser_dispatch_kind_names() -> BTreeSet<String> {
    include_str!("../../../../versionlens-parsers/src/document/parsers.rs")
        .split("ManifestKind::")
        .skip(1)
        .map(|tail| {
            tail.chars()
                .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
                .collect::<String>()
        })
        .filter(|kind| !kind.is_empty())
        .collect()
}

pub(super) fn versionable_kind_names() -> BTreeSet<String> {
    let source = include_str!("../../../../versionlens-model/src/document.rs");
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
        .filter(|variant| !variant.is_empty())
        .map(str::to_owned)
        .collect()
}
