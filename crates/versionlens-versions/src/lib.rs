mod latest;
mod model;
mod parse;
mod project;
mod range;

pub use latest::{latest_stable, latest_version, latest_version_with_prerelease_tags};
pub use model::{ProjectVersionBump, UpdateLevel};
pub use parse::normalized_version;
pub use project::{is_prerelease_project_version, next_project_version};
pub use range::{
    build_variants, compare_versions, is_build_update, is_dotnet_requirement_parseable, is_newer,
    is_update_available, requirement_has_empty_comparator_intersection, requirement_is_parseable,
    requirement_satisfies_latest, update_level,
};
