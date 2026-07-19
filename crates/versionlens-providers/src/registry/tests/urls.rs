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
        "https://proxy.test/!go.uber.org/!zap/@v/list"
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
