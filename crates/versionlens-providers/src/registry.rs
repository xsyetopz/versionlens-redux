use versionlens_parsers::Ecosystem;

mod eligibility;
mod urls;

pub use eligibility::{
    is_composer_platform_dependency, is_registry_dependency, is_registry_requirement,
    is_unsupported_dotnet_requirement,
};
pub use urls::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    merge_docker_hub_response_pages, python_package_json_url_template, registry_url,
    registry_url_with_base,
};

const PROVIDER_IDS: &[&str] = &[
    "cargo", "composer", "deno", "dotnet", "docker", "dub", "go", "maven", "npm", "python", "pub",
    "ruby",
];

pub fn provider_id(ecosystem: Ecosystem) -> &'static str {
    PROVIDER_IDS[ecosystem as usize]
}

#[cfg(test)]
mod tests;
