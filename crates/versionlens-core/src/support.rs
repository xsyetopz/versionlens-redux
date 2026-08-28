use anyhow::Error as AnyhowError;
use semver::{Error as SemverError, Version as SemverVersion, VersionReq as SemverVersionReq};
use std::error::Error as StdError;
use std::path::Path as StdPath;
use std::string::FromUtf8Error as StringFromUtf8Error;
use std::sync::Arc as StdArc;
use std::sync::{Mutex as StdMutex, PoisonError as SyncPoisonError};
use std::time::Duration as StdDuration;
use versionlens_cache::MemoryCache;

pub(crate) fn default<T: Default>() -> T {
    <T as Default>::default()
}

pub(crate) fn parse_semver(value: &str) -> Result<SemverVersion, SemverError> {
    value.parse()
}

pub(crate) fn recover_poison<T>(poisoned: SyncPoisonError<T>) -> T {
    poisoned.into_inner()
}

pub(crate) const fn duration_from_millis(milliseconds: u64) -> StdDuration {
    StdDuration::from_millis(milliseconds)
}

pub(crate) fn path(value: &str) -> &StdPath {
    value.as_ref()
}

pub(crate) fn mutex<T>(value: T) -> StdMutex<T> {
    StdMutex::new(value)
}

pub(crate) fn boxed<T>(value: T) -> Box<T> {
    <Box<_>>::new(value)
}

pub(crate) fn arc<T>(value: T) -> StdArc<T> {
    StdArc::new(value)
}

pub(crate) fn memory_cache<T>(ttl: StdDuration) -> MemoryCache<T> {
    <MemoryCache<T>>::new(ttl)
}

pub(crate) fn anyhow_error<E>(error: E) -> AnyhowError
where
    E: StdError + Send + Sync + 'static,
{
    error.into()
}

pub(crate) fn string_from_utf8(value: Vec<u8>) -> Result<String, StringFromUtf8Error> {
    value.try_into()
}

pub(crate) fn parse_semver_req(value: &str) -> Result<SemverVersionReq, SemverError> {
    value.parse()
}

#[cfg(test)]
pub(crate) mod tests;
