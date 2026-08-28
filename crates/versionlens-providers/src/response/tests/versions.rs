use super::{
    assert_latest, latest_version_for_requirement, latest_version_from_response,
    latest_version_from_response_with_prereleases, latest_version_with_tags, npm_build_versions,
    release_versions_from_response, release_versions_from_response_for_package,
};
use versionlens_model::Ecosystem::*;

include!("versions/json.rs");
include!("versions/ecosystems.rs");
include!("versions/go.rs");
include!("versions/packages.rs");
include!("versions/fallbacks.rs");
