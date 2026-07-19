mod latest;
mod parse;
mod pep440;
mod policy;
mod project;
mod range;
mod support;

pub use latest::{
    latest_stable, latest_version, latest_version_for_dialect, latest_version_with_prerelease_tags,
};
pub use parse::{normalized_version, normalized_version_for_dialect, strip_version_prefix};
pub use policy::{ProjectVersionBump, UpdateLevel, VersionDialect};
pub use project::{is_prerelease_project_version, next_project_version};
pub use range::{
    build_variants, compare_versions, compare_versions_for_dialect, is_build_update,
    is_dotnet_requirement_parseable, is_newer, is_update_available,
    is_update_available_for_dialect, requirement_has_empty_comparator_intersection,
    requirement_is_parseable, requirement_is_parseable_for_dialect, requirement_satisfies_latest,
    requirement_satisfies_latest_for_dialect, update_level,
};
pub(crate) use support::{parse_semver, parse_semver_req, semver_version};
