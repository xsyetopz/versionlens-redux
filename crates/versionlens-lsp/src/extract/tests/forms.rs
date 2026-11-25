//! Forms the fork's master text parsers read inside the file types both baselines share, each
//! pinned to what its catalog row extracts: sbt's `%%`, Gradle classifiers and rich versions,
//! untagged and digest-pinned images, `.nvmrc` comments, versionless and sourced gems, go.work
//! directories, `deps.edn` coordinates, Leiningen's own version, Carthage URLs and references,
//! parenthesized requirements, target-suffixed Rust channels, `NuGet` overrides and versionless
//! references, Terraform shorthand providers, rebar elements and plugins, Mix umbrella siblings,
//! kustomize digests, and versionless vcpkg objects.

use super::master::expect;

#[test]
fn reads_sbt_double_percent_with_scala_binary_version() {
    let text = "ThisBuild / scalaVersion := \"2.13.12\"\nlibraryDependencies ++= Seq(\n  \"org.typelevel\" %% \"cats-core\" % \"2.9.0\",\n  \"com.lihaoyi\" %%% \"upickle\" % \"3.1.0\",\n  \"org.slf4j\" % \"slf4j-api\" % \"2.0.9\"\n)\n";
    expect(
        "build.sbt",
        text,
        &[
            ("org.typelevel:cats-core_2.13", "2.9.0", "maven"),
            ("org.slf4j:slf4j-api", "2.0.9", "maven"),
        ],
    );
    let scala3 = "scalaVersion := \"3.3.1\"\nlibraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.10.0\"\n";
    expect(
        "build.sbt",
        scala3,
        &[("org.typelevel:cats-core_3", "2.10.0", "maven")],
    );
    let unknown = "libraryDependencies += \"org.typelevel\" %% \"cats-core\" % \"2.9.0\"\n";
    expect("build.sbt", unknown, &[]);
    // The fork's master fixture `parses-sbt-multiple-dependencies-on-one-line`, with the Scala
    // version `project::extract` appends for a build that sets none.
    let one_line = "libraryDependencies ++= Seq(\"org.zeta\" % \"zeta\" % \"1.0.0\", \"org.alpha\" %% \"alpha\" % \"2.0.0\")\nscalaVersion := \"2.12.x\"\n";
    expect(
        "build.sbt",
        one_line,
        &[
            ("org.zeta:zeta", "1.0.0", "maven"),
            ("org.alpha:alpha_2.12", "2.0.0", "maven"),
        ],
    );
}

#[test]
fn reads_gradle_classifiers_types_and_map_order() {
    let text = "dependencies {\n  implementation 'org.lwjgl:lwjgl:3.3.3:natives-linux'\n  implementation 'com.android.support:appcompat-v7:28.0.0@aar'\n  implementation name: 'guava', group: 'com.google.guava', version: '33.0.0-jre'\n}\n";
    expect(
        "build.gradle",
        text,
        &[
            ("org.lwjgl:lwjgl", "3.3.3", "maven"),
            ("com.android.support:appcompat-v7", "28.0.0", "maven"),
            ("com.google.guava:guava", "33.0.0-jre", "maven"),
        ],
    );
}

#[test]
fn reads_gradle_catalog_rich_versions() {
    let text = "[versions]\nguava = { strictly = \"33.0.0-jre\" }\n\n[libraries]\nguava = { module = \"com.google.guava:guava\", version.ref = \"guava\" }\nslf4j = { module = \"org.slf4j:slf4j-api\", version = { require = \"2.0.9\" } }\njunit = { group = \"junit\", name = \"junit\", version.prefer = \"4.13.2\" }\n";
    expect(
        "gradle/libs.versions.toml",
        text,
        &[
            ("com.google.guava:guava", "33.0.0-jre", "maven"),
            ("org.slf4j:slf4j-api", "2.0.9", "maven"),
            ("junit:junit", "4.13.2", "maven"),
        ],
    );
}

#[test]
fn reads_untagged_images_outside_build_stages() {
    let text = "FROM golang:1.22 AS build\nFROM build AS test\nFROM scratch\nFROM alpine\nFROM localhost:5000/app\nFROM --platform=$BUILDPLATFORM node AS web\n";
    expect(
        "Dockerfile",
        text,
        &[
            ("golang", "1.22", "docker"),
            ("alpine", "", "docker"),
            ("localhost:5000/app", "", "docker"),
            ("node", "", "docker"),
        ],
    );
}

#[test]
fn reads_compose_untagged_digest_and_extension_images() {
    let text = "x-cache: &cache\n  image: redis:7.2\nservices:\n  web:\n    image: nginx\n  db:\n    image: postgres:16.1@sha256:0123abcd\n  cache:\n    <<: *cache\n";
    expect(
        "compose.yaml",
        text,
        &[
            ("redis", "7.2", "docker"),
            ("nginx", "", "docker"),
            ("postgres", "16.1", "docker"),
        ],
    );
}

#[test]
fn reads_node_versions_after_comments() {
    expect(
        ".nvmrc",
        "# team pin\n20.11.1 # LTS\n",
        &[("node", "20.11.1", "npm")],
    );
    expect(".nvmrc", "lts/iron\n", &[]);
}

#[test]
fn reads_gems_without_version_and_from_sources() {
    let text = "gem \"rails\"\ngem 'rspec', group: :test, require: false\ngem \"local\", path: \"../local\"\ngem \"edge\", git: \"https://github.com/o/edge.git\"\ngem \"tagged\", github: \"o/tagged\", tag: \"v1.2.0\"\ngem \"branchy\", :github => \"o/branchy\", :branch => \"main\"\ngem \"private\", source: \"https://gems.example.com\"\n";
    expect(
        "Gemfile",
        text,
        &[
            ("rails", "", "rubygems"),
            ("rspec", "", "rubygems"),
            ("local", "../local", "directory"),
            ("edge", "https://github.com/o/edge.git", "source"),
            ("o/tagged", "v1.2.0", "github"),
            ("o/branchy", "main", "source"),
            ("private", "", "rubygems"),
        ],
    );
    expect(
        "a.gemspec",
        "  s.add_dependency 'rack'\n",
        &[("rack", "", "rubygems")],
    );
}

/// The fork's master fixture `parses-gemfile-git-and-path-block-dependencies-as-non-registry-sources`:
/// a bare gem in a `path` or `git` block comes from the block's source, not the gem server.
#[test]
fn reads_gems_in_path_and_git_blocks_from_their_source() {
    // Arrange
    let text = "path \"vendor/local\" do\n  gem \"local_one\"\nend\n\ngit \"https://github.com/rails/rails.git\", branch: \"main\" do\n  gem \"activesupport\"\n  gem \"actionpack\"\nend\n\ngem \"rack\"\n";
    let file = crate::catalog::file_type_of("/work/Gemfile", text).unwrap();

    // Act
    let deps: Vec<_> = crate::extract::extract(file, text)
        .into_iter()
        .map(|d| (d.name, d.req, d.registry, d.source))
        .collect();

    // Assert
    let rails = || Some("git repository".to_owned());
    assert_eq!(
        deps,
        [
            (
                "local_one".to_owned(),
                String::new(),
                "directory",
                Some("vendor/local".to_owned())
            ),
            ("activesupport".to_owned(), String::new(), "source", rails()),
            ("actionpack".to_owned(), String::new(), "source", rails()),
            ("rack".to_owned(), String::new(), "rubygems", None),
        ]
    );
}

#[test]
fn reads_go_work_directories() {
    let text = "go 1.22\n\nuse ./api\nuse (\n\t./web\n\t../shared\n)\n";
    expect(
        "go.work",
        text,
        &[
            ("go", "1.22", "golang"),
            ("./api", "./api", "directory"),
            ("./web", "./web", "directory"),
            ("../shared", "../shared", "directory"),
        ],
    );
}

#[test]
fn reads_deps_edn_coordinates_in_any_form() {
    let text = "{:deps {cheshire {:mvn/version \"5.12.0\"}\n        a/b$linux {:exclusions [c/d] :mvn/version \"1.0\"}\n        my/lib {:local/root \"../lib\"}\n        io.github.o/x {:git/tag \"v0.1.0\" :git/sha \"abc1234\"}}}\n";
    expect(
        "deps.edn",
        text,
        &[
            ("cheshire", "5.12.0", "clojars"),
            ("a/b", "1.0", "clojars"),
            ("my/lib", "../lib", "directory"),
            ("io.github.o/x", "v0.1.0", "source"),
        ],
    );
}

#[test]
fn reads_leiningen_own_version_and_quoted_names() {
    let text = "(defproject app \"0.1.0-SNAPSHOT\"\n  :dependencies [[\"org.clojure/clojure\" \"1.11.1\"]])\n";
    expect(
        "project.clj",
        text,
        &[
            ("leiningen", "0.1.0-SNAPSHOT", "project"),
            ("org.clojure/clojure", "1.11.1", "clojars"),
        ],
    );
}

#[test]
fn reads_carthage_urls_and_references() {
    let text = "git \"https://github.com/ReactiveX/RxSwift.git\" ~> 6.0\ngithub \"git@github.com:Alamofire/Alamofire.git\" >= 5.0\ngithub \"o/edge\" \"main\"\n";
    expect(
        "Cartfile",
        text,
        &[
            ("ReactiveX/RxSwift", "~> 6.0", "github"),
            ("Alamofire/Alamofire", ">= 5.0", "github"),
            ("o/edge", "main", "source"),
        ],
    );
}

#[test]
fn reads_parenthesized_requirements() {
    expect(
        "requirements.txt",
        "requests (>=2.0,<3)\n",
        &[("requests", ">=2.0,<3", "pypi")],
    );
}

#[test]
fn reads_each_version_bazel_lets_coexist() {
    let text = "bazel_dep(name = \"protobuf\", version = \"27.0\")\nmultiple_version_override(\n    module_name = \"protobuf\",\n    versions = [\"21.7\", \"27.0\"],\n)\n";
    expect(
        "MODULE.bazel",
        text,
        &[
            ("protobuf", "27.0", "bazel"),
            ("protobuf", "21.7", "bazel"),
            ("protobuf", "27.0", "bazel"),
        ],
    );
}

#[test]
fn reads_stack_snapshots_of_every_kind() {
    expect(
        "stack.yaml",
        "resolver: nightly-2026-05-29\n",
        &[("nightly", "2026-05-29", "stackage-nightly")],
    );
    expect(
        "stack.yaml",
        "snapshot: ghc-9.10.3\n",
        &[("ghc", "9.10.3", "ghc")],
    );
    let url = "https://raw.githubusercontent.com/commercialhaskell/stackage-snapshots/master/lts/24/43.yaml";
    expect(
        "stack.yaml",
        &format!("snapshot:\n  url: {url}\n  size: 729011\n"),
        &[("snapshot", url, "source")],
    );
    expect(
        "stack.yaml",
        "resolver: github:o/snaps:lts/1/2.yaml\n",
        &[("resolver", "github:o/snaps:lts/1/2.yaml", "source")],
    );
}

#[test]
fn keeps_opam_operator_outside_span() {
    // opam.ocaml.org/doc/Manual.html#Filtered-package-formulas: `{>= "1.0"}` is a lower bound,
    // and `{= "1.0"}` or a bare constraint-free version pins, as the fork's master read `= 1.0.0`.
    use crate::version::Bare;
    let text = "depends: [\n  \"lwt\" {>= \"5.0\"}\n  \"tls\" {= \"1.0.0\"}\n  \"ssl\" {> \"0.5\" & < \"1.0\"}\n]\n";
    let file = crate::catalog::file_type("/work/demo.opam").unwrap();
    let opam = crate::registry::get("opam").unwrap();
    let read: Vec<(String, String, Bare)> = super::extract(file, text)
        .into_iter()
        .map(|d| {
            let bare = d.bare(opam);
            (d.name, d.req, bare)
        })
        .collect();
    assert_eq!(
        read,
        [
            ("lwt".to_owned(), "5.0".to_owned(), Bare::Min),
            ("tls".to_owned(), "1.0.0".to_owned(), Bare::Exact),
            ("ssl".to_owned(), "0.5".to_owned(), Bare::Min),
        ]
    );
}

#[test]
fn reads_rust_channels_with_host() {
    expect(
        "rust-toolchain.toml",
        "[toolchain]\nchannel = \"1.75.0-x86_64-apple-darwin\"\n",
        &[("rust", "1.75.0", "rust")],
    );
}

/// The fork's master fixture `cargo-workspace-inherited-dependencies`: a member's inherited
/// dependency names the workspace, and a package field's `version.workspace` is no dependency.
#[test]
fn reads_cargo_workspace_inherited_dependencies() {
    // Arrange
    let text = "[package]\nname = \"member\"\nversion.workspace = true\n\n[dependencies]\nregex = { workspace = true, features = [\"unicode\"] }\n\n[build-dependencies]\ncc.workspace = true\n\n[target.'cfg(unix)'.dev-dependencies]\nrand = { optional = true, workspace = true }\n";
    let file = crate::catalog::file_type("/work/member/Cargo.toml").unwrap();

    // Act
    let deps: Vec<_> = crate::extract::extract(file, text)
        .into_iter()
        .map(|d| (d.name, d.registry, d.source))
        .collect();

    // Assert
    let workspace = || Some("workspace package".to_owned());
    assert_eq!(
        deps,
        [
            ("regex".to_owned(), "source", workspace()),
            ("cc".to_owned(), "source", workspace()),
            ("rand".to_owned(), "source", workspace()),
        ]
    );
}

/// The fork's master fixture `parses-cocoapods-podfile-dependencies`: every `pod` form, each from
/// where its options say it comes from.
#[test]
fn reads_pods_without_requirement_and_from_their_source() {
    let text = "source 'https://cdn.cocoapods.org/'\ntarget 'App' do\n  pod 'AFNetworking', '~> 4.0'\n  pod 'SSZipArchive'\n  pod 'LocalPod', :path => '../LocalPod'\n  pod 'GitPod', :git => 'https://github.com/acme/GitPod.git', :tag => '1.2.3'\n  pod 'Edge', :git => 'https://example.com/Edge.git', :branch => 'dev'\n  pod 'JSONKit', :podspec => 'https://example.com/JSONKit.podspec'\n  pod 'PonyDebugger', :source => 'https://private.example.com/specs'\nend\n";
    expect(
        "Podfile",
        text,
        &[
            ("AFNetworking", "~> 4.0", "cocoapods"),
            ("SSZipArchive", "", "cocoapods"),
            ("../LocalPod", "../LocalPod", "directory"),
            ("acme/GitPod", "1.2.3", "github"),
            ("Edge", "https://example.com/Edge.git", "source"),
            ("JSONKit", "https://example.com/JSONKit.podspec", "source"),
            ("PonyDebugger", "", "cocoapods"),
        ],
    );
}

/// The fork's master fixtures `parses-cpp-cmake-dependencies` and `parses-cpp-xmake-dependencies`:
/// a commented-out call stays unread, and each package `add_requires` lists is read.
#[test]
fn skips_commented_cmake_and_xmake_calls() {
    let cmake = "FetchContent_Declare(\n  fmt\n  GIT_REPOSITORY https://github.com/fmtlib/fmt.git\n  GIT_TAG 10.2.1\n)\n# FetchContent_Declare(commented_fmt GIT_REPOSITORY https://github.com/fmtlib/fmt.git GIT_TAG 1.0.0)\nCPMAddPackage(\n  NAME spdlog\n  GITHUB_REPOSITORY gabime/spdlog\n  VERSION 1.14.1\n)\n";
    expect(
        "CMakeLists.txt",
        cmake,
        &[
            ("fmtlib/fmt", "10.2.1", "github"),
            ("gabime/spdlog", "1.14.1", "github"),
        ],
    );
    let xmake = "add_requires(\"zlib 1.3.1\", \"fmt 10.2.1\")\nadd_requires(\"openssl\", {configs = {shared = \"yes\"}})\n-- add_requires(\"commented 1.0.0\")\n";
    expect(
        "xmake.lua",
        xmake,
        &[
            ("zlib", "1.3.1", "xmake"),
            ("fmt", "10.2.1", "xmake"),
            ("openssl", "", "xmake"),
        ],
    );
}

/// The fork's master fixtures for workflows, Julia manifests, rockspecs, and flakes: local actions,
/// development checkouts, and `path:` inputs link to their directory; Lua is a GitHub release; and
/// a branch input shows its head commit.
#[test]
fn reads_local_actions_dev_packages_lua_and_flake_branches() {
    expect(
        ".github/workflows/ci.yml",
        "jobs:\n  shared:\n    uses: ./.github/workflows/shared.yml\n  build:\n    steps:\n      - uses: ./actions/build\n      - uses: ${{ matrix.action }}\n",
        &[
            (
                "./.github/workflows/shared.yml",
                "./.github/workflows/shared.yml",
                "directory",
            ),
            ("./actions/build", "./actions/build", "directory"),
        ],
    );
    expect(
        "Manifest.toml",
        "[[deps.LocalPkg]]\npath = \"/home/user/.julia/dev/LocalPkg\"\nuuid = \"22222222-2222-2222-2222-222222222222\"\nversion = \"0.1.0\"\n",
        &[(
            "/home/user/.julia/dev/LocalPkg",
            "/home/user/.julia/dev/LocalPkg",
            "directory",
        )],
    );
    expect(
        "demo-1.0.0-1.rockspec",
        "dependencies = {\n   \"lua >= 5.1, < 5.5\",\n   \"luasocket == 3.1.0\",\n}\n",
        &[
            ("lua/lua", ">= 5.1, < 5.5", "github"),
            ("luasocket", "== 3.1.0", "luarocks"),
        ],
    );
    expect(
        "flake.nix",
        "{\n  inputs = {\n    nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n    stable.url = \"github:NixOS/nixpkgs/nixos-24.05\";\n    local.url = \"path:../local\";\n  };\n}\n",
        &[
            ("github.com/NixOS/nixpkgs", "nixos-unstable", "git-ref"),
            ("NixOS/nixpkgs", "24.05", "github"),
            ("../local", "../local", "directory"),
        ],
    );
}

/// The fork's master fixtures for .NET projects: central package management's `VersionOverride`,
/// and a reference whose version `Directory.Packages.props` sets, read without a requirement.
#[test]
fn reads_nuget_version_overrides_and_versionless_references() {
    // Arrange
    let text = "<Project Sdk=\"Microsoft.NET.Sdk\">\n  <ItemGroup>\n    <PackageReference Include=\"Serilog\" Version=\"3.1.1\" />\n    <PackageReference Include=\"Newtonsoft.Json\" VersionOverride=\"13.0.3\" />\n    <PackageReference Include=\"Polly\" />\n    <PackageReference Include=\"xunit\">\n      <PrivateAssets>all</PrivateAssets>\n    </PackageReference>\n  </ItemGroup>\n</Project>\n";

    // Act and Assert
    expect(
        "app.csproj",
        text,
        &[
            ("Serilog", "3.1.1", "nuget"),
            ("Newtonsoft.Json", "13.0.3", "nuget"),
            ("Polly", "", "nuget"),
            ("xunit", "", "nuget"),
        ],
    );
}

/// The fork's master fixture `parses-terraform-dependencies`: Terraform 0.12's shorthand
/// `aws = "~> 5.0"` names a provider in the `hashicorp` namespace.
#[test]
fn reads_terraform_shorthand_providers() {
    // Arrange
    let text = "terraform {\n  required_providers {\n    aws = \"~> 5.0\"\n    google = {\n      source  = \"hashicorp/google\"\n      version = \"~> 6.0\"\n    }\n  }\n}\n";

    // Act and Assert
    expect(
        "main.tf",
        text,
        &[
            ("hashicorp/aws", "~> 5.0", "terraform"),
            ("hashicorp/google", "~> 6.0", "terraform"),
        ],
    );
}

/// Each element shape rebar3.org/docs/configuration/dependencies/ and plugins/ list, in `deps`,
/// `plugins`, and `project_plugins`; the `{tag, ...}` inside a source is no package.
#[test]
fn reads_rebar_dependency_and_plugin_shapes() {
    // Arrange
    let text = "{deps, [\n  cowboy,\n  {jsx, \"3.1.0\"},\n  {hackney, {pkg, hackney_fork}},\n  {gun, \"2.0.0\", {pkg, gun_fork}},\n  {ranch, {git, \"https://github.com/ninenines/ranch.git\", {tag, \"2.1.0\"}}}\n]}.\n{plugins, [{rebar3_hex, \"7.0.7\"}]}.\n{project_plugins, [erlfmt]}.\n";

    // Act
    let names: Vec<_> = super::found("rebar.config", text)
        .into_iter()
        .map(|(name, req, _)| (name, req))
        .collect();

    // Assert
    assert_eq!(
        names,
        super::pairs(&[
            ("cowboy", ""),
            ("jsx", "3.1.0"),
            ("hackney_fork", ""),
            ("gun_fork", "2.0.0"),
            ("ranch", "https://github.com/ninenines/ranch.git"),
            ("rebar3_hex", "7.0.7"),
            ("erlfmt", ""),
        ])
    );
}

/// hexdocs.pm/mix/Mix.Tasks.Deps.html: an option on the line after the name still reads, and an
/// umbrella sibling `in_umbrella: true` is a workspace package.
#[test]
fn reads_multiline_mix_options_and_umbrella_siblings() {
    // Arrange
    let text = "defp deps do\n  [\n    {:plug,\n     git: \"https://github.com/elixir-plug/plug.git\"},\n    {:shared, in_umbrella: true}\n  ]\nend\n";

    // Act
    let labels = super::sources("mix.exs", text);

    // Assert
    assert_eq!(
        labels,
        super::pairs(&[("plug", "git repository"), ("shared", "workspace package"),])
    );
}

/// kubectl.docs.kubernetes.io/references/kustomize/kustomization/images/: an image pinned by
/// `digest` alone is read by name.
#[test]
fn reads_kustomize_digest_images() {
    // Arrange
    let text = "images:\n  - name: postgres\n    digest: sha256:24a0c4b4a4c0eb97a1aabb8e29f18e917d05abfe1b7a7c07857230879ce7d3d3\n  - name: nginx\n    newTag: 1.27.0\n";

    // Act
    let names: Vec<_> = super::found("kustomization.yaml", text)
        .into_iter()
        .map(|(name, _, registry)| (name, registry))
        .collect();

    // Assert
    assert!(
        names.contains(&("postgres".to_owned(), "docker")),
        "{names:?}"
    );
    assert!(names.contains(&("nginx".to_owned(), "docker")), "{names:?}");
}

/// learn.microsoft.com/en-us/vcpkg/reference/vcpkg-json#dependency-object: an object with a name
/// and features but no `version>=` is a dependency on the baseline's version.
#[test]
fn reads_versionless_vcpkg_objects() {
    // Arrange
    let text = "{\n  \"dependencies\": [\n    \"fmt\",\n    { \"name\": \"boost-asio\", \"features\": [\"ssl\"] },\n    { \"name\": \"zlib\", \"version>=\": \"1.3.1\" }\n  ]\n}\n";

    // Act
    let names: Vec<_> = super::found("vcpkg.json", text)
        .into_iter()
        .map(|(name, req, _)| (name, req))
        .collect();

    // Assert
    assert!(
        names.contains(&("boost-asio".to_owned(), String::new())),
        "{names:?}"
    );
    assert!(
        names.contains(&("zlib".to_owned(), "1.3.1".to_owned())),
        "{names:?}"
    );
}
