mod kind;
mod payload;
pub(crate) mod properties;

pub(crate) use kind::is_npm_package_manager;
pub use payload::dependency_payload;
pub(crate) use payload::into_dependency_payloads;
