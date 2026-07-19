use serde_json::to_string as to_json_string;
use std::fs::read_dir;
use std::io::ErrorKind::NotFound as IoNotFound;
use std::path::{Path, PathBuf};

use anyhow::Context;
use versionlens_model::Dependency;

use crate::error::FetchError;

use versionlens_model::Ecosystem::Dotnet;

type LocalDotnetVersions = Vec<String>;

pub(super) fn local_dotnet_package_body(
    dependency: &Dependency,
    url: &str,
) -> Result<Option<String>, FetchError> {
    let Some(source) = local_dotnet_source_path(dependency, url) else {
        return Ok(None);
    };

    let versions = local_dotnet_versions(&source, &dependency.name)?;
    if versions.is_empty() {
        return Ok(None);
    }

    let versions = to_json_string(&versions).context("failed to encode NuGet versions")?;
    Ok(Some(format!(r#"{{"versions":{versions}}}"#)))
}

fn local_dotnet_source_path(dependency: &Dependency, url: &str) -> Option<PathBuf> {
    if dependency.ecosystem != Dotnet || remote_url(url) {
        return None;
    }

    url.strip_prefix("file://")
        .map(|value| value.into())
        .or_else(|| Some(url.into()))
}

fn remote_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn local_dotnet_versions(source: &Path, name: &str) -> Result<LocalDotnetVersions, FetchError> {
    let mut versions = vec![];
    collect_hierarchical_versions(&source.join(name.to_lowercase()), &mut versions)?;
    collect_hierarchical_versions(&source.join(name), &mut versions)?;
    collect_flat_package_versions(source, name, &mut versions)?;
    Ok(versions)
}

fn collect_hierarchical_versions(
    package_dir: &Path,
    versions: &mut LocalDotnetVersions,
) -> Result<(), FetchError> {
    let entries = match read_dir(package_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == IoNotFound => return Ok(()),
        Err(error) => {
            return Err(crate::anyhow_error(error)
                .context(format!(
                    "failed to read NuGet package directory {}",
                    package_dir.display()
                ))
                .into());
        }
    };

    for entry in entries {
        let entry = entry.map_err(crate::anyhow_error)?;
        if entry.file_type().map_err(crate::anyhow_error)?.is_dir()
            && let Some(version) = entry.file_name().to_str()
        {
            push_unique_version(versions, version);
        }
    }
    Ok(())
}

fn collect_flat_package_versions(
    source: &Path,
    name: &str,
    versions: &mut LocalDotnetVersions,
) -> Result<(), FetchError> {
    let entries = match read_dir(source) {
        Ok(entries) => entries,
        Err(error) if error.kind() == IoNotFound => return Ok(()),
        Err(error) => {
            return Err(crate::anyhow_error(error)
                .context(format!(
                    "failed to read NuGet source directory {}",
                    source.display()
                ))
                .into());
        }
    };
    let prefix = format!("{}.", name.to_lowercase());

    for entry in entries {
        let entry = entry.map_err(crate::anyhow_error)?;
        let file_name = entry.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        let normalized = file_name.to_lowercase();
        if let Some(version) = normalized
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix(".nupkg"))
        {
            push_unique_version(versions, version);
        }
    }
    Ok(())
}

fn push_unique_version(versions: &mut LocalDotnetVersions, version: &str) {
    if !versions.iter().any(|known| known == version) {
        versions.push(version.to_owned());
    }
}
