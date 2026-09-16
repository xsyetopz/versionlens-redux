use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::ErrorKind::NotFound as IoNotFound;
use std::sync::Arc;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{Cargo, Docker, Dotnet, GitHub, Maven};
use versionlens_providers::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    merge_docker_hub_response_pages,
};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

use super::local_dotnet::local_dotnet_package_body;

type RegistryBodyResult = Result<Option<Arc<str>>, FetchError>;

const DOCKER_HUB_MAX_PAGES: u8 = 3;
const CRATES_IO_ORIGIN: &str = "https://crates.io";
const CRATES_IO_API_PREFIX: &str = "https://crates.io/api/v1/crates/";

impl VersionLensSession {
    pub(in crate::fetch::latest) async fn fetch_registry_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        if dependency.ecosystem == Cargo && official_crates_io_versions_url(url) {
            return self.fetch_crates_io_body(url, context, operation).await;
        }

        if dependency.ecosystem == GitHub && url.ends_with("/tags") {
            return self
                .fetch_github_tags_body(dependency, url, context, operation)
                .await;
        }

        if dependency.ecosystem == Docker && docker_hub_tags_page_url(url, 1).is_some() {
            return self
                .fetch_docker_hub_body(dependency, url, context, operation)
                .await;
        }

        if let Some(body) = local_dotnet_package_body(dependency, url)? {
            return Ok(Some(Arc::from(body)));
        }

        if dotnet_service_index_url(dependency, url) {
            return self
                .fetch_dotnet_package_body(dependency, url, context, operation)
                .await;
        }

        if let Some(body) = local_maven_metadata_body(dependency, url)? {
            return Ok(Some(body));
        }

        self.get_text_or_status_with_context(url, dependency.ecosystem, context, operation)
            .await
    }

    async fn fetch_crates_io_body(
        &self,
        versions_url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        let mut url = format!("{versions_url}?per_page=100");
        let mut visited = HashSet::new();
        let mut versions = Vec::new();
        loop {
            if !visited.insert(url.clone()) {
                return Ok(None);
            }
            let Some(body) = self
                .get_text_or_status_with_context(&url, Cargo, context, operation)
                .await?
            else {
                return Ok(None);
            };
            let Some(page) = cargo_page_from_response(&body) else {
                return Ok(None);
            };
            versions.extend(page.versions);
            match cargo_next_page_from_response(&body) {
                Some(CargoNextPage::Next(next)) => {
                    let Some(next) = crates_io_relative_url(&url, &next) else {
                        return Ok(None);
                    };
                    url = next;
                }
                Some(CargoNextPage::Complete) => {
                    return Ok(Some(Arc::from(
                        serde_json::json!({ "versions": versions }).to_string(),
                    )));
                }
                None => return Ok(None),
            }
        }
    }

    pub(in crate::fetch) async fn fetch_github_tags_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        let mut tags = Vec::<serde_json::Value>::new();
        let mut page = 1_u64;
        loop {
            let page_url = if page == 1 {
                url.to_owned()
            } else {
                format!("{url}?per_page=30&page={page}")
            };
            let Some(body) = self
                .get_text_or_status_with_context(
                    &page_url,
                    dependency.ecosystem,
                    context,
                    operation,
                )
                .await?
            else {
                return Ok(None);
            };
            let entries = serde_json::from_str::<Vec<serde_json::Value>>(&body)
                .map_err(|error| FetchError::from(crate::anyhow_error(error)))?;
            let has_next = entries.len() == 30;
            tags.extend(entries);
            if !has_next {
                return Ok(Some(Arc::from(serde_json::Value::Array(tags).to_string())));
            }
            page += 1;
        }
    }

    async fn fetch_dotnet_package_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        let Some(index) = self
            .get_text_or_status_with_context(url, dependency.ecosystem, context, operation)
            .await?
        else {
            return Ok(None);
        };
        let Some(package_url) = dotnet_package_url_from_service_index(&index, &dependency.name)
        else {
            return Ok(None);
        };

        self.get_text_or_status_with_context(&package_url, dependency.ecosystem, context, operation)
            .await
    }

    async fn fetch_docker_hub_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        let mut pages = vec![];

        for page in 1..=DOCKER_HUB_MAX_PAGES {
            let Some(page_url) = docker_hub_tags_page_url(url, page) else {
                return Ok(None);
            };
            let Some(body) = self
                .get_text_or_status_with_context(
                    &page_url,
                    dependency.ecosystem,
                    context,
                    operation,
                )
                .await?
            else {
                break;
            };
            let has_next = docker_hub_body_has_next_page(&body);
            pages.push(body.to_string());
            if !has_next {
                break;
            }
        }

        Ok(merge_docker_hub_response_pages(pages).map(Arc::from))
    }
}

fn dotnet_service_index_url(dependency: &Dependency, url: &str) -> bool {
    dependency.ecosystem == Dotnet
        && remote_url(url)
        && !url.contains("v3-flatcontainer")
        && !url.contains("flatcontainer")
}

fn remote_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn official_crates_io_versions_url(url: &str) -> bool {
    url.starts_with(CRATES_IO_API_PREFIX)
        && url.ends_with("/versions")
        && !url[CRATES_IO_API_PREFIX.len()..url.len() - "/versions".len()].contains('/')
}

fn crates_io_relative_url(current: &str, next: &str) -> Option<String> {
    if next.starts_with('?') {
        return Some(format!(
            "{}{}",
            current.split_once('?').map_or(current, |(base, _)| base),
            next
        ));
    }
    if next.starts_with('/') && !next.starts_with("//") {
        return Some(format!("{CRATES_IO_ORIGIN}{next}"));
    }
    None
}

#[derive(Debug, PartialEq, Eq)]
enum CargoNextPage {
    Next(String),
    Complete,
}

fn cargo_next_page_from_response(body: &str) -> Option<CargoNextPage> {
    match serde_json::from_str::<serde_json::Value>(body)
        .ok()?
        .pointer("/meta/next_page")?
    {
        serde_json::Value::String(next) if !next.is_empty() => {
            Some(CargoNextPage::Next(next.to_owned()))
        }
        serde_json::Value::Null => Some(CargoNextPage::Complete),
        _ => None,
    }
}

#[derive(serde::Deserialize)]
struct CargoPage {
    versions: Vec<serde_json::Value>,
}

fn cargo_page_from_response(body: &str) -> Option<CargoPage> {
    serde_json::from_str(body).ok()
}

fn local_maven_metadata_body(dependency: &Dependency, url: &str) -> RegistryBodyResult {
    if dependency.ecosystem != Maven || url.contains("://") {
        return Ok(None);
    }

    match read_to_string(url) {
        Ok(body) => Ok(Some(Arc::from(body))),
        Err(error) if error.kind() == IoNotFound => Ok(None),
        Err(error) => Err(crate::anyhow_error(error)
            .context(format!("failed to read Maven metadata file {url}"))
            .into()),
    }
}

#[cfg(test)]
mod tests;
