mod document;
mod ecosystem;
mod manifest;

pub use document::{Dependency, DocumentInput};
pub use ecosystem::{Ecosystem, ecosystem_config_namespace, ecosystem_from_config_name};
pub use manifest::{ManifestKind, ecosystem_for_manifest};

#[cfg(test)]
mod tests;
