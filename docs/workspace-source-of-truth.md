# Workspace/source-of-truth layer (0.4.0)

The graph is intentionally bounded to `DocumentInput.workspaceRoot`. It reads
only explicit workspace declarations, rejects malformed/missing/duplicate
members, and uses the open document text when the on-disk file is stale. A
local identity must be proven before registry routing is bypassed.

## Supported-parser disposition

| Family / manifests currently parsed | 0.4.0 disposition |
| --- | --- |
| npm/Bun/pnpm/Yarn `package.json`, `pnpm-workspace.yaml` | Workspace discovery and local resolution for package workspaces; `workspace:`, `catalog:`, `file:`, `link:`, and path references are local. Lerna fixed/independent policy is recognized as companion metadata. |
| Cargo `Cargo.toml` | Explicit workspace member discovery and local package identity/version resolution. `workspace = true`, path dependencies, inherited `workspace.package.version`, and `rust-version` remain parser-owned distinctions; coordinated edits are conservative. |
| Gradle settings/build/version catalogs; Maven POM | Existing parser/provider behavior retained. No heuristic graph mutation: project/property/plugin indirection is terminal until an explicit source range is available. |
| Python, .NET, Composer, Pub/Dart, Deno, Go, Hex-family, Ruby, Julia, CRAN, Hackage, Opam, Conan, Vcpkg, Swift, Zig, Nim, LuaRocks, CPAN, Haxelib, Terraform, Helm, Ansible, Bazel, Nix, Unity, CocoaPods, C/C++, Docker, GitHub Actions | Existing standalone parser and provider routing retained. No workspace schema is inferred; path/local references remain parser-owned terminal results rather than being silently discarded. |

This matrix is a disposition, not a promise to infer tool-specific build
evaluation. Unsupported or ambiguous references must remain visible to the
existing terminal behavior.

## Exhaustive `ManifestKind` coverage

The following is the complete parser enum inventory. `A` is workspace-aware
or local graph behavior, `B` is local/path terminal behavior, and `C` is an
explicit safe terminal disposition (the existing parser/provider remains the
owner):

| Disposition | ManifestKind values |
| --- | --- |
| A | `NpmPackageJson`, `NpmPackageJson5`, `NpmPackageYaml`, `PnpmYaml`, `CargoToml` |
| B | `PubspecOverridesYaml`, `PubspecYaml`, `GoMod`, `SwiftPackage`, `UnityProjectManifestJson`, `RubyGemspec`, `Gemfile`, `PaketReferences`, `CocoaPodsPodfile`, `DuneProject` |
| C | `ComposerJson`, `DenoJson`, `DenoImportMapJson`, `JsrJson`, `DotnetProjectJson`, `DotnetXml`, `PaketDependencies`, `DockerComposeYaml`, `Dockerfile`, `KustomizationYaml`, `DubJson`, `DubSdl`, `MavenPomXml`, `GradleBuild`, `GradleSettings`, `GradleVersionCatalogToml`, `SbtBuild`, `ClojureDepsEdn`, `LeiningenProjectClj`, `MixExs`, `RebarConfig`, `GleamToml`, `Opam`, `Cabal`, `CabalProject`, `StackYaml`, `JuliaProjectToml`, `JuliaManifestToml`, `RDescription`, `RenvLock`, `ConanfileTxt`, `ConanfilePy`, `VcpkgJson`, `Cmake`, `XmakeLua`, `MesonWrap`, `BazelWorkspace`, `ZigBuildZon`, `Nimble`, `LuaRockspec`, `Cpanfile`, `HaxelibJson`, `TerraformTf`, `HelmChartYaml`, `AnsibleGalaxyRequirementsYaml`, `BazelModule`, `NixFlake`, `GitHubActions`, `VersionLensMultiRegistries`, `Unknown` |

## Authoritative specifications consulted

* npm package/workspace metadata (npm CLI v12): `https://docs.npmjs.com/cli/v12/configuring-npm/package-json`
* Bun package workspaces: `https://bun.sh/docs/install/workspaces`
* pnpm workspaces/catalogs: `https://pnpm.io/workspaces`, `https://pnpm.io/catalogs`
  and pnpm configuration: `https://pnpm.io/npmrc`
* Yarn workspaces/protocol: `https://yarnpkg.com/features/workspaces`, `https://yarnpkg.com/protocol`
* Lerna versioning: `https://lerna.js.org/docs/features/version-and-publish`
* Cargo workspaces and inherited package metadata: `https://doc.rust-lang.org/cargo/reference/workspaces.html`
* Gradle multi-project/version catalogs: `https://docs.gradle.org/current/userguide/multi_project_builds.html`, `https://docs.gradle.org/current/userguide/version_catalogs.html`
* Maven POM/modules: `https://maven.apache.org/pom.html`, `https://maven.apache.org/guides/mini/guide-multiple-modules.html`
* Python project metadata: `https://packaging.python.org/en/latest/specifications/pyproject-toml/`
* uv workspaces/sources: `https://docs.astral.sh/uv/concepts/projects/workspaces/`, `https://docs.astral.sh/uv/concepts/projects/dependencies/`
* Go workspaces/modules: `https://go.dev/ref/mod#workspaces`, `https://go.dev/ref/mod#go-mod-file`
* Dart/ Pub workspaces: `https://dart.dev/tools/pub/workspaces`
* .NET/MSBuild project references: `https://learn.microsoft.com/en-us/visualstudio/msbuild/common-msbuild-project-items`, `https://learn.microsoft.com/en-us/dotnet/core/project-sdk/msbuild-props`
* Deno workspaces: `https://docs.deno.com/runtime/fundamentals/workspaces/`
* Composer path repositories: `https://getcomposer.org/doc/05-repositories.md#path`
* Mix umbrella applications: `https://hexdocs.pm/mix/Mix.Tasks.New.html#module-umbrella-projects`

Accessed August 29, 2026. These sources establish manifest-owned project
metadata as the source of truth; companion policy tools do not replace it.
