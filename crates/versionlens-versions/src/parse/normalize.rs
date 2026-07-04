pub(super) fn normalize_version(raw: &str) -> String {
    let version = raw
        .trim_start_matches(['v', '=', '^', '~', '>', '<'])
        .trim_start();
    normalize_pep440_prerelease(version).unwrap_or_else(|| version.to_owned())
}

fn normalize_pep440_prerelease(version: &str) -> Option<String> {
    for (marker, normalized) in [
        ("alpha", "alpha"),
        ("beta", "beta"),
        ("pre", "pre"),
        ("rc", "rc"),
        ("a", "alpha"),
        ("b", "beta"),
    ] {
        let Some(index) = version.rfind(marker) else {
            continue;
        };
        let number = &version[index + marker.len()..];
        if !number.is_empty() && number.chars().all(|char| char.is_ascii_digit()) {
            let base = version[..index].trim_end_matches(['.', '-']);
            return Some(format!("{base}-{normalized}.{number}"));
        }
    }
    None
}
