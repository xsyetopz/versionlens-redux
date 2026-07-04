use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    merge_docker_hub_response_pages,
};

use crate::VersionLensSession;
use crate::error::FetchError;
use crate::registry::RegistryContext;

use super::local_dotnet::local_dotnet_package_body;

const DOCKER_HUB_MAX_PAGES: u8 = 3;

impl VersionLensSession {
    pub(in crate::fetch::latest) fn fetch_registry_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
    ) -> Result<Option<String>, FetchError> {
        if dependency.ecosystem == Ecosystem::Docker && docker_hub_tags_page_url(url, 1).is_some() {
            return self.fetch_docker_hub_body(dependency, url, context);
        }

        if let Some(body) = local_dotnet_package_body(dependency, url)? {
            return Ok(Some(body));
        }

        if dotnet_service_index_url(dependency, url) {
            return self.fetch_dotnet_package_body(dependency, url, context);
        }

        if let Some(body) = local_maven_metadata_body(dependency, url)? {
            return Ok(Some(body));
        }

        self.get_text_or_status_with_context(url, dependency.ecosystem, context)
    }

    fn fetch_dotnet_package_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
    ) -> Result<Option<String>, FetchError> {
        let Some(index) =
            self.get_text_or_status_with_context(url, dependency.ecosystem, context)?
        else {
            return Ok(None);
        };
        let Some(package_url) = dotnet_package_url_from_service_index(&index, &dependency.name)
        else {
            return Ok(None);
        };

        self.get_text_or_status_with_context(&package_url, dependency.ecosystem, context)
    }

    fn fetch_docker_hub_body(
        &self,
        dependency: &Dependency,
        url: &str,
        context: &RegistryContext,
    ) -> Result<Option<String>, FetchError> {
        let mut pages = Vec::new();

        for page in 1..=DOCKER_HUB_MAX_PAGES {
            let Some(page_url) = docker_hub_tags_page_url(url, page) else {
                return Ok(None);
            };
            let Some(body) =
                self.get_text_or_status_with_context(&page_url, dependency.ecosystem, context)?
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
    dependency.ecosystem == Ecosystem::Dotnet
        && remote_url(url)
        && !url.contains("v3-flatcontainer")
        && !url.contains("flatcontainer")
}

fn remote_url(url: &str) -> bool {
    url.starts_with("https://") || url.starts_with("http://")
}

fn local_maven_metadata_body(
    dependency: &Dependency,
    url: &str,
) -> Result<Option<String>, FetchError> {
    if dependency.ecosystem != Ecosystem::Maven || url.contains("://") {
        return Ok(None);
    }

    match std::fs::read_to_string(url) {
        Ok(body) => Ok(Some(body)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(anyhow::Error::new(error)
            .context(format!("failed to read Maven metadata file {url}"))
            .into()),
    }
}
