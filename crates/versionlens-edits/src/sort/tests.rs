use std::cmp::Reverse;
use std::fs::read_to_string;
use std::path::PathBuf;
use versionlens_model::{Dependency, Ecosystem};
use versionlens_model::{Position, Range, TextEdit};

use super::{can_sort_dependencies, sort_dependency_edits};
use versionlens_model::Ecosystem::{Cargo, Composer, Deno, Go, Maven, Npm, Pub, Python, Ruby};

include!("tests/fundamentals.rs");
include!("tests/manifests.rs");
include!("tests/registries.rs");
include!("tests/unsupported.rs");
fn apply_same_line_edits(text: &str, edits: &[TextEdit]) -> String {
    let mut output = text.to_owned();
    let mut ordered = edits.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|edit| Reverse(edit.range.start.character));
    for edit in ordered {
        assert_eq!(edit.range.start.line, 0);
        assert_eq!(edit.range.end.line, 0);
        let start = usize::try_from(edit.range.start.character).unwrap();
        let end = usize::try_from(edit.range.end.character).unwrap();
        output.replace_range(start..end, &edit.new_text);
    }
    output
}

fn package_file_fixture(name: &str) -> &'static str {
    let path = repo_root()
        .join("tests/fixtures/versionlens-edits/src/sort/tests")
        .join(name);
    let contents = read_to_string(&path).unwrap_or_else(|error| {
        panic!(
            "failed to read package-file fixture {}: {error}",
            path.display()
        )
    });
    crate::leaked_string(contents)
}

fn repo_root() -> PathBuf {
    let manifest_dir: PathBuf = env!("CARGO_MANIFEST_DIR").into();
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .expect("crate should be under crates/")
        .to_path_buf()
}
