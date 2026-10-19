use versionlens_model::Dependency;
use versionlens_model::Ecosystem::{
    AnsibleGalaxy, Bazel, Docker, Hackage, Helm, Pub, Python, Ruby,
};
use versionlens_providers::{
    RegistryEndpoint, ansible_role_registry_url_with_base, python_package_json_url_template,
    registry_endpoint, registry_endpoint_with_base,
};

const GITHUB_API_REPO_PREFIX: &str = "https://api.github.com/repos/";

pub(super) fn hosted_registry_endpoints(dependency: &Dependency) -> Option<Vec<RegistryEndpoint>> {
    let hosted_url = dependency.hosted_url.as_deref()?;

    if hosted_url.starts_with(GITHUB_API_REPO_PREFIX) {
        return Some(vec![RegistryEndpoint::ecosystem(hosted_url.to_owned())]);
    }

    if dependency.ecosystem == Python && hosted_url.contains("://") {
        return Some(vec![python_source_registry_endpoint(
            dependency, hosted_url,
        )]);
    }

    if dependency.ecosystem == Hackage && hosted_url == "stackage" {
        return Some(vec![registry_endpoint(
            dependency.ecosystem,
            &dependency.name,
        )]);
    }

    if dependency.ecosystem == Ruby {
        return Some(vec![registry_endpoint_with_base(
            dependency.ecosystem,
            &dependency.name,
            Some(hosted_url),
        )]);
    }

    if dependency.ecosystem == Docker {
        return Some(vec![docker_hosted_registry_endpoint(
            dependency, hosted_url,
        )]);
    }

    if dependency.ecosystem == Pub {
        return Some(vec![pub_hosted_registry_endpoint(dependency, hosted_url)]);
    }

    if dependency.ecosystem == Helm {
        return Some(vec![helm_hosted_registry_endpoint(dependency, hosted_url)]);
    }

    if dependency.ecosystem == AnsibleGalaxy {
        return Some(vec![ansible_hosted_registry_endpoint(
            dependency, hosted_url,
        )]);
    }

    if dependency.ecosystem == Bazel {
        return Some(vec![registry_endpoint_with_base(
            dependency.ecosystem,
            &dependency.name,
            Some(hosted_url),
        )]);
    }

    None
}

fn python_source_registry_endpoint(dependency: &Dependency, hosted_url: &str) -> RegistryEndpoint {
    let url = python_package_json_url_template(hosted_url);
    registry_endpoint_with_base(dependency.ecosystem, &dependency.name, Some(&url))
}

fn docker_hosted_registry_endpoint(dependency: &Dependency, hosted_url: &str) -> RegistryEndpoint {
    let explicit_name = format!("{hosted_url}/{}", dependency.name);
    registry_endpoint(dependency.ecosystem, &explicit_name)
}

fn pub_hosted_registry_endpoint(dependency: &Dependency, hosted_url: &str) -> RegistryEndpoint {
    let name = dependency
        .hosted_name
        .as_deref()
        .unwrap_or(&dependency.name);
    registry_endpoint_with_base(dependency.ecosystem, name, Some(hosted_url))
}

fn helm_hosted_registry_endpoint(dependency: &Dependency, hosted_url: &str) -> RegistryEndpoint {
    if hosted_url.starts_with("oci://") {
        let name = dependency
            .hosted_name
            .as_deref()
            .unwrap_or(&dependency.name);
        let explicit_name = format!("{hosted_url}/{name}");
        return registry_endpoint(dependency.ecosystem, &explicit_name);
    }

    registry_endpoint_with_base(dependency.ecosystem, &dependency.name, Some(hosted_url))
}

fn ansible_hosted_registry_endpoint(dependency: &Dependency, hosted_url: &str) -> RegistryEndpoint {
    if hosted_url == "role" {
        return RegistryEndpoint::ecosystem(ansible_role_registry_url_with_base(
            "https://galaxy.ansible.com",
            dependency
                .hosted_name
                .as_deref()
                .unwrap_or(&dependency.name),
        ));
    }

    registry_endpoint_with_base(dependency.ecosystem, &dependency.name, Some(hosted_url))
}
