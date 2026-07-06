use semver::{
    BuildMetadata as SemverBuildMetadata, Error as SemverError, Prerelease as SemverPrerelease,
    Version as SemverVersion, VersionReq as SemverVersionReq,
};
mod latest;
mod model;
mod parse;
mod project;
mod range;

pub use latest::{latest_stable, latest_version, latest_version_with_prerelease_tags};
pub use model::{ProjectVersionBump, UpdateLevel};
pub use parse::{normalized_version, strip_version_prefix};
pub use project::{is_prerelease_project_version, next_project_version};
pub use range::{
    build_variants, compare_versions, is_build_update, is_dotnet_requirement_parseable, is_newer,
    is_update_available, requirement_has_empty_comparator_intersection, requirement_is_parseable,
    requirement_satisfies_latest, update_level,
};

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

fn empty_prerelease() -> SemverPrerelease {
    "".parse().expect("empty semver prerelease is valid")
}

fn empty_build_metadata() -> SemverBuildMetadata {
    "".parse().expect("empty semver build metadata is valid")
}

pub(crate) fn semver_version(major: u64, minor: u64, patch: u64) -> SemverVersion {
    SemverVersion {
        major,
        minor,
        patch,
        pre: empty_prerelease(),
        build: empty_build_metadata(),
    }
}

pub(crate) fn parse_semver_req(value: &str) -> Result<SemverVersionReq, SemverError> {
    value.parse()
}
