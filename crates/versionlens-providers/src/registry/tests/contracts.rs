#[test]
fn registry_endpoints_identify_response_capabilities() {
    assert_eq!(
        registry_endpoint(Go, "Go.uber.org/Zap").response_kind,
        RegistryResponseKind::GoModuleList
    );
    assert_eq!(
        registry_endpoint(Python, "requests").response_kind,
        RegistryResponseKind::PythonRss
    );
    let simple = registry_endpoint_with_base(
        Python,
        "My_Package.Name",
        Some("https://pypi.test/repository/simple/"),
    );
    assert_eq!(
        simple.url,
        "https://pypi.test/repository/simple/my-package-name/"
    );
    assert_eq!(simple.response_kind, RegistryResponseKind::PythonSimple);

    let json = registry_endpoint_with_base(
        Python,
        "requests",
        Some("https://pypi.test/pypi/{name}/json"),
    );
    assert_eq!(json.response_kind, RegistryResponseKind::PythonJson);
}

#[test]
fn go_module_proxy_urls_case_encode_module_paths() {
    assert_eq!(
        registry_url(Go, "example.com/M"),
        "https://proxy.golang.org/example.com/!m/@v/list"
    );
    assert_eq!(
        registry_url_with_base(
            Go,
            "Go.uber.org/Zap",
            Some("https://proxy.test/{base-module}/@v/list")
        ),
        "https://proxy.test/!go.uber.org/!zap/@v/list"
    );
}

#[test]
fn builds_custom_package_registry_urls() {
    assert_eq!(
        registry_url_with_base(
            Composer,
            "phpunit/phpunit",
            Some("https://composer.test/p2")
        ),
        "https://composer.test/p2/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url_with_base(Composer, "phpunit/phpunit", Some("https://composer.test")),
        "https://composer.test/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url_with_base(
            Ruby,
            "rails",
            Some("https://gems.test/api/v1/versions/{name}.json")
        ),
        "https://gems.test/api/v1/versions/rails.json"
    );
    assert_eq!(
        registry_url_with_base(Ruby, "rails", Some("https://gems.test/{name}/{name}")),
        "https://gems.test/rails/rails"
    );
    assert_eq!(
        registry_url_with_base(Ruby, "rails", Some("https://gems.test")),
        "https://gems.test/api/v1/versions/rails.json"
    );
    assert_eq!(
        registry_url_with_base(Pub, "http", Some("https://pub.test/api/packages")),
        "https://pub.test/api/packages/http"
    );
    assert_eq!(
        registry_url_with_base(Pub, "http", Some("https://pub.test/")),
        "https://pub.test/api/packages/http"
    );
    assert_eq!(
        registry_url_with_base(Haxelib, "tink_core", Some("https://haxe.test")),
        "https://haxe.test/p/tink_core/versions/"
    );
    assert_eq!(
        registry_url_with_base(Terraform, "hashicorp/aws", Some("https://registry.test")),
        "https://registry.test/v1/providers/hashicorp/aws/versions"
    );
    assert_eq!(
        registry_url_with_base(
            Terraform,
            "registry.opentofu.org/opentofu/random",
            Some("https://registry.test")
        ),
        "https://registry.opentofu.org/v1/providers/opentofu/random/versions"
    );
    assert_eq!(
        registry_url_with_base(Helm, "apache", Some("https://charts.example.test")),
        "https://charts.example.test/index.yaml"
    );
    assert_eq!(
        registry_url_with_base(
            Helm,
            "oci://registry.example.com/charts/mysql",
            Some("https://charts.example.test")
        ),
        "https://registry.example.com/v2/charts/mysql/tags/list"
    );
    assert_eq!(
        registry_url_with_base(
            AnsibleGalaxy,
            "acme.private",
            Some("https://galaxy.example.test")
        ),
        "https://galaxy.example.test/api/v3/plugin/ansible/content/published/collections/index/acme/private/versions/"
    );
    assert_eq!(
        registry_url_with_base(Bazel, "rules_cc", Some("https://bcr.example.test")),
        "https://bcr.example.test/modules/rules_cc/metadata.json"
    );
}

#[test]
fn builds_haxelib_registry_urls() {
    assert_eq!(
        registry_url(Haxelib, "tink_core"),
        "https://lib.haxe.org/p/tink_core/versions/"
    );
}

#[test]
fn registry_urls_escape_unsafe_bytes_for_every_ecosystem() {
    let ecosystems = [
        Cargo,
        Composer,
        Deno,
        Dotnet,
        Docker,
        Dub,
        Go,
        Maven,
        Npm,
        Python,
        Pub,
        Ruby,
        Hex,
        Opam,
        Hackage,
        Julia,
        Cran,
        Conan,
        Vcpkg,
        Swift,
        Zig,
        Nim,
        LuaRocks,
        Cpan,
        Haxelib,
        Terraform,
        Helm,
        AnsibleGalaxy,
        Bazel,
        Nix,
        Unity,
        CocoaPods,
        Cpp,
    ];

    for ecosystem in ecosystems {
        let url = registry_url(ecosystem, " package name\\with\nnewline ");
        assert!(
            url.bytes().all(|byte| byte.is_ascii_graphic()),
            "{ecosystem:?} produced an unsafe registry URL: {url:?}"
        );
        assert!(
            !url.contains([' ', '\\', '\n', '\r', '\t']),
            "{ecosystem:?} produced an unsafe registry URL: {url:?}"
        );
    }
}

#[test]
fn trims_package_names_for_registry_urls() {
    assert_eq!(
        registry_url(Npm, " @types/node "),
        "https://registry.npmjs.org/@types%2fnode"
    );
    assert_eq!(
        registry_url_with_base(
            Python,
            " requests ",
            Some("https://pypi.test/pypi/{name}/json")
        ),
        "https://pypi.test/pypi/requests/json"
    );
    assert_eq!(
        registry_url_with_base(
            Dotnet,
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
    assert!(is_registry_requirement(Npm, "^1.0.0"));
    assert!(is_registry_requirement(Python, ">=2.0"));
    assert!(is_registry_requirement(Python, ""));
    assert!(!is_registry_requirement(Npm, "file:../local"));
    assert!(!is_registry_requirement(
        Npm,
        "github:twbs/bootstrap#v5.3.8"
    ));
    assert!(!is_registry_requirement(Npm, "gitlab:org/package#v1.2.3"));
    assert!(!is_registry_requirement(
        Npm,
        "bitbucket:org/package#v1.2.3"
    ));
    assert!(!is_registry_requirement(Npm, "workspace:*"));
    assert!(!is_registry_requirement(Npm, "catalog:react18"));
    assert!(!is_registry_requirement(Npm, "portal:../local"));
    assert!(!is_registry_requirement(
        Npm,
        "exec:./scripts/build-package.js"
    ));
    assert!(!is_registry_requirement(
        Npm,
        "patch:@types/react@18.0.0#./patches/react.patch"
    ));
    assert!(!is_registry_requirement(Cargo, "../local"));
    assert!(!is_registry_requirement(Cargo, "workspace:true"));
    assert!(!is_registry_requirement(
        Cargo,
        "https://example.test/repo.git"
    ));
    assert!(!is_registry_requirement(Docker, "$TAG"));
    assert!(!is_registry_requirement(Docker, "sha256:abc123"));
    assert!(is_registry_requirement(Dotnet, "1.2.3.4"));
    assert!(is_registry_requirement(Dotnet, "[1.2.3.4,)"));
    assert!(is_registry_requirement(Dotnet, "[1.2.3.*,)"));
    assert!(is_registry_requirement(Dotnet, "[1.2.3,2.0.0)"));
    assert!(!is_registry_requirement(Ruby, "vendor/local"));
    assert!(!is_registry_dependency(Cran, "R", ">= 4.3"));
    assert!(is_registry_requirement(Python, "sha256:abc123"));
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
        assert!(!is_registry_dependency(Composer, name, "^1.0"));
    }

    assert!(!is_composer_platform_dependency("phpunit/phpunit"));
    assert!(is_registry_dependency(Composer, "phpunit/phpunit", "^10.0"));
}

#[test]
fn identifies_supported_docker_registry_dependencies() {
    assert!(is_registry_dependency(Docker, "ubuntu", "24.04"));
    assert!(is_registry_dependency(
        Docker,
        "docker.io/library/node",
        "22"
    ));
    assert!(is_registry_dependency(
        Docker,
        "mcr.microsoft.com/dotnet/sdk",
        "9.0"
    ));
    assert!(is_registry_dependency(Docker, "ghcr.io/org/app", "1.2.3"));
    assert!(!is_registry_dependency(Docker, "$IMAGE", "latest"));
    assert!(!is_registry_dependency(Docker, "ubuntu", "$TAG"));
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
