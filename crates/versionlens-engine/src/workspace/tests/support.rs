use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use versionlens_model::{Dependency, DocumentInput, Ecosystem, Position, Range};

pub(crate) struct TestWorkspace {
    pub(crate) root: PathBuf,
}

impl TestWorkspace {
    pub(crate) fn new(label: &str) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "versionlens-{label}-{}-{serial}",
            std::process::id()
        ));
        fs::create_dir(&root).unwrap();
        Self {
            root: root.canonicalize().unwrap(),
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.root
    }

    pub(crate) fn write(&self, relative: &str, text: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        path
    }

    pub(crate) fn write_all(&self, files: &[(&str, &str)]) {
        for (relative, text) in files {
            self.write(relative, text);
        }
    }

    pub(crate) fn npm() -> Self {
        let workspace = Self::new("workspace");
        workspace.write_all(&[
            ("package.json", r#"{"workspaces":["packages/*"]}"#),
            (
                "packages/a/package.json",
                r#"{"name":"a","version":"1.0.0"}"#,
            ),
        ]);
        workspace
    }

    pub(crate) fn document(&self, relative: &str, text: &str, version: u64) -> DocumentInput {
        let language_id = Path::new(relative)
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("json");
        DocumentInput::new(
            self.root.join(relative).to_string_lossy(),
            language_id,
            text,
            Some(self.root.to_string_lossy().into_owned()),
        )
        .with_version(version)
    }

    pub(crate) fn read(path: &Path) -> Option<String> {
        fs::read_to_string(path).ok()
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn dependency(name: &str, requirement: &str) -> Dependency {
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 0,
            character: 1,
        },
    };
    Dependency {
        name: name.to_owned(),
        requirement: requirement.to_owned(),
        ecosystem: Ecosystem::Npm,
        group: "dependencies".to_owned(),
        hosted_url: None,
        hosted_name: None,
        range,
        requirement_range: range,
        requirement_prefix: String::new(),
        requirement_suffix: String::new(),
        canonical_reference: None,
    }
}
