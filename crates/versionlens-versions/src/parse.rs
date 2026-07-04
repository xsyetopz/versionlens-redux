use semver::Version;

mod coerce;
mod normalize;
mod requirement;

use coerce::coerced_version;
use normalize::normalize_version;

pub(crate) use requirement::{normalize_requirement, strip_version_prefix};

pub(crate) fn parse_version(raw: &str) -> Option<Version> {
    parse_padded_version(&normalize_version(raw))
}

pub fn normalized_version(raw: &str) -> Option<String> {
    Some(parse_version(raw)?.to_string())
}

pub(crate) fn parse_coerced_version(raw: &str) -> Option<Version> {
    parse_padded_version(coerced_version(raw)?)
}

pub(crate) fn parse_padded_version(raw: &str) -> Option<Version> {
    let version = match raw.split('.').count() {
        1 => format!("{raw}.0.0"),
        2 => format!("{raw}.0"),
        _ => raw.to_owned(),
    };
    Version::parse(&version).ok()
}
