use versionlens_model::Ecosystem;

pub(super) struct ResponseRequest<'a> {
    pub(super) package: &'a str,
    pub(super) requirement: &'a str,
    pub(super) include_prereleases: bool,
    pub(super) prerelease_tags: &'a [String],
}

/// A request for the latest version found in a registry response body.
#[derive(Debug)]
pub struct LatestVersionRequest<'a> {
    /// The ecosystem whose response format the body uses.
    pub ecosystem: Ecosystem,
    /// The package the body describes.
    pub package: &'a str,
    /// The version requirement the result is chosen for.
    pub requirement: &'a str,
    /// The response body.
    pub body: &'a str,
    /// Whether prerelease versions can be chosen.
    pub include_prereleases: bool,
    /// Prerelease tags that are accepted when prereleases are excluded.
    pub prerelease_tags: &'a [String],
}
