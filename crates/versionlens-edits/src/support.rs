use semver::{Error as SemverError, Version as SemverVersion};

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

#[cfg(test)]
pub(crate) mod tests;
