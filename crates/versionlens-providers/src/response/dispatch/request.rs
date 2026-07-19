use versionlens_model::Ecosystem;

pub(super) struct ResponseRequest<'a> {
    pub(super) package: &'a str,
    pub(super) requirement: &'a str,
    pub(super) include_prereleases: bool,
    pub(super) prerelease_tags: &'a [String],
}

pub struct LatestVersionRequest<'a> {
    pub ecosystem: Ecosystem,
    pub package: &'a str,
    pub requirement: &'a str,
    pub body: &'a str,
    pub include_prereleases: bool,
    pub prerelease_tags: &'a [String],
}
