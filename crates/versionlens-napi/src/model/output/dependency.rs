use napi_derive::napi;
use versionlens_vscode_model::DependencyPayload;

use crate::model::position::{NativeRange, native_range_from_core};

#[napi(object)]
pub struct NativeDependency {
    pub name: String,
    pub requirement: String,
    pub ecosystem: String,
    pub group: String,
    pub hosted_url: Option<String>,
    pub hosted_name: Option<String>,
    pub range: NativeRange,
    pub requirement_range: NativeRange,
}

impl NativeDependency {
    pub(super) fn from_core(dependency: DependencyPayload) -> Self {
        Self {
            name: dependency.name,
            requirement: dependency.requirement,
            ecosystem: dependency.ecosystem,
            group: dependency.group,
            hosted_url: dependency.hosted_url,
            hosted_name: dependency.hosted_name,
            range: native_range_from_core(dependency.range),
            requirement_range: native_range_from_core(dependency.requirement_range),
        }
    }
}

impl From<DependencyPayload> for NativeDependency {
    fn from(value: DependencyPayload) -> Self {
        Self::from_core(value)
    }
}
