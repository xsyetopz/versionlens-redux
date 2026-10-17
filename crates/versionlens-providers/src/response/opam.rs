use versionlens_model::latest_version_with_prerelease_tags;

pub(crate) fn latest_opam_version(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    latest_version_with_prerelease_tags(
        body.match_indices("(latest)")
            .filter_map(|(latest_marker, _)| version_before_latest_marker(body, latest_marker)),
        include_prereleases,
        prerelease_tags,
    )
}

fn version_before_latest_marker(body: &str, latest_marker: usize) -> Option<&str> {
    let before = body.get(..latest_marker)?.trim_end();
    let end = before.len();
    let start = before
        .char_indices()
        .rev()
        .find_map(|(index, ch)| (!is_opam_version_char(ch)).then_some(index + ch.len_utf8()))
        .unwrap_or(0);
    before.get(start..end).filter(|version| !version.is_empty())
}

fn is_opam_version_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_' | '+' | '~')
}
