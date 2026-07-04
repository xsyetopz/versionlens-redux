use versionlens_parsers::Ecosystem;

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

impl<'a> ResponseRequest<'a> {
    pub(super) fn from_latest(request: &LatestVersionRequest<'a>) -> Self {
        Self {
            package: request.package,
            requirement: request.requirement,
            include_prereleases: request.include_prereleases,
            prerelease_tags: request.prerelease_tags,
        }
    }
}
