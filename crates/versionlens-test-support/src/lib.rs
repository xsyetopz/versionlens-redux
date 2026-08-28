//! Test-only repository fixture access shared by workspace crates.

use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FixtureError {
    pub path: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for FixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to read fixture {}: {}",
            self.path.display(),
            self.source
        )
    }
}

impl std::error::Error for FixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub fn read_fixture(manifest_dir: &str, base: &str, name: &str) -> Result<String, FixtureError> {
    let manifest_path = PathBuf::from(manifest_dir);
    let repository_root = manifest_path
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| FixtureError {
            path: manifest_path.join(base).join(name),
            source: io::Error::new(
                io::ErrorKind::InvalidInput,
                "manifest directory has no repository root",
            ),
        })?;
    let path = repository_root.join(base).join(name);
    fs::read_to_string(&path).map_err(|source| FixtureError { path, source })
}

#[macro_export]
macro_rules! fixture {
    ($base:expr, $name:expr) => {
        $crate::read_fixture(env!("CARGO_MANIFEST_DIR"), $base, $name)
    };
}

#[macro_export]
macro_rules! static_fixture {
    ($base:expr, $name:expr) => {
        $crate::fixture!($base, $name)
            .map(|value| <Box<str>>::leak(value.into_boxed_str()) as &'static str)
    };
}
