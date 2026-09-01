use super::*;
use crate::support::tests::{file_uri, local_test_root};
use std::fs::create_dir_all;
use std::fs::remove_dir_all;
use std::fs::write;

#[test]
fn project_version_code_lenses_offer_stable_bumps() {
    let session = standard_session();
    let input = package_document("version-1.2.3.json");

    let output = session.analyze_document(input);
    let titles = lens_titles(&output);
    let commands = crate::support::tests::code_lens_commands(&output);

    assert_eq!(titles, ["U major 2.0.0", "U minor 1.3.0", "U patch 1.2.4"]);
    assert_eq!(commands, ["updateMajor", "updateMinor", "updatePatch"]);
}

#[test]
fn project_version_code_lenses_offer_prerelease_bumps() {
    let session = standard_session();
    let input = package_document("version-1.2.3-beta.4.json");

    let output = session.analyze_document(input);
    let titles = lens_titles(&output);
    let commands = crate::support::tests::code_lens_commands(&output);

    assert_eq!(titles, ["U release 1.2.3", "U prerelease 1.2.3-beta.5"]);
    assert_eq!(commands, ["updateRelease", "updatePrerelease"]);
}

#[test]
fn build_code_lens_chooses_available_build_versions() {
    let output = build_code_lens_output(
        "left-pad-1.0.0-build.1.json",
        "1.0.0+build.2",
        &[
            "1.0.0",
            "1.0.0+build.1",
            "1.0.0+build.2",
            "1.0.0+build.3",
            "1.1.0",
        ],
    );
    let titles = lens_titles(&output);
    let commands = lens_commands(&output);

    assert_eq!(titles, ["L latest 1.0.0+build.1", "B change build"]);
    assert_eq!(commands, ["", "versionlens.suggestion.onChooseBuild"]);
    assert_eq!(
        &output.code_lenses[1].arguments[1..],
        [
            "left-pad",
            "1.0.0+build.1",
            "1.0.0",
            "1.0.0+build.1",
            "1.0.0+build.2",
            "1.0.0+build.3"
        ]
    );
}

#[test]
fn build_code_lens_keeps_latest_status_when_current_has_build_versions() {
    let output = build_code_lens_output(
        "left-pad-3.0.0.json",
        "3.0.0",
        &["1.0.0", "2.0.0", "2.1.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"],
    );
    let titles = lens_titles(&output);
    let commands = lens_commands(&output);

    assert_eq!(
        titles,
        ["L latest 3.0.0", "D downgrade 2.1.0", "B change build"]
    );
    assert_eq!(
        commands,
        [
            "",
            "versionlens.suggestion.onUpdateDependency",
            "versionlens.suggestion.onChooseBuild"
        ]
    );
    assert_eq!(
        &output.code_lenses[2].arguments[1..],
        ["left-pad", "3.0.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
    );
}

#[test]
fn build_code_lens_keeps_latest_status_when_current_build_differs_from_latest_build() {
    let output = build_code_lens_output(
        "left-pad-3.0.0-b1.json",
        "3.0.0+b2",
        &["1.0.0", "2.0.0", "2.1.0", "3.0.0", "3.0.0+b1", "3.0.0+b2"],
    );
    let titles = lens_titles(&output);
    let commands = lens_commands(&output);

    assert_eq!(
        titles,
        ["L latest 3.0.0+b1", "D downgrade 2.1.0", "B change build"]
    );
    assert_eq!(
        commands,
        [
            "",
            "versionlens.suggestion.onUpdateDependency",
            "versionlens.suggestion.onChooseBuild"
        ]
    );
    assert_eq!(
        &output.code_lenses[2].arguments[1..],
        ["left-pad", "3.0.0+b1", "3.0.0", "3.0.0+b1", "3.0.0+b2"]
    );
}

#[test]
fn build_code_lens_uses_latest_build_when_variant_list_is_missing() {
    let session = standard_session();
    let input = package_document("left-pad-1.0.0-build.1.json");

    let output = crate::support::tests::analyze_with_responses(
        &session,
        &input,
        &[crate::support::tests::npm_latest_response(
            "left-pad",
            "1.0.0+build.2",
        )],
    );

    assert_eq!(output.code_lenses[0].title, "B change build");
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onChooseBuild"
    );
    assert_eq!(
        &output.code_lenses[0].arguments[1..],
        ["left-pad", "1.0.0+build.1", "1.0.0+build.2"]
    );
}

#[test]
fn directory_code_lens_opens_local_dependency_path() {
    let session = standard_session();
    let root = local_test_root("directory-codelens");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();
    let input = DocumentInput::new(
        file_uri(&app.join("package.json")),
        "json".to_owned(),
        package_file_fixture("local-file-dependency.json"),
        None,
    );

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    let local_path = local.to_string_lossy();
    assert_eq!(output.code_lenses[0].title, "D file://../local");
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onFileLink"
    );
    assert_eq!(output.code_lenses[0].arguments, [local_path.as_ref()]);
    remove_dir_all(root).unwrap();
}

#[test]
fn npm_link_code_lens_opens_package_json_target_path() {
    let session = standard_session();
    let root = local_test_root("npm-link-codelens");
    let app = root.join("app");
    let local = root.join("local");
    create_dir_all(&app).unwrap();
    create_dir_all(&local).unwrap();
    write(
        local.join("package.json"),
        package_file_fixture("empty-package.json"),
    )
    .unwrap();
    let input = DocumentInput::new(
        file_uri(&app.join("package.json")),
        "json".to_owned(),
        package_file_fixture("local-link-dependency.json"),
        None,
    );

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    let target_path = local.join("package.json");
    let target_path = target_path.to_string_lossy();
    assert_eq!(
        output.code_lenses[0].title,
        "D file://../local/package.json"
    );
    assert_eq!(
        output.code_lenses[0].command,
        "versionlens.suggestion.onFileLink"
    );
    assert_eq!(output.code_lenses[0].arguments, [target_path.as_ref()]);
    remove_dir_all(root).unwrap();
}

#[test]
fn missing_directory_code_lens_is_disabled_not_found_status() {
    let session = standard_session();
    let input = DocumentInput::new(
        "file:///repo/app/package.json".to_owned(),
        "json".to_owned(),
        package_file_fixture("local-file-dependency.json"),
        None,
    );

    session.resolve_document(input.clone());
    let output = session.analyze_document(input);

    assert_eq!(output.code_lenses[0].title, "E not found ../local");
    assert_eq!(output.code_lenses[0].command, "");
    assert!(output.code_lenses[0].arguments.is_empty());
}
