use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value as JsonValue;

use self::pattern::WorkspacePattern;

mod pattern;

const PACKAGE_MANIFEST: &str = "package.json";
const CARGO_MANIFEST: &str = "Cargo.toml";
const IGNORED_DIRECTORIES: &[&str] = &[".git", "node_modules", "target", "vendor", "build", "dist"];

/// Finds manifests selected by workspace declarations below a canonical root.
///
/// `read` is the source of truth for file contents and may provide unsaved
/// overlays. `overlays` lets a matching manifest participate before it exists
/// on disk; paths outside `root` are always rejected. A member path, file or
/// directory, for which `skips` is true is not searched.
pub(super) fn manifests(
    root: &Path,
    read: &impl Fn(&Path) -> Option<String>,
    overlays: &[PathBuf],
    skips: &mut impl FnMut(&Path, bool) -> bool,
) -> Vec<PathBuf> {
    let mut result = BTreeSet::new();

    if let Some(package) = bounded(root, &root.join(PACKAGE_MANIFEST))
        && let Some(text) = read(&package)
    {
        result.insert(package);
        if let Some(patterns) = package_patterns(&text) {
            result.extend(expand(
                root,
                PACKAGE_MANIFEST,
                &patterns,
                read,
                overlays,
                skips,
            ));
        }
    }

    if let Some(pnpm) = bounded(root, &root.join("pnpm-workspace.yaml"))
        && let Some(text) = read(&pnpm)
        && let Some(patterns) = pnpm_patterns(&text)
    {
        result.extend(expand(
            root,
            PACKAGE_MANIFEST,
            &patterns,
            read,
            overlays,
            skips,
        ));
    }

    if let Some(cargo) = bounded(root, &root.join(CARGO_MANIFEST))
        && let Some(text) = read(&cargo)
    {
        result.insert(cargo);
        if let Some(patterns) = cargo_patterns(&text) {
            result.extend(expand(
                root,
                CARGO_MANIFEST,
                &patterns,
                read,
                overlays,
                skips,
            ));
        }
    }

    result.into_iter().collect()
}

#[derive(Debug)]
struct Patterns {
    include: Vec<WorkspacePattern>,
    exclude: Vec<WorkspacePattern>,
}

struct DiskCandidateSearch<'a> {
    root: &'a Path,
    manifest_name: &'a str,
    patterns: &'a Patterns,
    explicitly_named_ignored: BTreeSet<&'static str>,
}

impl DiskCandidateSearch<'_> {
    fn ignores_directory(&self, name: &str) -> bool {
        IGNORED_DIRECTORIES.contains(&name) && !self.explicitly_named_ignored.contains(name)
    }

    fn collect(
        &self,
        directory: &Path,
        skips: &mut impl FnMut(&Path, bool) -> bool,
        result: &mut BTreeSet<PathBuf>,
    ) {
        let relative = directory.strip_prefix(self.root).unwrap_or(Path::new(""));
        if !relative.as_os_str().is_empty() && selected(self.patterns, relative) {
            let manifest = directory.join(self.manifest_name);
            if let Some(manifest) = bounded(self.root, &manifest)
                && manifest.is_file()
                && !skips(&manifest, false)
            {
                result.insert(manifest);
            }
        }

        let Ok(entries) = fs::read_dir(directory) else {
            return;
        };
        let mut directories = entries
            .flatten()
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                file_type.is_dir().then_some(entry.path())
            })
            .collect::<Vec<_>>();
        directories.sort();
        for child in directories {
            let Some(name) = child.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if self.ignores_directory(name) || skips(&child, true) {
                continue;
            }
            let Ok(relative) = child.strip_prefix(self.root) else {
                continue;
            };
            let components = path_segments(relative);
            if self
                .patterns
                .include
                .iter()
                .any(|pattern| pattern.could_match_descendant(&components))
            {
                self.collect(&child, skips, result);
            }
        }
    }
}

impl Patterns {
    fn from_signed(values: impl IntoIterator<Item = String>) -> Self {
        let mut include = Vec::new();
        let mut exclude = Vec::new();
        for value in values {
            if value.starts_with("!(") {
                if let Some(pattern) = WorkspacePattern::parse(&value) {
                    include.push(pattern);
                }
            } else if let Some(value) = value.strip_prefix('!') {
                if let Some(pattern) = WorkspacePattern::parse(value) {
                    exclude.push(pattern);
                }
            } else if let Some(pattern) = WorkspacePattern::parse(&value) {
                include.push(pattern);
            }
        }
        Self { include, exclude }
    }
}

fn package_patterns(text: &str) -> Option<Patterns> {
    let value = serde_json::from_str::<JsonValue>(text).ok()?;
    let workspaces = value.get("workspaces")?;
    let values = match workspaces {
        JsonValue::Array(values) => string_json_values(values),
        JsonValue::Object(value) => value
            .get("packages")
            .and_then(JsonValue::as_array)
            .map_or_else(Vec::new, |values| string_json_values(values)),
        _ => return None,
    };
    Some(Patterns::from_signed(values))
}

fn string_json_values(values: &[JsonValue]) -> Vec<String> {
    values
        .iter()
        .filter_map(JsonValue::as_str)
        .map(str::to_owned)
        .collect()
}

fn pnpm_patterns(text: &str) -> Option<Patterns> {
    versionlens_parsers::parse_pnpm_workspace_packages(text).map(Patterns::from_signed)
}

fn cargo_patterns(text: &str) -> Option<Patterns> {
    let workspace = versionlens_parsers::parse_cargo_workspace_members(text)?;
    let patterns = |values: Vec<String>| {
        values
            .iter()
            .filter_map(|value| WorkspacePattern::parse(value))
            .collect()
    };
    Some(Patterns {
        include: patterns(workspace.members),
        exclude: patterns(workspace.exclude),
    })
}

fn expand(
    root: &Path,
    manifest_name: &str,
    patterns: &Patterns,
    read: &impl Fn(&Path) -> Option<String>,
    overlays: &[PathBuf],
    skips: &mut impl FnMut(&Path, bool) -> bool,
) -> BTreeSet<PathBuf> {
    if patterns.include.is_empty() {
        return BTreeSet::new();
    }

    let search = DiskCandidateSearch {
        root,
        manifest_name,
        patterns,
        explicitly_named_ignored: IGNORED_DIRECTORIES
            .iter()
            .copied()
            .filter(|name| {
                patterns
                    .include
                    .iter()
                    .any(|pattern| pattern.explicitly_names(name))
            })
            .collect(),
    };
    let mut candidates = BTreeSet::new();
    search.collect(root, skips, &mut candidates);
    for overlay in overlays {
        if overlay.file_name().and_then(|name| name.to_str()) != Some(manifest_name) {
            continue;
        }
        let Some(path) = bounded(root, overlay) else {
            continue;
        };
        let Some(relative) = overlay
            .parent()
            .and_then(|parent| parent.strip_prefix(root).ok())
        else {
            continue;
        };
        let relative_segments = path_segments(relative);
        if relative_segments
            .iter()
            .any(|name| search.ignores_directory(name))
        {
            continue;
        }
        if selected(patterns, relative) && !skips(&path, false) && read(&path).is_some() {
            candidates.insert(path);
        }
    }
    candidates
}

fn selected(patterns: &Patterns, relative: &Path) -> bool {
    let path = path_segments(relative);
    patterns
        .include
        .iter()
        .any(|pattern| pattern.matches(&path))
        && !patterns
            .exclude
            .iter()
            .any(|pattern| pattern.matches(&path))
}

fn path_segments(path: &Path) -> Vec<&str> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect()
}

pub(super) fn bounded(root: &Path, path: &Path) -> Option<PathBuf> {
    let path = lexical_normalize(path);
    if !path.starts_with(root) {
        return None;
    }
    let mut ancestor = path.as_path();
    loop {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => {
                let canonical = ancestor.canonicalize().ok()?;
                if !canonical.starts_with(root) {
                    return None;
                }
                if ancestor != path && !canonical.is_dir() {
                    return None;
                }
                return if ancestor == path {
                    Some(canonical)
                } else {
                    Some(path)
                };
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ancestor = ancestor.parent()?;
            }
            Err(_) => return None,
        }
    }
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            _ => result.push(component.as_os_str()),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workspace::tests::support::TestWorkspace;
    use std::collections::BTreeMap;

    fn assert_manifests(
        workspace: &TestWorkspace,
        read: &impl Fn(&Path) -> Option<String>,
        overlays: &[PathBuf],
        expected: &[&str],
    ) {
        let actual = manifests(&workspace.root, read, overlays, &mut |_, _| false)
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
            r"catalog:
  packages:
    - ignored/**
packages:
  - 'packages/**'
  - '!packages/private/**'
",
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
            r"packages:
  - 'packages/@(app|lib-*)'
  - 'services/!(internal|legacy)'
  - 'tools/+(plugin-)core'
  - 'examples/?(demo-)site'
  - 'addons/*(experimental-)base'
  - '!packages/@(lib-private|lib-legacy)'
",
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
    fn literal_regex_characters_and_classes_match_as_globs() {
        let dotted = WorkspacePattern::parse("packages/v1.0+(x)$").unwrap();
        assert!(dotted.matches(&["packages", "v1.0xx$"]));
        assert!(!dotted.matches(&["packages", "v1a0x$"]));

        let class = WorkspacePattern::parse("packages/[a-c][!0-9]").unwrap();
        assert!(class.matches(&["packages", "bz"]));
        assert!(!class.matches(&["packages", "dz"]));
        assert!(!class.matches(&["packages", "b5"]));

        let unclosed = WorkspacePattern::parse("packages/[ab{c").unwrap();
        assert!(unclosed.matches(&["packages", "[ab{c"]));
        assert!(unclosed.explicitly_names("[ab{c"));
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
        let component = WorkspacePattern::parse(&format!("packages/{component}"))
            .expect("valid component pattern");
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

        let paths = manifests(&workspace.root, &read, &[], &mut |_, _| false);
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
}
