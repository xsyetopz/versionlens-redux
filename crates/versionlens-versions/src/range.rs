mod compare;
mod dotnet;
mod requirements;
mod update;

pub use compare::{compare_versions, is_newer};
pub use dotnet::is_dotnet_requirement_parseable;
pub use requirements::requirement_has_empty_comparator_intersection;
pub use update::{
    build_variants, is_build_update, is_update_available, requirement_is_parseable,
    requirement_satisfies_latest, update_level,
};

#[cfg(test)]
mod tests;
