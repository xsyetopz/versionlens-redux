use semver::{Error as SemverError, Version as SemverVersion};
mod range;
mod replacement;
mod sort;
mod update;

pub use sort::{can_sort_dependencies, sort_dependency_edits};
pub use update::{bulk_update_edits, update_edits};

#[cfg(test)]
pub(crate) fn leaked_string(contents: String) -> &'static str {
    <Box<str>>::leak(contents.into_boxed_str())
}

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}
