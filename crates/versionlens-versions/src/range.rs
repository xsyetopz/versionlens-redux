mod compare;
mod dotnet;
mod requirements;
mod update;

pub use compare::{compare_versions, compare_versions_for_dialect, is_newer};
pub use dotnet::is_dotnet_requirement_parseable;
pub use requirements::requirement_has_empty_comparator_intersection;
pub use update::{
    build_variants, is_build_update, is_update_available, is_update_available_for_dialect,
    requirement_is_parseable, requirement_is_parseable_for_dialect, requirement_satisfies_latest,
    requirement_satisfies_latest_for_dialect, update_level,
};

#[cfg(test)]
mod tests;
