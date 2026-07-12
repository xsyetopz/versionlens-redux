use super::{
    docker_hub_body_has_next_page, docker_hub_tags_page_url, dotnet_package_url_from_service_index,
    is_composer_platform_dependency, is_registry_dependency, is_registry_requirement,
    merge_docker_hub_response_pages, provider_id, registry_url, registry_url_with_base,
};
use versionlens_parsers::Ecosystem::{
    AnsibleGalaxy, Bazel, Cargo, CocoaPods, Composer, Conan, Cpan, Cpp, Cran, Deno, Docker, Dotnet,
    Dub, Go, Hackage, Haxelib, Helm, Hex, Julia, LuaRocks, Maven, Nim, Nix, Npm, Opam, Pub, Python,
    Ruby, Swift, Terraform, Unity, Vcpkg, Zig,
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "table-driven manifest coverage stays readable as one scenario"
)]
fn builds_registry_urls() {
    assert_eq!(provider_id(Cargo), "cargo");
    assert_eq!(provider_id(Composer), "composer");
    assert_eq!(provider_id(Deno), "deno");
    assert_eq!(provider_id(Npm), "npm");
    assert_eq!(
        registry_url(Npm, "@types/node"),
        "https://registry.npmjs.org/@types%2fnode"
    );
    assert_eq!(
        registry_url(Npm, "octokit/core.js"),
        "https://api.github.com/repos/octokit/core.js/tags"
    );
    assert_eq!(
        registry_url(Cargo, "serde"),
        "https://crates.io/api/v1/crates/serde/versions"
    );
    assert_eq!(
        registry_url(Composer, "phpunit/phpunit"),
        "https://repo.packagist.org/p2/phpunit/phpunit.json"
    );
    assert_eq!(
        registry_url(Deno, "@std/assert"),
        "https://jsr.io/@std/assert/meta.json"
    );
    assert_eq!(provider_id(Dotnet), "dotnet");
    assert_eq!(
        registry_url(Dotnet, "Newtonsoft.Json"),
        "https://api.nuget.org/v3-flatcontainer/newtonsoft.json/index.json"
    );
    assert_eq!(provider_id(Docker), "docker");
    assert_eq!(
        registry_url(Docker, "ubuntu"),
        "https://hub.docker.com/v2/namespaces/library/repositories/ubuntu/tags"
    );
    assert_eq!(
        registry_url(Docker, "library/node"),
        "https://hub.docker.com/v2/namespaces/library/repositories/node/tags"
    );
    assert_eq!(
        registry_url(Docker, "mcr.microsoft.com/dotnet/sdk"),
        "https://mcr.microsoft.com/api/v1/catalog/dotnet/sdk/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Docker, "mcr.microsoft.com/dotnet"),
        "https://mcr.microsoft.com/api/v1/catalog/library/dotnet/tags?reg=mar"
    );
    assert_eq!(
        registry_url(Docker, "docker.io/library/node"),
        "https://hub.docker.com/v2/namespaces/library/repositories/node/tags"
    );
    assert_eq!(
        registry_url(Docker, "ghcr.io/org/app"),
        "https://ghcr.io/v2/org/app/tags/list"
    );
    assert_eq!(
        registry_url(Docker, "localhost:5000/org/app"),
        "https://localhost:5000/v2/org/app/tags/list"
    );
    assert_eq!(
        registry_url(Docker, "one/two/three"),
        "https://hub.docker.com/v2/namespaces/one/repositories/two/tags"
    );
    assert_eq!(provider_id(Dub), "dub");
    assert_eq!(
        registry_url(Dub, "vibe-d"),
        "https://code.dlang.org/api/packages/vibe-d/info?minimize=true"
    );
    assert_eq!(
        registry_url(Dub, "org/pkg name"),
        "https://code.dlang.org/api/packages/org%2Fpkg%20name/info?minimize=true"
    );
    assert_eq!(provider_id(Go), "go");
    assert_eq!(
        registry_url(Go, "Go.uber.org/Zap"),
        "https://proxy.golang.org/!go.uber.org/!zap/@v/list"
    );
    assert_eq!(provider_id(Hex), "hex");
    assert_eq!(
        registry_url(Hex, "plug"),
        "https://hex.pm/api/packages/plug"
    );
    assert_eq!(provider_id(Opam), "opam");
    assert_eq!(
        registry_url(Opam, "lwt"),
        "https://opam.ocaml.org/packages/lwt/"
    );
    assert_eq!(provider_id(Hackage), "hackage");
    assert_eq!(
        registry_url(Hackage, "aeson"),
        "https://hackage.haskell.org/package/aeson.json"
    );
    assert_eq!(
        registry_url(Hackage, "stackage-lts"),
        "https://www.stackage.org/api/v1/snapshots"
    );
    assert_eq!(provider_id(Julia), "julia");
    assert_eq!(
        registry_url(Julia, "Example"),
        "https://raw.githubusercontent.com/JuliaRegistries/General/master/E/Example/Versions.toml"
    );
    assert_eq!(provider_id(Cran), "cran");
    assert_eq!(
        registry_url(Cran, "dplyr"),
        "https://cran.r-project.org/src/contrib/PACKAGES"
    );
    assert_eq!(provider_id(Conan), "conan");
    assert_eq!(
        registry_url(Conan, "zlib"),
        "https://center2.conan.io/v2/conans/search?q=zlib/*"
    );
    assert_eq!(provider_id(Vcpkg), "vcpkg");
    assert_eq!(
        registry_url(Vcpkg, "fmt"),
        "https://raw.githubusercontent.com/microsoft/vcpkg/master/versions/f-/fmt.json"
    );
    assert_eq!(provider_id(Swift), "swift");
    assert_eq!(
        registry_url(Swift, "mona.LinkedList"),
        "https://packages.swift.org/mona/LinkedList"
    );
    assert_eq!(provider_id(Zig), "zig");
    assert_eq!(
        registry_url(Zig, "ziglibs/known-folders"),
        "https://api.github.com/repos/ziglibs/known-folders/tags"
    );
    assert_eq!(provider_id(Nim), "nim");
    assert_eq!(
        registry_url(Nim, "jester"),
        "https://raw.githubusercontent.com/nim-lang/packages/master/packages.json"
    );
    assert_eq!(
        registry_url(Nim, "user/pkg"),
        "https://api.github.com/repos/user/pkg/tags"
    );
    assert_eq!(provider_id(LuaRocks), "luarocks");
    assert_eq!(
        registry_url(LuaRocks, "luasocket"),
        "https://luarocks.org/manifest"
    );
    assert_eq!(provider_id(Cpan), "cpan");
    assert_eq!(
        registry_url(Cpan, "Plack"),
        "https://fastapi.metacpan.org/v1/download_url/Plack"
    );
    assert_eq!(provider_id(Terraform), "terraform");
    assert_eq!(
        registry_url(Terraform, "hashicorp/aws"),
        "https://registry.terraform.io/v1/providers/hashicorp/aws/versions"
    );
    assert_eq!(
        registry_url(Terraform, "registry.opentofu.org/opentofu/random"),
        "https://registry.opentofu.org/v1/providers/opentofu/random/versions"
    );
    assert_eq!(provider_id(Helm), "helm");
    assert_eq!(
        registry_url(Helm, "apache"),
        "https://charts.bitnami.com/bitnami/index.yaml"
    );
    assert_eq!(provider_id(AnsibleGalaxy), "ansible");
    assert_eq!(
        registry_url(AnsibleGalaxy, "community.general"),
        "https://galaxy.ansible.com/api/v3/plugin/ansible/content/published/collections/index/community/general/versions/"
    );
    assert_eq!(provider_id(Bazel), "bazel");
    assert_eq!(
        registry_url(Bazel, "rules_cc"),
        "https://raw.githubusercontent.com/bazelbuild/bazel-central-registry/main/modules/rules_cc/metadata.json"
    );
    assert_eq!(provider_id(Nix), "nix");
    assert_eq!(
        registry_url(Nix, "NixOS/nixpkgs"),
        "https://api.github.com/repos/NixOS/nixpkgs/tags"
    );
    assert_eq!(provider_id(CocoaPods), "cocoapods");
    assert_eq!(
        registry_url(CocoaPods, "AFNetworking"),
        "https://trunk.cocoapods.org/api/v1/pods/AFNetworking"
    );
    assert_eq!(
        registry_url(CocoaPods, "QueryKit/Attribute"),
        "https://trunk.cocoapods.org/api/v1/pods/QueryKit"
    );
    assert_eq!(
        registry_url_with_base(
            CocoaPods,
            "PonyDebugger",
            Some("https://private.example.com/specs")
        ),
        "https://private.example.com/specs/PonyDebugger"
    );
    assert_eq!(provider_id(Unity), "unity");
    assert_eq!(
        registry_url(Unity, "com.unity.timeline"),
        "https://packages.unity.com/com.unity.timeline"
    );
    assert_eq!(
        registry_url_with_base(
            Unity,
            "com.example.tools.physics",
            Some("https://registry.example.com")
        ),
        "https://registry.example.com/com.example.tools.physics"
    );
    assert_eq!(provider_id(Maven), "maven");
    assert_eq!(
        registry_url(Maven, "org.springframework:spring-core"),
        "https://repo.maven.apache.org/maven2/org/springframework/spring-core/maven-metadata.xml"
    );
    assert_eq!(provider_id(Python), "python");
    assert_eq!(
        registry_url(Python, "requests"),
        "https://pypi.org/rss/project/requests/releases.xml"
    );
    assert_eq!(provider_id(Ruby), "ruby");
    assert_eq!(
        registry_url(Ruby, "rails"),
        "https://rubygems.org/api/v1/versions/rails.json"
    );
    assert_eq!(
        registry_url(Ruby, "rspec/rspec-rails"),
        "https://api.github.com/repos/rspec/rspec-rails/tags"
    );
    assert_eq!(provider_id(Pub), "pub");
    assert_eq!(
        registry_url(Pub, "http"),
        "https://pub.dev/api/packages/http"
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "table-driven manifest coverage stays readable as one scenario"
)]
fn builds_custom_registry_urls() {
    assert_eq!(
        registry_url_with_base(Cargo, "serde", Some("https://mirror.test/crates")),
        "https://mirror.test/crates/serde/versions"
    );
    assert_eq!(
        registry_url_with_base(
            Go,
            "Go.uber.org/Zap",
            Some("https://proxy.test/{base-module}/@v/list")
        ),
        "https://proxy.test/!go.uber.org/!zap/@v/list"
    );
    assert_eq!(
        registry_url_with_base(Go, "Go.uber.org/Zap", Some("https://proxy.test")),
        "https://proxy.test"
    );
    assert_eq!(
        registry_url_with_base(
            Python,
            "requests",
            Some("https://pypi.test/pypi/{name}/json")
        ),
        "https://pypi.test/pypi/requests/json"
    );
    assert_eq!(
        registry_url_with_base(Python, "requests", Some("https://pypi.test/pypi")),
        "https://pypi.test/pypi"
    );
    assert_eq!(
        registry_url_with_base(Npm, "@types/node", Some("https://registry.test/npm/")),
        "https://registry.test/npm/@types%2fnode"
    );
    assert_eq!(
        registry_url_with_base(
            Maven,
            "org.springframework:spring-core",
            Some("https://repo.test/maven2/")
        ),
        "https://repo.test/maven2/org/springframework/spring-core/maven-metadata.xml"
    );
    assert_eq!(
        registry_url_with_base(
            Dotnet,
            "Newtonsoft.Json",
            Some("https://nuget.test/v3-flatcontainer")
        ),
        "https://nuget.test/v3-flatcontainer/newtonsoft.json/index.json"
    );
    assert_eq!(
        registry_url_with_base(Dub, "org/pkg name", Some("https://dub.test/packages")),
        "https://dub.test/packages/org%2Fpkg%20name/info?minimize=true"
    );
    assert_eq!(
        registry_url_with_base(Docker, "org/app", Some("https://registry.test/v2")),
        "https://registry.test/v2/org/app/tags/list"
    );
    assert_eq!(
        registry_url_with_base(Deno, "@scope/pkg", Some("https://jsr.example.test/")),
        "https://jsr.example.test/@scope/pkg/meta.json"
    );
    assert_eq!(
        registry_url_with_base(Hex, "plug", Some("https://hex.test")),
        "https://hex.test/api/packages/plug"
    );
    assert_eq!(
        registry_url_with_base(Hex, "plug", Some("https://hex.test/api")),
        "https://hex.test/api/packages/plug"
    );
    assert_eq!(
        registry_url_with_base(Hex, "plug_crypto", Some("https://hex.test/api/packages")),
        "https://hex.test/api/packages/plug_crypto"
    );
    assert_eq!(
        registry_url_with_base(Opam, "lwt", Some("https://opam.test")),
        "https://opam.test/packages/lwt/"
    );
    assert_eq!(
        registry_url_with_base(Opam, "cohttp-lwt-unix", Some("https://opam.test/packages")),
        "https://opam.test/packages/cohttp-lwt-unix/"
    );
    assert_eq!(
        registry_url_with_base(Hackage, "aeson", Some("https://hackage.test/package")),
        "https://hackage.test/package/aeson.json"
    );
    assert_eq!(
        registry_url_with_base(Julia, "DataFrames", Some("https://registry.test/General")),
        "https://registry.test/General/D/DataFrames/Versions.toml"
    );
    assert_eq!(
        registry_url_with_base(Cran, "dplyr", Some("https://cran.test")),
        "https://cran.test/src/contrib/PACKAGES"
    );
    assert_eq!(
        registry_url_with_base(
            Go,
            "Go.uber.org/Zap",
            Some("https://proxy.test/{base-module}/{base-module}")
        ),
        "https://proxy.test/!go.uber.org/!zap/!go.uber.org/!zap"
    );
    assert_eq!(
        registry_url_with_base(Python, "requests", Some("https://pypi.test/{name}/{name}")),
        "https://pypi.test/requests/requests"
    );
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
