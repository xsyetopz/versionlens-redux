use versionlens_model::latest_version_with_prerelease_tags;

pub(crate) fn latest_julia_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        body.lines().filter_map(version_header),
        include_prereleases,
        prerelease_tags,
    )
}

fn version_header(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed.strip_prefix('[')?.strip_suffix(']')
}
