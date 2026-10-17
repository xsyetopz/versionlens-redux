use versionlens_model::latest_version_with_prerelease_tags;

pub(crate) fn latest_cran_version(
    body: &str,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        cran_package_versions(body, package),
        include_prereleases,
        prerelease_tags,
    )
}

pub(crate) fn cran_release_versions(body: &str, package: &str) -> Vec<String> {
    cran_package_versions(body, package)
        .map(|value| value.to_owned())
        .collect()
}

fn cran_package_versions<'a>(body: &'a str, package: &'a str) -> impl Iterator<Item = &'a str> {
    body.split("\n\n").filter_map(move |record| {
        (record_field(record, "Package")? == package).then(|| record_field(record, "Version"))?
    })
}

fn record_field<'a>(record: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}:");
    record
        .lines()
        .find_map(|line| line.strip_prefix(&prefix).map(|value| value.trim()))
}
