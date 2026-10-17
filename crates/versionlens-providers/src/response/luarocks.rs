use versionlens_model::{latest_version_with_prerelease_tags, normalized_version};

pub(crate) fn latest_luarocks_version(
    body: &str,
    package: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let versions = luarocks_package_versions(body, package).collect::<Vec<_>>();
    latest_luarocks_rock_version(versions.iter().copied()).or_else(|| {
        latest_version_with_prerelease_tags(versions, include_prereleases, prerelease_tags)
    })
}

fn luarocks_package_versions<'a>(body: &'a str, package: &'a str) -> impl Iterator<Item = &'a str> {
    package_manifest_block(body, package)
        .into_iter()
        .flat_map(rockspec_version_keys)
}

fn package_manifest_block<'a>(body: &'a str, package: &str) -> Option<&'a str> {
    let package_key = format!(r#"["{package}"]"#);
    let key_start = body.find(&package_key)?;
    let block_start = key_start + body[key_start..].find('{')?;
    let mut depth = 0usize;
    let mut end = block_start;
    for (relative, ch) in body[block_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    end = block_start + relative;
                    break;
                }
            }
            _ => {}
        }
    }
    (end > block_start).then_some(&body[block_start..=end])
}

fn rockspec_version_keys(block: &str) -> impl Iterator<Item = &str> {
    block.lines().filter_map(|line| {
        let line = line.trim_start();
        let rest = line.strip_prefix("[\"")?;
        let end = rest.find("\"]")?;
        Some(&rest[..end])
    })
}

fn latest_luarocks_rock_version<'a>(versions: impl IntoIterator<Item = &'a str>) -> Option<String> {
    versions
        .into_iter()
        .filter_map(|raw| {
            let (base, revision) = raw.rsplit_once('-')?;
            let revision = revision.parse::<u64>().ok()?;
            let base = crate::parse_semver(&normalized_version(base)?).ok()?;
            base.pre.is_empty().then_some(((base, revision), raw))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, raw)| raw.to_owned())
}
