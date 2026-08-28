use std::cmp::Reverse;
use versionlens_model::{Dependency, Ecosystem, Position, Range, TextEdit};

use super::{can_sort_dependencies, sort_dependency_edits};
use crate::support::tests::range;
use versionlens_model::Ecosystem::*;

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
    crate::support::tests::fixture("tests/fixtures/versionlens-edits/src/sort/tests", name)
}
