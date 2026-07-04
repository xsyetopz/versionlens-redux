mod cargo;
mod composer;
mod deno;
mod dub;
mod npm;
mod pub_dev;
mod python;
mod ruby;

pub(super) use cargo::{cargo_registry_url, cargo_registry_url_with_base};
pub(super) use composer::{composer_registry_url, composer_registry_url_with_base};
pub(super) use deno::{deno_registry_url, deno_registry_url_with_base};
pub(super) use dub::{dub_registry_url, dub_registry_url_with_base};
pub(super) use npm::{npm_registry_url, npm_registry_url_with_base};
pub(super) use pub_dev::{pub_registry_url, pub_registry_url_with_base};
pub use python::python_package_json_url_template;
pub(super) use python::{python_registry_url, python_registry_url_with_base};
pub(super) use ruby::{ruby_registry_url, ruby_registry_url_with_base};
