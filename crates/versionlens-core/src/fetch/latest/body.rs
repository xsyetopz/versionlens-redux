use std::fs::read_to_string;
use std::io::ErrorKind::NotFound as IoNotFound;
use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{Docker, Dotnet, GitHub, Maven};
use versionlens_providers::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    merge_docker_hub_response_pages,
};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;
use crate::session::operation::OperationContext;

use super::local_dotnet::local_dotnet_package_body;

type RegistryBodyResult = Result<Option<String>, FetchError>;

const DOCKER_HUB_MAX_PAGES: u8 = 3;

impl VersionLensSession {
    pub(in crate::fetch::latest) fn fetch_registry_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        if dependency.ecosystem == GitHub && url.ends_with("/tags") {
            return self.fetch_github_tags_body(dependency, url, context, operation);
        }

        if dependency.ecosystem == Docker && docker_hub_tags_page_url(url, 1).is_some() {
            return self.fetch_docker_hub_body(dependency, url, context, operation);
        }

        if let Some(body) = local_dotnet_package_body(dependency, url)? {
            return Ok(Some(body));
        }

        if dotnet_service_index_url(dependency, url) {
            return self.fetch_dotnet_package_body(dependency, url, context, operation);
        }

        if let Some(body) = local_maven_metadata_body(dependency, url)? {
            return Ok(Some(body));
        }

        self.get_text_or_status_with_context(url, dependency.ecosystem, context, operation)
    }

    pub(in crate::fetch) fn fetch_github_tags_body(
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
            let Some(body) = self.get_text_or_status_with_context(
                &page_url,
                dependency.ecosystem,
                context,
                operation,
            )?
            else {
                return Ok(None);
            };
            let entries = serde_json::from_str::<Vec<serde_json::Value>>(&body)
                .map_err(|error| FetchError::from(crate::anyhow_error(error)))?;
            let has_next = entries.len() == 30;
            tags.extend(entries);
            if !has_next {
                return Ok(Some(serde_json::Value::Array(tags).to_string()));
            }
            page += 1;
        }
    }

    fn fetch_dotnet_package_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
        operation: &OperationContext,
    ) -> RegistryBodyResult {
        let Some(index) =
            self.get_text_or_status_with_context(url, dependency.ecosystem, context, operation)?
        else {
            return Ok(None);
        };
        let Some(package_url) = dotnet_package_url_from_service_index(&index, &dependency.name)
        else {
            return Ok(None);
        };

        self.get_text_or_status_with_context(&package_url, dependency.ecosystem, context, operation)
    }

    fn fetch_docker_hub_body(
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
            let Some(body) = self.get_text_or_status_with_context(
                &page_url,
                dependency.ecosystem,
                context,
                operation,
            )?
            else {
                break;
            };
            let has_next = docker_hub_body_has_next_page(&body);
            pages.push(body);
            if !has_next {
                break;
            }
        }

        Ok(merge_docker_hub_response_pages(pages))
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

fn local_maven_metadata_body(dependency: &Dependency, url: &str) -> RegistryBodyResult {
    if dependency.ecosystem != Maven || url.contains("://") {
        return Ok(None);
    }

    match read_to_string(url) {
        Ok(body) => Ok(Some(body)),
        Err(error) if error.kind() == IoNotFound => Ok(None),
        Err(error) => Err(crate::anyhow_error(error)
            .context(format!("failed to read Maven metadata file {url}"))
            .into()),
    }
}
