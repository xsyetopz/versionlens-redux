use std::cmp::Reverse;
#[test]
fn apply_command_does_not_sort_clojure_deps_edn_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///deps.edn".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("deps.edn"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_leiningen_project_clj_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///project.clj".to_owned(),
            language_id: "clojure".to_owned(),
            text: package_file_fixture("project.clj"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_terraform_required_providers() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///main.tf".to_owned(),
            language_id: "terraform".to_owned(),
            text: package_file_fixture("main.tf"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_helm_chart_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///Chart.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("Chart.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_ansible_requirements() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/requirements.yml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("ansible-requirements.yml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_bazel_modules() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/MODULE.bazel".to_owned(),
            language_id: "starlark".to_owned(),
            text: package_file_fixture("MODULE.bazel"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_cocoapods_podfile_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/Podfile".to_owned(),
            language_id: "ruby".to_owned(),
            text: package_file_fixture("Podfile"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_unity_project_manifest_dependencies() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/Packages/manifest.json".to_owned(),
            language_id: "json".to_owned(),
            text: package_file_fixture("unity-manifest.json"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_kustomization_images() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/kustomization.yaml".to_owned(),
            language_id: "yaml".to_owned(),
            text: package_file_fixture("kustomization.yaml"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

    assert!(output.suggestions.is_empty());
    assert!(output.edits.is_empty());
}

#[test]
fn apply_command_does_not_sort_nix_flake_inputs() {
    let session = standard_session();

    let output = session.apply_command(
        DocumentInput {
            uri: "file:///work/flake.nix".to_owned(),
            language_id: "nix".to_owned(),
            text: package_file_fixture("flake.nix"),
            workspace_root: None,
        },
        Some("sort"),
        None,
        &[],
    );

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
