use std::fmt;

use serde::de::{IgnoredAny, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use versionlens_versions::{latest_version, latest_version_with_prerelease_tags};

#[derive(Default)]
struct CargoResponse<'a> {
    versions: Vec<CargoVersion<'a>>,
    summary: Option<CargoSummary<'a>>,
}

enum CargoVersion<'a> {
    Label(&'a str),
    Release { num: Option<&'a str>, yanked: bool },
}

impl CargoVersion<'_> {
    fn label(&self) -> Option<&str> {
        match self {
            Self::Label(version) => Some(version),
            Self::Release { num, yanked } => (!yanked).then_some(*num).flatten(),
        }
    }
}

#[derive(Deserialize)]
struct CargoSummary<'a> {
    default_version: Option<&'a str>,
    max_version: Option<&'a str>,
    max_stable_version: Option<&'a str>,
    #[serde(default)]
    yanked: bool,
}

impl<'de> Deserialize<'de> for CargoResponse<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CargoResponseVisitor)
    }
}

struct CargoResponseVisitor;

impl<'de> Visitor<'de> for CargoResponseVisitor {
    type Value = CargoResponse<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a crates.io response object or a version array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut versions = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
        while let Some(version) = sequence.next_element()? {
            versions.push(version);
        }
        Ok(CargoResponse {
            versions,
            summary: None,
        })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut response = CargoResponse::default();
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "versions" => response.versions = map.next_value()?,
                "crate" => response.summary = map.next_value()?,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(response)
    }
}

impl<'de> Deserialize<'de> for CargoVersion<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CargoVersionVisitor)
    }
}

struct CargoVersionVisitor;

impl<'de> Visitor<'de> for CargoVersionVisitor {
    type Value = CargoVersion<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a version string or crates.io release object")
    }

    fn visit_borrowed_str<E>(self, version: &'de str) -> Result<Self::Value, E> {
        Ok(CargoVersion::Label(version))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut num = None;
        let mut yanked = false;
        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "num" => num = map.next_value()?,
                "yanked" => yanked = map.next_value()?,
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(CargoVersion::Release { num, yanked })
    }
}

pub(crate) fn latest_cargo_response(
    body: &str,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let response = serde_json::from_str::<CargoResponse<'_>>(body).ok()?;
    let versions = response.versions.as_slice();
    let summary = response.summary.as_ref();

    if include_prereleases {
        return latest_version_with_prerelease_tags(
            versions.iter().filter_map(CargoVersion::label),
            true,
            prerelease_tags,
        )
        .or_else(|| latest_cargo_summary_version(summary, true, prerelease_tags));
    }

    let stable = latest_version(versions.iter().filter_map(CargoVersion::label), false);
    if summary.is_some_and(|summary| summary.default_version.is_some() || summary.yanked) {
        return stable;
    }

    stable.or_else(|| latest_cargo_summary_version(summary, false, prerelease_tags))
}

fn latest_cargo_summary_version(
    summary: Option<&CargoSummary<'_>>,
    include_prereleases: bool,
    prerelease_tags: &[String],
) -> Option<String> {
    let summary = summary?;
    if include_prereleases {
        return summary
            .max_version
            .and_then(|version| {
                latest_version_with_prerelease_tags([version], true, prerelease_tags)
            })
            .or_else(|| stable_cargo_summary_version(summary));
    }

    stable_cargo_summary_version(summary).or_else(|| {
        summary
            .max_version
            .and_then(|version| latest_version([version], false))
    })
}

fn stable_cargo_summary_version(summary: &CargoSummary<'_>) -> Option<String> {
    summary
        .max_stable_version
        .and_then(|version| latest_version([version], false))
}
