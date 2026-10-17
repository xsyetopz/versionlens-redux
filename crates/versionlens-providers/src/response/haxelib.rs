use versionlens_model::latest_version_with_prerelease_tags;

pub(crate) fn latest_haxelib_version(
    body: &str,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = haxelib_install_versions(body, package);
    latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
}

fn haxelib_install_versions<'a>(body: &'a str, package: &'a str) -> impl Iterator<Item = &'a str> {
    let marker = format!("haxelib install {package} ");
    body.match_indices(&marker)
        .filter_map(|(index, _)| {
            let version_start = index + marker.len();
            let version_end = body[version_start..]
                .find(|ch: char| !is_haxelib_version_character(ch))
                .map_or(body.len(), |relative| version_start + relative);
            (version_end > version_start).then_some(&body[version_start..version_end])
        })
        .collect::<Vec<_>>()
        .into_iter()
}

fn is_haxelib_version_character(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+')
}
