use std::path::{Path, PathBuf};

use anyhow::Context;
use versionlens_parsers::{Dependency, Ecosystem};

use crate::error::FetchError;

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

    let versions = serde_json::to_string(&versions).context("failed to encode NuGet versions")?;
    Ok(Some(format!(r#"{{"versions":{versions}}}"#)))
}

fn local_dotnet_source_path(dependency: &Dependency, url: &str) -> Option<PathBuf> {
    if dependency.ecosystem != Ecosystem::Dotnet || remote_url(url) {
        return None;
    }

    url.strip_prefix("file://")
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from(url)))
}

fn remote_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn local_dotnet_versions(source: &Path, name: &str) -> Result<Vec<String>, FetchError> {
    let mut versions = Vec::new();
    collect_hierarchical_versions(&source.join(name.to_lowercase()), &mut versions)?;
    collect_hierarchical_versions(&source.join(name), &mut versions)?;
    collect_flat_package_versions(source, name, &mut versions)?;
    Ok(versions)
}

fn collect_hierarchical_versions(
    package_dir: &Path,
    versions: &mut Vec<String>,
) -> Result<(), FetchError> {
    let entries = match std::fs::read_dir(package_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!(
                    "failed to read NuGet package directory {}",
                    package_dir.display()
                ))
                .into());
        }
    };

    for entry in entries {
        let entry = entry.map_err(anyhow::Error::new)?;
        if entry.file_type().map_err(anyhow::Error::new)?.is_dir()
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
    versions: &mut Vec<String>,
) -> Result<(), FetchError> {
    let entries = match std::fs::read_dir(source) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(anyhow::Error::new(error)
                .context(format!(
                    "failed to read NuGet source directory {}",
                    source.display()
                ))
                .into());
        }
    };
    let prefix = format!("{}.", name.to_lowercase());

    for entry in entries {
        let entry = entry.map_err(anyhow::Error::new)?;
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

fn push_unique_version(versions: &mut Vec<String>, version: &str) {
    if !versions.iter().any(|known| known == version) {
        versions.push(version.to_owned());
    }
}
