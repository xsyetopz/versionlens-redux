use super::*;
use crate::workspace::tests::support::TestWorkspace;
use std::collections::BTreeMap;

fn assert_manifests(
    workspace: &TestWorkspace,
    read: &impl Fn(&Path) -> Option<String>,
    overlays: &[PathBuf],
    expected: &[&str],
) {
    let actual = manifests(&workspace.root, read, overlays)
        .into_iter()
        .map(|path| path.strip_prefix(&workspace.root).unwrap().to_path_buf())
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        expected.iter().map(PathBuf::from).collect::<Vec<_>>()
    );
}

fn assert_disk_manifests(workspace: &TestWorkspace, expected: &[&str]) {
    assert_manifests(workspace, &TestWorkspace::read, &[], expected);
}

fn assert_single_overlay_manifests(
    workspace: &TestWorkspace,
    overlay: &PathBuf,
    expected: &[&str],
) {
    let read = |path: &Path| {
        if path == overlay {
            Some("{}".to_owned())
        } else {
            fs::read_to_string(path).ok()
        }
    };
    assert_manifests(workspace, &read, std::slice::from_ref(overlay), expected);
}

fn write_empty_manifests(workspace: &TestWorkspace, paths: &[&str]) {
    for path in paths {
        workspace.write(path, "{}");
    }
}

fn assert_pnpm_manifests(
    workspace: &TestWorkspace,
    document: &str,
    candidates: &[&str],
    expected: &[&str],
) {
    workspace.write("pnpm-workspace.yaml", document);
    write_empty_manifests(workspace, candidates);
    assert_disk_manifests(workspace, expected);
}

#[test]
fn package_workspace_object_uses_structural_and_segment_glob_matching() {
    let workspace = TestWorkspace::new("discovery");
    workspace.write(
        PACKAGE_MANIFEST,
        r#"{
          "description": "workspaces: packages/deep/*",
          "workspaces": { "packages": ["packages/{a,b?}"] }
        }"#,
    );
    write_empty_manifests(
        &workspace,
        &[
            "packages/a/package.json",
            "packages/b1/package.json",
            "packages/b12/package.json",
            "packages/deep/member/package.json",
        ],
    );
    assert_disk_manifests(
        &workspace,
        &[
            "package.json",
            "packages/a/package.json",
            "packages/b1/package.json",
        ],
    );
}

#[test]
fn pnpm_yaml_reads_only_packages_and_applies_exclusions() {
    let workspace = TestWorkspace::new("discovery");
    assert_pnpm_manifests(
        &workspace,
        r#"catalog:
  packages:
    - ignored/**
packages:
  - 'packages/**'
  - '!packages/private/**'
"#,
        &[
            "packages/a/package.json",
            "packages/group/b/package.json",
            "packages/private/c/package.json",
            "ignored/x/package.json",
            "packages/node_modules/hidden/package.json",
        ],
        &["packages/a/package.json", "packages/group/b/package.json"],
    );
}

#[test]
fn pnpm_yaml_supports_extglobs_and_extglob_exclusions() {
    let workspace = TestWorkspace::new("pnpm-extglobs");
    assert_pnpm_manifests(
        &workspace,
        r#"packages:
  - 'packages/@(app|lib-*)'
  - 'services/!(internal|legacy)'
  - 'tools/+(plugin-)core'
  - 'examples/?(demo-)site'
  - 'addons/*(experimental-)base'
  - '!packages/@(lib-private|lib-legacy)'
"#,
        &[
            "packages/app/package.json",
            "packages/lib-core/package.json",
            "packages/lib-private/package.json",
            "packages/docs/package.json",
            "services/public/package.json",
            "services/internal/package.json",
            "tools/core/package.json",
            "tools/plugin-core/package.json",
            "tools/plugin-plugin-core/package.json",
            "examples/site/package.json",
            "examples/demo-site/package.json",
            "examples/demo-demo-site/package.json",
            "addons/base/package.json",
            "addons/experimental-base/package.json",
            "addons/experimental-experimental-base/package.json",
        ],
        &[
            "addons/base/package.json",
            "addons/experimental-base/package.json",
            "addons/experimental-experimental-base/package.json",
            "examples/demo-site/package.json",
            "examples/site/package.json",
            "packages/app/package.json",
            "packages/lib-core/package.json",
            "services/public/package.json",
            "tools/plugin-core/package.json",
            "tools/plugin-plugin-core/package.json",
        ],
    );
}

#[test]
fn pnpm_yaml_treats_a_leading_negative_extglob_as_an_include() {
    let workspace = TestWorkspace::new("pnpm-leading-negative-extglob");
    assert_pnpm_manifests(
        &workspace,
        "packages:\n  - '!(private|legacy)/**'\n",
        &[
            "private/app/package.json",
            "legacy/app/package.json",
            "public/app/package.json",
        ],
        &["public/app/package.json"],
    );
}

#[test]
fn unsupported_negative_extglob_forms_are_rejected() {
    assert!(WorkspacePattern::parse("packages/pre-!(private)").is_none());
    assert!(WorkspacePattern::parse("packages/!(!(private))").is_none());
}

#[test]
fn cargo_toml_supports_multiline_members_globs_and_exclude() {
    let workspace = TestWorkspace::new("discovery");
    workspace.write(
        CARGO_MANIFEST,
        r#"[package]
name = "root"

[workspace]
members = [
  "crates/*",
  "tools/**",
]
exclude = ["crates/skip", "tools/private/**"]

[package.metadata]
members = ["unrelated"]
"#,
    );
    workspace.write_all(&[
        ("crates/a/Cargo.toml", "[package]\nname = 'a'"),
        ("crates/skip/Cargo.toml", "[package]\nname = 'skip'"),
        ("crates/deep/b/Cargo.toml", "[package]\nname = 'b'"),
        ("tools/group/c/Cargo.toml", "[package]\nname = 'c'"),
        ("tools/private/d/Cargo.toml", "[package]\nname = 'd'"),
        ("unrelated/Cargo.toml", "[package]\nname = 'unrelated'"),
    ]);
    assert_disk_manifests(
        &workspace,
        &[
            "Cargo.toml",
            "crates/a/Cargo.toml",
            "tools/group/c/Cargo.toml",
        ],
    );
}

#[test]
fn unsaved_root_and_new_member_overlays_drive_discovery() {
    let workspace = TestWorkspace::new("discovery");
    workspace.write(PACKAGE_MANIFEST, r#"{"workspaces":["disk/*"]}"#);
    workspace.write("disk/a/package.json", "{}");
    let root_manifest = workspace.root.join(PACKAGE_MANIFEST);
    let draft_manifest = workspace.root.join("draft/new/package.json");
    let overlays = BTreeMap::from([
        (root_manifest, r#"{"workspaces":["draft/**"]}"#.to_owned()),
        (
            draft_manifest,
            r#"{"name":"new","version":"1.0.0"}"#.to_owned(),
        ),
    ]);
    let read = |path: &Path| {
        overlays
            .get(path)
            .cloned()
            .or_else(|| fs::read_to_string(path).ok())
    };

    assert_manifests(
        &workspace,
        &read,
        &overlays.keys().cloned().collect::<Vec<_>>(),
        &["draft/new/package.json", "package.json"],
    );
}

#[test]
fn parent_patterns_and_outside_overlays_cannot_escape_the_root() {
    let workspace = TestWorkspace::new("discovery");
    workspace.write(
        PACKAGE_MANIFEST,
        r#"{"workspaces":["../outside/**","packages/**"]}"#,
    );
    workspace.write("packages/a/package.json", "{}");
    let outside = workspace
        .root
        .parent()
        .unwrap()
        .join("outside/package.json");
    assert_single_overlay_manifests(
        &workspace,
        &outside,
        &["package.json", "packages/a/package.json"],
    );
}

#[cfg(unix)]
#[test]
fn recursive_globs_do_not_follow_directory_symlinks() {
    use std::os::unix::fs::symlink;

    let workspace = TestWorkspace::new("discovery");
    workspace.write(PACKAGE_MANIFEST, r#"{"workspaces":["**"]}"#);
    workspace.write("packages/a/package.json", "{}");
    symlink(&workspace.root, workspace.root.join("packages/a/loop")).unwrap();

    assert_disk_manifests(&workspace, &["package.json", "packages/a/package.json"]);
}

#[cfg(unix)]
#[test]
fn nonexistent_overlay_below_an_outside_symlink_is_rejected() {
    use std::os::unix::fs::symlink;

    let workspace = TestWorkspace::new("discovery");
    let outside = TestWorkspace::new("discovery-outside");
    workspace.write(PACKAGE_MANIFEST, r#"{"workspaces":["linked/**"]}"#);
    symlink(&outside.root, workspace.root.join("linked")).unwrap();
    let overlay = workspace.root.join("linked/new/package.json");
    assert_single_overlay_manifests(&workspace, &overlay, &["package.json"]);
}

#[test]
fn repeated_stars_and_brace_choices_match_without_cartesian_expansion() {
    let component = format!("{}z", "*a".repeat(256));
    let matching_value = format!("{}z", "a".repeat(256));
    let failing_value = format!("{}x", "a".repeat(256));
    let component =
        WorkspacePattern::parse(&format!("packages/{component}")).expect("valid component pattern");
    assert!(component.matches(&["packages", &matching_value]));
    assert!(!component.matches(&["packages", &failing_value]));

    let member = "a".repeat(24);
    let pattern = WorkspacePattern::parse(&format!("packages/{}", "{a,b}".repeat(24)))
        .expect("valid workspace pattern");
    assert!(pattern.matches(&["packages", &member]));
    assert!(!pattern.matches(&["packages", &format!("{member}c")]));

    let recursive = WorkspacePattern::parse(&format!("{}leaf", "**/".repeat(128)))
        .expect("valid recursive pattern");
    let mut path = vec!["branch"; 128];
    path.push("leaf");
    assert!(recursive.matches(&path));
}

#[test]
fn disk_member_bodies_are_not_read_during_enumeration() {
    use std::cell::RefCell;

    let workspace = TestWorkspace::new("discovery");
    workspace.write(PACKAGE_MANIFEST, r#"{"workspaces":["packages/*"]}"#);
    let member = workspace.write("packages/a/package.json", "{}");
    let reads = RefCell::new(Vec::new());
    let read = |path: &Path| {
        reads.borrow_mut().push(path.to_path_buf());
        fs::read_to_string(path).ok()
    };

    let paths = manifests(&workspace.root, &read, &[]);
    assert!(paths.contains(&member));
    assert_eq!(
        reads
            .borrow()
            .iter()
            .filter(|path| *path == &member)
            .count(),
        0
    );
}
