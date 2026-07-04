use versionlens_parsers::{Dependency, Ecosystem};
use versionlens_providers::{
    python_package_json_url_template, registry_url, registry_url_with_base,
};

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(super) fn hosted_registry_urls(dependency: &Dependency) -> Option<Vec<String>> {
    let hosted_url = dependency.hosted_url.as_deref()?;

    if hosted_url.starts_with(GITHUB_API_REPO_PREFIX) {
        return Some(vec![hosted_url.to_owned()]);
    }

    if dependency.ecosystem == Ecosystem::Python {
        return Some(vec![python_source_registry_url(dependency, hosted_url)]);
    }

    if dependency.ecosystem == Ecosystem::Ruby {
        return Some(vec![registry_url_with_base(
            dependency.ecosystem,
            &dependency.name,
            Some(hosted_url),
        )]);
    }

    if dependency.ecosystem == Ecosystem::Docker {
        return Some(vec![docker_hosted_registry_url(dependency, hosted_url)]);
    }

    if dependency.ecosystem == Ecosystem::Pub {
        return Some(vec![pub_hosted_registry_url(dependency, hosted_url)]);
    }

    None
}

fn python_source_registry_url(dependency: &Dependency, hosted_url: &str) -> String {
    let url = python_package_json_url_template(hosted_url);
    registry_url_with_base(dependency.ecosystem, &dependency.name, Some(&url))
}

fn docker_hosted_registry_url(dependency: &Dependency, hosted_url: &str) -> String {
    let explicit_name = format!("{hosted_url}/{}", dependency.name);
    registry_url(dependency.ecosystem, &explicit_name)
}

fn pub_hosted_registry_url(dependency: &Dependency, hosted_url: &str) -> String {
    format!("{hosted_url}/{}", dependency.name)
}
