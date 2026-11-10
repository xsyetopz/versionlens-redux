use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
/// The kind of dependency manifest a document is recognised as.
pub enum ManifestKind {
    /// A Cargo `Cargo.toml` package manifest.
    CargoToml,
    /// A Cargo `config.toml` configuration file.
    CargoConfigToml,
    /// A Composer `composer.json` manifest.
    ComposerJson,
    /// A Deno `deno.json` or `deno.jsonc` configuration file.
    DenoJson,
    /// A Deno import map JSON file.
    DenoImportMapJson,
    /// A JSR `jsr.json` package manifest.
    JsrJson,
    /// A legacy .NET `project.json` manifest.
    DotnetProjectJson,
    /// A .NET XML project or props file such as `.csproj` or `Directory.Packages.props`.
    DotnetXml,
    /// A .NET `dotnet-tools.json` local tool manifest.
    DotnetToolsJson,
    /// A Paket `paket.dependencies` file.
    PaketDependencies,
    /// A Paket `paket.references` file.
    PaketReferences,
    /// A Docker Compose YAML file.
    DockerComposeYaml,
    /// A `Dockerfile`.
    Dockerfile,
    /// A Kustomize `kustomization.yaml` file.
    KustomizationYaml,
    /// A D language `dub.json` package manifest.
    DubJson,
    /// A D language `dub.sdl` package manifest.
    DubSdl,
    /// A Ruby Bundler `Gemfile`.
    Gemfile,
    /// A Ruby `.gemspec` file.
    RubyGemspec,
    /// A Go `go.mod` module file.
    GoMod,
    /// A Maven `pom.xml` project file.
    MavenPomXml,
    /// A Gradle `build.gradle` or `build.gradle.kts` build script.
    GradleBuild,
    /// A Gradle `settings.gradle` or `settings.gradle.kts` script.
    GradleSettings,
    /// A Gradle version catalog TOML file.
    GradleVersionCatalogToml,
    /// An sbt `build.sbt` file.
    SbtBuild,
    /// A Clojure `deps.edn` file.
    ClojureDepsEdn,
    /// A Leiningen `project.clj` file.
    LeiningenProjectClj,
    /// An Elixir `mix.exs` project file.
    MixExs,
    /// An Erlang `rebar.config` file.
    RebarConfig,
    /// A Gleam `gleam.toml` project file.
    GleamToml,
    /// An OCaml opam package file.
    Opam,
    /// An OCaml `dune-project` file.
    DuneProject,
    /// A Haskell `.cabal` package file.
    Cabal,
    /// A Haskell `cabal.project` file.
    CabalProject,
    /// A Haskell Stack `stack.yaml` file.
    StackYaml,
    /// A Julia `Project.toml` file.
    JuliaProjectToml,
    /// A Julia `Manifest.toml` file.
    JuliaManifestToml,
    /// An R package `DESCRIPTION` file.
    RDescription,
    /// An R `renv.lock` lockfile.
    RenvLock,
    /// A Conan `conanfile.txt` file.
    ConanfileTxt,
    /// A Conan `conanfile.py` recipe.
    ConanfilePy,
    /// A vcpkg `vcpkg.json` manifest.
    VcpkgJson,
    /// A `CMake` `CMakeLists.txt` or `.cmake` script.
    Cmake,
    /// An xmake `xmake.lua` build file.
    XmakeLua,
    /// A Meson `.wrap` dependency file.
    MesonWrap,
    /// A Bazel `WORKSPACE` file.
    BazelWorkspace,
    /// A Swift Package Manager `Package.swift` manifest.
    SwiftPackage,
    /// A Carthage `Cartfile`.
    Cartfile,
    /// A Zig `build.zig.zon` package manifest.
    ZigBuildZon,
    /// A Nim `.nimble` package file.
    Nimble,
    /// A `LuaRocks` `.rockspec` file.
    LuaRockspec,
    /// A Perl `cpanfile`.
    Cpanfile,
    /// A Haxe `haxelib.json` manifest.
    HaxelibJson,
    /// A Terraform or `OpenTofu` `.tf` file.
    TerraformTf,
    /// A Helm `Chart.yaml` file.
    HelmChartYaml,
    /// An Ansible Galaxy `requirements.yml` file.
    AnsibleGalaxyRequirementsYaml,
    /// A Bazel `MODULE.bazel` file.
    BazelModule,
    /// A Nix `flake.nix` file.
    NixFlake,
    /// A Unity `Packages/manifest.json` project manifest.
    UnityProjectManifestJson,
    /// A `CocoaPods` `Podfile`.
    CocoaPodsPodfile,
    /// An npm-style `package.json` manifest.
    NpmPackageJson,
    /// An npm-style `package.json5` manifest.
    NpmPackageJson5,
    /// An npm-style `package.yaml` manifest.
    NpmPackageYaml,
    /// A pnpm `pnpm-workspace.yaml` file.
    PnpmYaml,
    /// A Python `Pipfile`.
    PythonPipfile,
    /// A Python `pyproject.toml` file.
    PythonPyprojectToml,
    /// A Python `requirements.txt` file.
    PythonRequirementsTxt,
    /// A Dart `pubspec_overrides.yaml` file.
    PubspecOverridesYaml,
    /// A Dart or Flutter `pubspec.yaml` file.
    PubspecYaml,
    /// A `VersionLens` multi-registry configuration file.
    VersionLensMultiRegistries,
    /// A file that matches no supported manifest kind.
    Unknown,
    /// A GitHub Actions workflow or action file.
    GitHubActions,
    /// A `.nvmrc` Node version file.
    Nvmrc,
    /// A `.node-version` file.
    NodeVersion,
    /// A `.bun-version` file.
    BunVersion,
    /// A plain-text `rust-toolchain` file.
    RustToolchain,
    /// A `rust-toolchain.toml` file.
    RustToolchainToml,
}
