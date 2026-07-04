mod range;
mod replacement;
mod sort;
mod update;

pub use sort::{can_sort_dependencies, sort_dependency_edits};
pub use update::{bulk_update_edits, update_edits};
