use versionlens_parsers::Ecosystem;

use super::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    is_composer_platform_dependency, is_registry_dependency, is_registry_requirement,
    merge_docker_hub_response_pages, provider_id, registry_url, registry_url_with_base,
};

#[test]
fn builds_registry_urls() {
    assert_eq!(provider_id(Ecosystem::Cargo), "cargo");
    assert_eq!(provider_id(Ecosystem::Composer), "composer");
    assert_eq!(provider_id(Ecosystem::Deno), "deno");
    assert_eq!(provider_id(Ecosystem::Npm), "npm");
    assert_eq!(
        registry_url(Ecosystem::Npm, "@types/node"),
        "https://registry.npmjs.org/@types%2fnode"
    );
    assert_eq!(
        registry_url(Ecosystem::Npm, "octokit/core.js"),
        "https://api.github.com/repos/octokit/core.js/tags"
    );
    assert_eq!(
        registry_url(Ecosystem::Cargo, "serde"),
        "https://crates.io/api/v1/crates/serde/versions"
    );
    assert_eq!(
        registry_url(Ecosystem::Composer, "phpunit/phpunit"),
        "https://repo.packagist.org/p2/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url(Ecosystem::Deno, "@std/assert"),
        "https://jsr.io/@std/assert/meta.json"
    );
    assert_eq!(provider_id(Ecosystem::Dotnet), "dotnet");
    assert_eq!(
        registry_url(Ecosystem::Dotnet, "Newtonsoft.Json"),
        "https://api.nuget.org/v3-flatcontainer/newtonsoft.json/index.json"
    );
    assert_eq!(provider_id(Ecosystem::Docker), "docker");
    assert_eq!(
        registry_url(Ecosystem::Docker, "ubuntu"),
        "https://hub.docker.com/v2/namespaces/library/repositories/ubuntu/tags"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "library/node"),
        "https://hub.docker.com/v2/namespaces/library/repositories/node/tags"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "mcr.microsoft.com/dotnet/sdk"),
        "https://mcr.microsoft.com/api/v1/catalog/dotnet/sdk/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "mcr.microsoft.com/dotnet"),
        "https://mcr.microsoft.com/api/v1/catalog/library/dotnet/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "docker.io/library/node"),
        "https://mcr.microsoft.com/api/v1/catalog/library/node/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "ghcr.io/org/app"),
        "https://mcr.microsoft.com/api/v1/catalog/org/app/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "localhost:5000/org/app"),
        "https://mcr.microsoft.com/api/v1/catalog/org/app/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Ecosystem::Docker, "one/two/three"),
        "https://hub.docker.com/v2/namespaces/one/repositories/two/tags"
    );
    assert_eq!(provider_id(Ecosystem::Dub), "dub");
    assert_eq!(
        registry_url(Ecosystem::Dub, "vibe-d"),
        "https://code.dlang.org/api/packages/vibe-d/info?minimize=true"
    );
    assert_eq!(
        registry_url(Ecosystem::Dub, "org/pkg name"),
        "https://code.dlang.org/api/packages/org%2Fpkg%20name/info?minimize=true"
    );
    assert_eq!(provider_id(Ecosystem::Go), "go");
    assert_eq!(
        registry_url(Ecosystem::Go, "Go.uber.org/Zap"),
        "https://proxy.golang.org/go.uber.org/zap/@v/list"
    );
    assert_eq!(provider_id(Ecosystem::Maven), "maven");
    assert_eq!(
        registry_url(Ecosystem::Maven, "org.springframework:spring-core"),
        "https://repo.maven.apache.org/maven2/org/springframework/spring-core/maven-metadata.xml"
    );
    assert_eq!(provider_id(Ecosystem::Python), "python");
    assert_eq!(
        registry_url(Ecosystem::Python, "requests"),
        "https://pypi.org/rss/project/requests/releases.xml"
    );
    assert_eq!(provider_id(Ecosystem::Ruby), "ruby");
    assert_eq!(
        registry_url(Ecosystem::Ruby, "rails"),
        "https://rubygems.org/api/v1/versions/rails.json"
    );
    assert_eq!(
        registry_url(Ecosystem::Ruby, "rspec/rspec-rails"),
        "https://api.github.com/repos/rspec/rspec-rails/tags"
    );
    assert_eq!(provider_id(Ecosystem::Pub), "pub");
    assert_eq!(
        registry_url(Ecosystem::Pub, "http"),
        "https://pub.dev/api/packages/http"
    );
}

#[test]
fn builds_custom_registry_urls() {
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Cargo,
            "serde",
            Some("https://mirror.test/crates")
        ),
        "https://mirror.test/crates/serde/versions"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Go,
            "Go.uber.org/Zap",
            Some("https://proxy.test/{base-module}/@v/list")
        ),
        "https://proxy.test/go.uber.org/zap/@v/list"
    );
    assert_eq!(
        registry_url_with_base(Ecosystem::Go, "Go.uber.org/Zap", Some("https://proxy.test")),
        "https://proxy.test"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Python,
            "requests",
            Some("https://pypi.test/pypi/{name}/json")
        ),
        "https://pypi.test/pypi/requests/json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Python,
            "requests",
            Some("https://pypi.test/pypi")
        ),
        "https://pypi.test/pypi"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Npm,
            "@types/node",
            Some("https://registry.test/npm/")
        ),
        "https://registry.test/npm/@types%2fnode"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Maven,
            "org.springframework:spring-core",
            Some("https://repo.test/maven2/")
        ),
        "https://repo.test/maven2/org/springframework/spring-core/maven-metadata.xml"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Dotnet,
            "Newtonsoft.Json",
            Some("https://nuget.test/v3-flatcontainer")
        ),
        "https://nuget.test/v3-flatcontainer/newtonsoft.json/index.json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Dub,
            "org/pkg name",
            Some("https://dub.test/packages")
        ),
        "https://dub.test/packages/org%2Fpkg%20name/info?minimize=true"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Docker,
            "org/app",
            Some("https://registry.test/v2")
        ),
        "https://registry.test/v2/org/app/tags/list"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Go,
            "Go.uber.org/Zap",
            Some("https://proxy.test/{base-module}/{base-module}")
        ),
        "https://proxy.test/go.uber.org/zap/{base-module}"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Python,
            "requests",
            Some("https://pypi.test/{name}/{name}")
        ),
        "https://pypi.test/requests/{name}"
    );
}

#[test]
fn builds_custom_package_registry_urls() {
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Composer,
            "phpunit/phpunit",
            Some("https://composer.test/p2")
        ),
        "https://composer.test/p2/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Composer,
            "phpunit/phpunit",
            Some("https://composer.test")
        ),
        "https://composer.test/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Ruby,
            "rails",
            Some("https://gems.test/api/v1/versions/{name}.json")
        ),
        "https://gems.test/api/v1/versions/rails.json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Ruby,
            "rails",
            Some("https://gems.test/{name}/{name}")
        ),
        "https://gems.test/rails/{name}"
    );
    assert_eq!(
        registry_url_with_base(Ecosystem::Ruby, "rails", Some("https://gems.test")),
        "https://gems.test"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Pub,
            "http",
            Some("https://pub.test/api/packages")
        ),
        "https://pub.test/api/packages/http"
    );
}

#[test]
fn trims_package_names_for_registry_urls() {
    assert_eq!(
        registry_url(Ecosystem::Npm, " @types/node "),
        "https://registry.npmjs.org/@types%2fnode"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Python,
            " requests ",
            Some("https://pypi.test/pypi/{name}/json")
        ),
        "https://pypi.test/pypi/requests/json"
    );
    assert_eq!(
        registry_url_with_base(
            Ecosystem::Dotnet,
            " Newtonsoft.Json ",
            Some("https://nuget.test/v3-flatcontainer")
        ),
        "https://nuget.test/v3-flatcontainer/newtonsoft.json/index.json"
    );
}

#[test]
fn builds_dotnet_package_url_from_service_index() {
    let body = r#"{
      "resources": [
        {
          "@id": "",
          "@type": "PackageBaseAddress/3.0.0"
        },
        {
          "@id": "https://nuget.test/v3-flatcontainer/",
          "@type": "PackageBaseAddress/3.0.0"
        }
      ]
    }"#;

    assert_eq!(
        dotnet_package_url_from_service_index(body, "Newtonsoft.Json").as_deref(),
        Some("https://nuget.test/v3-flatcontainer/newtonsoft.json/index.json")
    );
}

#[test]
fn identifies_non_registry_requirements() {
    assert!(is_registry_requirement(Ecosystem::Npm, "^1.0.0"));
    assert!(is_registry_requirement(Ecosystem::Python, ">=2.0"));
    assert!(is_registry_requirement(Ecosystem::Python, ""));
    assert!(!is_registry_requirement(Ecosystem::Npm, "file:../local"));
    assert!(!is_registry_requirement(
        Ecosystem::Npm,
        "github:twbs/bootstrap#v5.3.8"
    ));
    assert!(!is_registry_requirement(
        Ecosystem::Npm,
        "gitlab:org/package#v1.2.3"
    ));
    assert!(!is_registry_requirement(
        Ecosystem::Npm,
        "bitbucket:org/package#v1.2.3"
    ));
    assert!(!is_registry_requirement(Ecosystem::Npm, "workspace:*"));
    assert!(!is_registry_requirement(Ecosystem::Npm, "catalog:react18"));
    assert!(!is_registry_requirement(Ecosystem::Cargo, "../local"));
    assert!(!is_registry_requirement(
        Ecosystem::Cargo,
        "https://example.test/repo.git"
    ));
    assert!(!is_registry_requirement(Ecosystem::Docker, "$TAG"));
    assert!(!is_registry_requirement(Ecosystem::Docker, "sha256:abc123"));
    assert!(is_registry_requirement(Ecosystem::Dotnet, "1.2.3.4"));
    assert!(is_registry_requirement(Ecosystem::Dotnet, "[1.2.3.4,)"));
    assert!(is_registry_requirement(Ecosystem::Dotnet, "[1.2.3.*,)"));
    assert!(is_registry_requirement(Ecosystem::Dotnet, "[1.2.3,2.0.0)"));
    assert!(!is_registry_requirement(Ecosystem::Ruby, "vendor/local"));
    assert!(is_registry_requirement(Ecosystem::Python, "sha256:abc123"));
}

#[test]
fn identifies_composer_platform_dependencies() {
    for name in [
        "php",
        "ext-json",
        "lib-curl",
        "composer",
        "composer-plugin-api",
        "composer-runtime-api",
    ] {
        assert!(is_composer_platform_dependency(name));
        assert!(!is_registry_dependency(Ecosystem::Composer, name, "^1.0"));
    }

    assert!(!is_composer_platform_dependency("phpunit/phpunit"));
    assert!(is_registry_dependency(
        Ecosystem::Composer,
        "phpunit/phpunit",
        "^10.0"
    ));
}

#[test]
fn identifies_supported_docker_registry_dependencies() {
    assert!(is_registry_dependency(Ecosystem::Docker, "ubuntu", "24.04"));
    assert!(is_registry_dependency(
        Ecosystem::Docker,
        "docker.io/library/node",
        "22"
    ));
    assert!(is_registry_dependency(
        Ecosystem::Docker,
        "mcr.microsoft.com/dotnet/sdk",
        "9.0"
    ));
    assert!(is_registry_dependency(
        Ecosystem::Docker,
        "ghcr.io/org/app",
        "1.2.3"
    ));
    assert!(!is_registry_dependency(
        Ecosystem::Docker,
        "$IMAGE",
        "latest"
    ));
    assert!(!is_registry_dependency(Ecosystem::Docker, "ubuntu", "$TAG"));
}

#[test]
fn builds_docker_hub_page_urls() {
    assert_eq!(
        docker_hub_tags_page_url(
            "https://hub.docker.com/v2/namespaces/library/repositories/node/tags",
            2
        )
        .as_deref(),
        Some(
            "https://hub.docker.com/v2/namespaces/library/repositories/node/tags?page=2&page_size=100&ordering=last_updated"
        )
    );
    assert_eq!(
        docker_hub_tags_page_url(
            "https://mcr.microsoft.com/api/v1/catalog/dotnet/sdk/tags?reg=mar",
            1
        ),
        None
    );
}

#[test]
fn reads_docker_hub_next_page_markers() {
    assert!(docker_hub_body_has_next_page(
        r#"{"next":"https://hub.docker.com/page/2","results":[]}"#
    ));
    assert!(!docker_hub_body_has_next_page(
        r#"{"next":null,"results":[]}"#
    ));
    assert!(!docker_hub_body_has_next_page(
        r#"{"next":"","results":[]}"#
    ));
}

#[test]
fn merges_docker_hub_response_pages() {
    let merged = merge_docker_hub_response_pages(vec![
        r#"{"count":2,"next":"page-2","results":[{"name":"24"}]}"#.to_owned(),
        r#"{"count":2,"next":null,"results":[{"name":"23"}]}"#.to_owned(),
    ]);

    assert!(
        merged
            .as_deref()
            .is_some_and(|body| body.contains(r#""name":"24""#))
    );
    assert!(
        merged
            .as_deref()
            .is_some_and(|body| body.contains(r#""name":"23""#))
    );
}
