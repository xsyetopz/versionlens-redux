use serde_json::Value;

use super::{RuntimeSource, versions::go_releases};

impl RuntimeSource {
    pub fn preferred_release(
        &self,
        body: &str,
        requirement: &str,
    ) -> Result<Option<String>, String> {
        if matches!(self, Self::Go) && matches!(requirement, "stable" | "oldstable") {
            let mut releases = go_releases(body)?
                .into_iter()
                .filter_map(|(version, stable)| {
                    stable.then(|| {
                        semver::Version::parse(&version)
                            .ok()
                            .map(|parsed| (parsed, version))
                    })?
                })
                .collect::<Vec<_>>();
            releases.sort_unstable_by(|left, right| right.0.cmp(&left.0));
            let latest = releases
                .first()
                .ok_or_else(|| "Go release response has no stable release".to_owned())?;
            if requirement == "stable" {
                return Ok(Some(latest.1.clone()));
            }
            let latest_family = (latest.0.major, latest.0.minor);
            return releases
                .into_iter()
                .find(|(version, _)| (version.major, version.minor) != latest_family)
                .map(|(_, version)| Some(version))
                .ok_or_else(|| "Go release response has no oldstable release".to_owned());
        }
        if let Self::DenoChannel(_) = self {
            return self
                .versions(body, requirement)
                .map(|versions| versions.into_iter().next());
        }
        let (channel_field, versions_field, label) = match self {
            Self::PackageManager(_) => ("dist-tags", "versions", "package-manager"),
            Self::YarnModern => ("aliases", "tags", "Yarn"),
            _ => return Ok(None),
        };
        let metadata: Value = serde_json::from_str(body)
            .map_err(|error| format!("invalid {label} metadata: {error}"))?;
        let channels = &metadata[channel_field];
        let versions = &metadata[versions_field];
        let tag = if self.is_channel(requirement) {
            requirement
        } else {
            "latest"
        };
        let version = channels[tag]
            .as_str()
            .ok_or_else(|| format!("{label} release channel {tag} was not returned"))?;
        if semver::Version::parse(version).is_err()
            || match versions {
                Value::Object(versions) => !versions.contains_key(version),
                Value::Array(versions) => !versions.iter().any(|candidate| candidate == version),
                _ => true,
            }
        {
            return Err(format!(
                "{label} release channel {tag} has no valid published version"
            ));
        }
        Ok(Some(version.to_owned()))
    }
}
