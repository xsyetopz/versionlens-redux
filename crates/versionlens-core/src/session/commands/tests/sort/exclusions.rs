use std::cmp::Reverse;
#[tokio::test]
async fn apply_command_does_not_sort_clojure_deps_edn_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///deps.edn", "clojure", "deps.edn").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_leiningen_project_clj_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///project.clj", "clojure", "project.clj").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_terraform_required_providers() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///main.tf", "terraform", "main.tf").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_helm_chart_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///Chart.yaml", "yaml", "Chart.yaml").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_ansible_requirements() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/requirements.yml", "yaml", "ansible-requirements.yml").await;

    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_bazel_modules() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/MODULE.bazel", "starlark", "MODULE.bazel").await;

    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_cocoapods_podfile_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/Podfile", "ruby", "Podfile").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_unity_project_manifest_dependencies() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/Packages/manifest.json", "json", "unity-manifest.json").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_kustomization_images() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/kustomization.yaml", "yaml", "kustomization.yaml").await;

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[tokio::test]
async fn apply_command_does_not_sort_nix_flake_inputs() {
    let session = standard_session();

    let output = sort_fixture(&session, "file:///work/flake.nix", "nix", "flake.nix").await;

    assert!(output.edits.is_empty());
}

fn apply_line_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut lines: Vec<String> = text.lines().map(|value| value.to_owned()).collect();
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| Reverse(edit.range.start.line));
    for edit in ordered {
        let start = usize::try_from(edit.range.start.line).unwrap();
        let end = usize::try_from(edit.range.end.line).unwrap();
        lines.splice(
            start..=end,
            edit.new_text.lines().map(|value| value.to_owned()),
        );
    }
    lines.join("\n")
}
