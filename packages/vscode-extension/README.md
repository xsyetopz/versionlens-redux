# VersionLens for Visual Studio Code

VersionLens shows package version information directly in VS Code dependency manifests, using a Rust core for parsing, version comparison, and registry lookups.

[![CI](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml/badge.svg)](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/xsyetopz/vscode-versionlens?include_prereleases)](https://github.com/xsyetopz/vscode-versionlens/releases)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

![Show releases](images/faq/show-releases.gif)

## Use it when

- Show CodeLens version hints while editing dependency files.
- Mark dependencies as current, outdated, fixed, or constrained.
- Show prerelease versions on demand.
- Surface OSV.dev vulnerability warnings in the editor.

## Supported languages and ecosystems

Supported manifest ecosystems:

| Ecosystem | Registry/source |
| --- | --- |
| Cargo / Rust | crates.io |
| Conan / C and C++ | ConanCenter and Conan-compatible v2 remotes |
| C/C++ build files | GitHub tags and Xmake package recipes |
| CPAN / Perl | MetaCPAN |
| Composer / PHP | Packagist |
| Deno | Deno, JSR, npm |
| Docker / Compose / OCI images | Docker Hub, Microsoft Container Registry, and OCI-compatible registries |
| .NET / F# / NuGet / Paket | NuGet v3 service indexes and Paket source declarations |
| Dub / D | code.dlang.org |
| Go | proxy.golang.org |
| Haskell / Cabal / Stack | Hackage |
| Hex / BEAM | Hex.pm and configured Hex-compatible repositories |
| Julia | Julia General registry |
| LuaRocks / Lua | LuaRocks manifests |
| Maven / Java | Maven Central |
| Nim / Nimble | Nimble packages index and GitHub tags |
| npm / Bun / pnpm / JSPM | npm-compatible registries |
| OCaml / opam | opam.ocaml.org package pages |
| Pub / Dart / Flutter | pub.dev and hosted Pub repositories |
| Python | PyPI |
| R / CRAN | CRAN package indexes |
| Ruby | RubyGems |
| Swift Package Manager | Swift package registries and GitHub tags |
| Terraform / OpenTofu | Terraform Registry protocol |
| Helm | Helm chart repositories and OCI registries |
| Ansible Galaxy | Ansible Galaxy API for collections and roles |
| Bazel / Bzlmod | Bazel Central Registry metadata and configured Bazel registries |
| Nix flakes | GitHub tags for GitHub-backed flake inputs |
| Kustomize | Container image registries through Docker/OCI tag lookup |
| Unity Package Manager | Unity npm-compatible registries and scoped registries |
| vcpkg / C and C++ | vcpkg registry version database |
| Zig | Zig package URLs and GitHub tags |
| Haxelib / Haxe | Haxelib project versions |
| CocoaPods / iOS and macOS | CocoaPods Specs repositories |

TOML support requires a VS Code TOML language extension such as Even Better TOML.

### Cargo / Rust support

Scans `Cargo.toml` files. It reads package `version`, `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]`, target-specific dependency tables under `[target.*.dependencies]`, `[target.*.dev-dependencies]`, and `[target.*.build-dependencies]`, plus `[workspace.dependencies]`. Inline dependency tables with `package` use the renamed package identity for registry lookup while keeping the local dependency key as the displayed name. Dependencies inherited with `workspace = true`, local `path` dependencies, and Git dependencies are fixed sources without registry update suggestions.

Cargo dependencies query `https://crates.io/api/v1/crates/{name}/versions`. Set `versionlens.cargo.apiUrl` for a different crates.io-compatible API base. Workspace `.cargo/config.toml` and `.cargo/config` registry entries are honored, including named registries, sparse registry URLs, and crates.io source replacement. OSV vulnerability lookups use the `crates.io` ecosystem and Cargo Package URL identity. Sorting is enabled for Cargo dependency tables.

### CPAN / Perl support

Scans `cpanfile` files. It reads `requires`, `recommends`, `suggests`, and `conflicts` declarations, phase blocks such as `on 'test' => sub { ... }`, feature blocks, and shortcut declarations such as `build_requires` and `test_requires`. Edits preserve CPAN version range operators such as `>=`, `<=`, `==`, and `!=` when parser support allows a single editable version segment.

CPAN dependencies query MetaCPAN's `download_url/{module}` endpoint at `https://fastapi.metacpan.org/v1`. Set `versionlens.cpan.apiUrl` for a different MetaCPAN-compatible API base. OSV vulnerability lookups use the `CPAN` ecosystem and CPAN Package URL identity. Sorting is disabled for cpanfile manifests.

### Composer / PHP support

Scans `composer.json` files. It reads package `version`, `require`, `require-dev`, `conflict`, `replace`, and `provide` links. Composer platform packages such as `php` and `ext-*` are fixed platform requirements without registry updates. Local `path` packages and repository URL objects are fixed/local sources. Edits preserve Composer constraints, stability flags such as `@beta`, commit references such as `#abc123`, and inline aliases such as `dev-bugfix as 1.0.x-dev`.

Composer dependencies query Packagist-compatible metadata under `https://repo.packagist.org/p2`. Set `versionlens.composer.apiUrl` for a different Packagist-compatible API base. `repositories` entries with Composer repository URLs, `only`/`exclude` filters, package repositories with inline package versions, disabled Packagist configuration, and `auth.json` HTTP basic or bearer credentials are honored when resolving registry URLs. Branch aliases in Composer metadata are considered when selecting latest versions. OSV vulnerability lookups use the `Packagist` ecosystem and Composer Package URL identity. Sorting is enabled for Composer dependency link groups.

### Deno / JSR / npm import support

Scans `deno.json`, `deno.jsonc`, `import_map.json`, `jsr.json`, and `jsr.jsonc` files. Deno config and import-map files read `version`, `imports`, and `scopes`; `jsr.json` and `jsr.jsonc` read the package `name` plus project `version`. `jsr:` and `npm:` import specifiers use the embedded package identity for lookup, including unversioned specifiers where an update can insert the missing version. Directory import-map specifiers preserve their slash style when edited. Non-`jsr:` and non-`npm:` Deno imports are left without registry suggestions.

JSR dependencies query `https://jsr.io/{scope}/{package}/meta.json` through `versionlens.deno.apiUrl`. `npm:` imports use npm-compatible registry resolution and workspace npm/Yarn/pnpm/Bun registry configuration where available. Edits preserve the `jsr:` or `npm:` scheme, package alias, and specifier form. OSV.dev does not cover JSR packages, so JSR import-map dependencies skip vulnerability lookups. Sorting is enabled for `imports` and each `scopes.*` import group.

### Docker / Compose / OCI image support

Scans `Dockerfile`, `dockerfile`, `*.dockerfile`, `*.Dockerfile`, `compose.yaml`, `compose.yml`, `docker-compose.yaml`, `docker-compose.yml`, and compose override/variant YAML files. Dockerfiles read every `FROM` image, including `--platform`, aliases with `AS`, missing tags, explicit tags, and digest-pinned images. Compose files read service `image` values, service `build` contexts as fixed local build paths, and top-level `x-*` extension image fields where the extension shape contains reusable service-like image definitions. Namespace-qualified Docker Hub images such as `library/nginx:1.25` keep the full repository name instead of treating `library` as a registry.

Docker Hub images query Docker Hub tag pages, `mcr.microsoft.com/...` images query the Microsoft Container Registry tag catalog, and explicit registry images such as `ghcr.io/org/app:1.2.3` or `my_private.registry:5000/redis` use the OCI Distribution `/v2/{name}/tags/list` route. Digest-only pins are reported as fixed and are not rewritten to tags. Images containing unresolved Dockerfile or Compose variables are unsupported without registry suggestions. OSV.dev does not cover container images, so Docker image references skip vulnerability lookups. Sorting is disabled for Dockerfile and Compose manifests because declaration order affects build stages and service semantics.

### .NET / F# / NuGet / Paket support

Scans `.csproj`, `.fsproj`, `.vbproj`, `.props`, `.targets`, `Directory.Packages.props`, `packages.config`, `project.json`, `paket.dependencies`, and `paket.references` files. Project XML files read `Sdk` versions, project `Version` and `AssemblyVersion` properties, `PackageReference` attributes and child `<Version>` nodes, `VersionOverride` item updates, `PackageVersion`, `GlobalPackageReference`, and `DotNetCliToolReference` items, including references inside conditioned item groups. `packages.config` reads `<package id="..." version="..." />` entries. `paket.dependencies` reads `nuget` lines and `source` declarations; `paket.references` entries are fixed project references because they do not carry editable versions.

NuGet dependencies use NuGet v3 service indexes, with `https://api.nuget.org/v3/index.json` as the default source. Set `versionlens.dotnet.nuget.sources` with v3 service index URLs to override the discovered sources. Workspace `NuGet.config` files are read nearest-first for package sources, source clearing/removal, package source mapping, and request-scoped credentials; local file/package-folder sources are ignored for registry suggestions. Paket `source` lines add HTTP(S) NuGet sources for Paket manifests. Four-segment NuGet assembly-style versions and unresolved local/file sources are left without destructive update suggestions. OSV vulnerability lookups use the NuGet ecosystem and NuGet Package URL identity. Sorting is enabled for safe package item groups such as `PackageReference`, `PackageVersion`, `GlobalPackageReference`, and `DotNetCliToolReference`; it does not sort across separate item groups.

### Dub / D support

Scans `dub.json`, `dub.selections.json`, and `dub.sdl` files. `dub.json` reads top-level `dependencies`, selected `versions`, and per-configuration dependency maps under `configurations.*.dependencies`. Dependency sub-objects with `version`, `path`, or `repository` fields are parsed from their editable source ranges. `dub.sdl` reads `dependency` directives with `version`, `path`, or `repository` attributes. Local `path` dependencies and direct repository dependencies are fixed sources without registry update suggestions.

Dub dependencies query `https://code.dlang.org/api/packages/{name}/info?minimize=true`. Set `versionlens.dub.apiUrl` for a different Dub-compatible package API base. OSV.dev does not cover Dub packages, so Dub dependencies skip vulnerability lookups. Sorting is enabled for Dub `dependencies` and `versions` groups.

### Haxelib / Haxe support

Scans `haxelib.json` files. It reads the top-level `dependencies` object where dependency names are keys and values are exact Haxelib versions or an empty string for Haxelib's latest-version behavior. Edits replace only the version value for pinned dependencies.

Haxelib dependencies query the project versions page under `https://lib.haxe.org/p/{name}/versions/`. Set `versionlens.haxelib.apiUrl` for a different Haxelib-compatible base URL. OSV.dev does not cover Haxelib packages, so Haxelib dependencies skip vulnerability lookups. Sorting is disabled for `haxelib.json` manifests.

### Terraform / OpenTofu support

Scans `.tf` and `.tofu` files. It reads provider requirements from `terraform.required_providers`, including object requirements with `source` and `version` attributes and legacy string requirements such as `aws = "~> 5.0"`. Provider sources without an explicit hostname default to `registry.terraform.io`; omitted sources use `registry.terraform.io/hashicorp/{local_name}`. Built-in providers such as `terraform.io/builtin/terraform` are fixed built-in providers without registry update suggestions.

Terraform and OpenTofu providers query the Terraform Registry protocol `GET /v1/providers/{namespace}/{type}/versions` endpoint at `https://registry.terraform.io`. Set `versionlens.terraform.apiUrl` for a different compatible registry base. Provider sources with an explicit hostname use that hostname for lookup. Edits preserve HCL version constraint operators such as `~>`, `>=`, `<=`, and `!=` when parser support exposes a single editable version segment. OSV.dev does not cover Terraform providers, so Terraform dependencies skip vulnerability lookups. Sorting is disabled for Terraform and OpenTofu HCL manifests.

### Helm support

Scans `Chart.yaml` files. It reads chart dependencies from the top-level `dependencies` list, including `name`, `version`, `repository`, and `alias`. Dependencies using an alias display the alias while using the original chart name for registry lookup. Local `file:` repositories and repository aliases such as `@repo` are fixed sources without registry update suggestions.

HTTP(S) Helm repositories query the repository `index.yaml` file, using the dependency `repository` URL when present and `https://charts.bitnami.com/bitnami` as the default `versionlens.helm.apiUrl` base otherwise. OCI chart repositories query the OCI Distribution tags route for the chart path. Edits preserve Helm semver constraint operators such as `~` by replacing only the editable version segment. OSV.dev does not cover Helm charts, so Helm dependencies skip vulnerability lookups. Sorting is disabled for Helm `Chart.yaml` manifests.

### Ansible Galaxy support

Scans Ansible Galaxy `requirements.yml` and `requirements.yaml` files. It reads `roles` and `collections` entries, extracts `name`/`src`, `version`, and collection `source` fields, and skips `include` entries. Galaxy collection lookups use the configured Ansible Galaxy API base URL from `versionlens.ansible.apiUrl`; Galaxy role lookups use the role search API. Git or URL role sources are fixed git repositories and are not updated. Sorting is disabled; requirements order can be meaningful. OSV vulnerability lookup is unsupported for Ansible Galaxy packages.

### Bazel support

Scans Bazel Bzlmod `MODULE.bazel` manifests. It reads `bazel_dep`, `single_version_override`, and `multiple_version_override` version declarations, preserves the Starlark string version range for updates, and uses Bazel registry `metadata.json` version lists from `versionlens.bazel.apiUrl` or the dependency registry override. `git_override`, `archive_override`, and `local_path_override` entries are reported as fixed non-registry dependencies and are not updated. Sorting is disabled; directive order and override grouping can carry meaning. OSV vulnerability lookup is unsupported for Bazel modules.

### Nix flakes support

Scans `flake.nix` files. It reads Nix flake `inputs` URL declarations, including `github:owner/repo/ref`, `github:owner/repo?ref=...`, and `github:owner/repo?rev=...` forms. GitHub-backed inputs use the repository for lookup while preserving the input alias and editable ref segment. Inputs that follow another input, local path inputs, file inputs, and generic Git URLs are reported as fixed sources without registry update suggestions.

Nix flake inputs query GitHub repository tags through `https://api.github.com/repos`. Set `versionlens.nix.apiUrl` for a different GitHub-compatible API base. Edits preserve common Nix channel prefixes such as `nixos-` and `release-` by replacing only the comparable release segment when parser support exposes one. Sorting is disabled; flake input order and grouping can carry project meaning. OSV vulnerability lookup is unsupported for Nix flake inputs.

### Kustomize support

Scans `kustomization.yaml`, `kustomization.yml`, and `Kustomization` files. It reads the Kustomize `images` list, including `name`, `newName`, `newTag`, and `digest` fields. Image replacements with `newName` use the replacement image for registry lookup while preserving the original `name` as alias metadata. Digest-pinned images are treated as fixed image digests and do not receive tag update suggestions.

Kustomize image updates use the existing Docker/OCI image lookup behavior for Docker Hub and explicit registries. Edits replace only the `newTag` value or the editable tag segment in scalar image entries. Sorting is disabled; image replacement order can carry project meaning. OSV vulnerability lookup is unsupported for Kustomize image entries.

### Unity Package Manager support

Scans Unity project `Packages/manifest.json` files. It reads the project manifest `dependencies` object, where package names map to requested versions. `file:` local folders and tarballs, Git URLs, and generic URL dependencies are fixed sources without registry update suggestions.

Unity package dependencies use npm-compatible registry metadata. The default registry base is `https://packages.unity.com`; set `versionlens.unity.apiUrl` for a different Unity-compatible registry. Project `scopedRegistries` entries are honored by matching the closest package-name scope and using that registry URL for the matching dependency. Sorting is disabled; Unity project manifest dependency order can be project-maintained. OSV vulnerability lookup is unsupported for Unity Package Manager packages.

### Go support

Scans `go.mod` and `go.work` files. It reads `require`, `replace`, and `exclude` directives from `go.mod`, including factored directive blocks and quoted module or version tokens. In `go.work`, local `use` directories are fixed workspace directories, and `replace` directives follow the same local-directory handling as `go.mod`. `exclude` entries are fixed excluded versions instead of registry update suggestions.

Go dependencies query the configured module proxy using the Go module proxy protocol at `https://proxy.golang.org/{base-module}/@v/list`, with an `@latest` fallback for compatible proxy URLs. Module proxy paths are case-encoded using Go's `!` escape for uppercase letters. Set `versionlens.golang.apiUrl` with a `{base-module}` template for a different module proxy. Workspace `.env` `GOPROXY` entries add proxy URLs; `GOPROXY=direct` and `GOPROXY=off` disable public proxy lookups, and `GONOPROXY` or `GOPRIVATE` path patterns suppress proxy lookups for matching private modules. Retracted and deprecated versions are ignored when that metadata is present in proxy JSON responses. OSV vulnerability lookups use the `Go` ecosystem and Go Package URL identity.

### Maven and Gradle version catalog support

Scans `pom.xml`, `build.gradle`, `build.gradle.kts`, `settings.gradle`, `settings.gradle.kts`, `gradle/libs.versions.toml`, `build.sbt`, `deps.edn`, and `project.clj` files. Maven POMs read project and parent coordinates, regular dependencies, `dependencyManagement` entries including imported BOMs, build plugins, plugin management, and the same dependency/plugin sections inside profiles. Maven property interpolation resolves project and parent coordinate properties when building lookup identities. Gradle build and settings scripts read plugin DSL versions plus Maven module dependencies in single-string and `group`/`name`/`version` map notation; local `project(...)`, `files(...)`, and `fileTree(...)` dependencies are fixed local sources without registry updates. Sorting is disabled for Maven POMs, Gradle build scripts, and Gradle settings scripts because declaration order can affect build behavior. Gradle version catalogs read `[versions]`, `[libraries]`, and `[plugins]` entries, including string module notation, `group`/`name` objects, direct `version` values, nested version objects such as `prefer`, and `version.ref` references. Version aliases and `version.ref` entries are fixed catalog references because they do not identify a single registry artifact by themselves. sbt build files read `libraryDependencies` entries using `%` and `%%`; `%%` dependencies use the declared `scalaVersion` binary version when present and are fixed when that binary version cannot be resolved from the build file. Clojure `deps.edn` files read `:deps` and alias `:extra-deps`, `:override-deps`, `:default-deps`, and `:replace-deps` maps; Git and local coordinates are fixed sources without registry updates. Leiningen `project.clj` files read the project version plus `:dependencies`, `:managed-dependencies`, `:plugins`, and profile dependency vectors; symbol and string dependency coordinates use Maven-compatible package identity.

Maven-family dependencies use Maven Central-compatible metadata under the configured `versionlens.maven.apiUrl` base. POM repositories, plugin repositories, Maven `settings.xml` active profiles, mirrors, and credentials are honored where they can be resolved from the workspace. Gradle Maven repositories and plugin-management repositories are used for Gradle scripts and plugin marker artifacts; Clojure and Leiningen manifests query Maven Central before Clojars-compatible metadata. OSV vulnerability lookups use the `Maven` ecosystem and Maven Package URL identity.

### npm / pnpm / Yarn / Bun support

Scans `package.json`, `package.json5`, `package.yaml`, `package.yml`, `pnpm-workspace.yaml`, `pnpm-workspace.yml`, `.yarnrc.yaml`, and `.yarnrc.yml` files. Package manifests read project `version`, `packageManager`, Bun-compatible `devEngines.packageManager`, dependency groups, optional and peer dependency groups, `bundleDependencies`/`bundledDependencies`, Bun `trustedDependencies`, Yarn `resolutions`, npm and pnpm `overrides`, JSPM dependency groups, pnpm `packageExtensions`, pnpm catalogs, and Bun workspace catalogs. Name-only bundled/trusted dependency arrays are fixed metadata without registry updates. Workspace catalog references, npm package-manager pins, local paths, `file:`, `link:`, `workspace:`, `exec:`, `patch:`, unsupported Git refs, and override self-references that do not identify a registry version do not get destructive registry suggestions. npm aliases such as `npm:target@1.2.3` use the target package for lookup while preserving the alias form when edited.

Workspace YAML files read pnpm `catalog`, named `catalogs`, `overrides`, and `packageExtensions` dependency groups. Yarn `.yarnrc.yml` files are used for npm registry configuration, including `npmRegistryServer` and scoped `npmScopes`; they are not dependency manifests by themselves. npm-compatible dependencies query the configured registry package endpoint, with `https://registry.npmjs.org` as the default base. Workspace `.npmrc`, package `.npmrc`, Yarn `.yarnrc.yml`, and Bun `bunfig.toml` registry settings are honored for default registries, scoped registries, auth tokens, and request-scoped HTTP settings where configured. npm dist-tags are resolved from registry `dist-tags`, prerelease filtering follows VersionLens provider settings, OSV vulnerability lookups use the `npm` ecosystem and npm Package URL identity, and sorting is enabled for safe dependency maps, catalogs, overrides, and package-extension groups.

### Conan / C and C++ support

Scans `conanfile.txt` and `conanfile.py` files. `conanfile.txt` reads `[requires]`, `[tool_requires]`, and `[test_requires]` references. `conanfile.py` reads declarative `requires`, `tool_requires`, `build_requires`, `test_requires`, and `python_requires` string attributes. Updates preserve Conan version-range brackets plus user/channel and recipe revision suffixes such as `@user/channel` and `#revision`.

Conan registry dependencies query the Conan v2 search endpoint at `https://center2.conan.io/v2/conans/search?q={name}/*`. Set `versionlens.conan.apiUrl` for a different Conan-compatible v2 remote. OSV vulnerability lookups use Conan Package URL identity when an exact version is present. Sorting is disabled for Conan manifests.

### C/C++ build file support

Scans `CMakeLists.txt`, `*.cmake`, `xmake.lua`, `subprojects/*.wrap`, `WORKSPACE`, and `WORKSPACE.bazel` files. CMake files read GitHub-backed `FetchContent_Declare`, `ExternalProject_Add`, and `CPMAddPackage` declarations when a tag or version can be paired with a repository URL. Xmake files read `add_requires(...)` dependencies. Meson wrap files read `url` or `source_url`, plus `revision`, `wrapdb_version`, or a GitHub tag embedded in the URL. `source_fallback_url` is ignored for version selection. Bazel WORKSPACE files read `http_archive(...)` declarations with GitHub archive URLs and tag-like versions.

GitHub-backed C/C++ dependencies query repository tags through the GitHub tags API. Xmake dependencies without an explicit GitHub repository query package recipes from `https://raw.githubusercontent.com/xmake-io/xmake-repo/master`; set `versionlens.cpp.apiUrl` for a different raw Xmake package registry base. OSV vulnerability lookups use OSV.dev `GIT` queries for GitHub-backed C/C++ dependencies when an exact tag version is present. Sorting is disabled for C/C++ build files.

### vcpkg / C and C++ support

Scans `vcpkg.json` files. It reads top-level `dependencies`, feature dependencies under `features.*.dependencies`, and `overrides`. Version-constrained dependency objects update the `version>=` field, overrides update the `version` field, and baseline-only dependencies without an inline version are fixed baseline dependencies.

vcpkg registry dependencies query the configured registry repository version database at `https://raw.githubusercontent.com/microsoft/vcpkg/master/versions/{prefix}/{name}.json`. Set `versionlens.vcpkg.apiUrl` for a different raw vcpkg registry base. OSV vulnerability lookups use vcpkg Package URL identity when an exact version is present. Sorting is disabled for vcpkg manifests.

### Swift Package Manager support

Scans `Package.swift` files. It reads `.package(...)` entries in the package `dependencies` array, including `from:`, `.exact(...)`, `.upToNextMajor(from:)`, and `.upToNextMinor(from:)` version requirements. GitHub URL dependencies use GitHub tags for lookup. Branch, revision, non-GitHub Git URL, and local `path:` dependencies are fixed sources without registry update suggestions.

Swift registry dependencies query `/{scope}/{name}` on the configured Swift package registry base, `https://packages.swift.org`. Set `versionlens.swift.apiUrl` for a different Swift package registry base. OSV vulnerability lookups use the `SwiftURL` ecosystem and Swift Package URL identity. Sorting is disabled for Swift manifests.

### Zig support

Scans `build.zig.zon` files. It reads dependencies from the `.dependencies` struct, including `.url`, `.hash`, and local `.path` fields. GitHub archive URLs such as `/archive/refs/tags/{version}.tar.gz` expose the tag segment as the editable version and use GitHub tags for lookup. Hash-only, plain URL, and local path dependencies are fixed sources without registry update suggestions.

Non-GitHub Zig package identities use the configured `versionlens.zig.apiUrl` base, `https://pkg.ziglang.org`. OSV.dev does not cover Zig packages, so Zig dependencies skip vulnerability lookups. Sorting is disabled for Zig manifests.

### Nim / Nimble support

Scans `*.nimble` files. It reads `requires "..."` declarations at the top level, inside `dev:` blocks, and inside Nimble feature blocks. Version requirements preserve Nimble operators such as `==`, `>=`, `<=`, `^=`, and `~=` when edits are applied. Dependencies pinned to `#head` are fixed moving-branch dependencies without registry update suggestions.

GitHub URL dependencies use GitHub tags for lookup. Plain Nimble package names query the configured packages index at `https://raw.githubusercontent.com/nim-lang/packages/master/packages.json`. Set `versionlens.nim.apiUrl` for a compatible packages index or registry base. OSV.dev does not cover Nimble packages, so Nimble dependencies skip vulnerability lookups. Sorting is disabled for Nimble manifests.

### LuaRocks / Lua support

Scans `*.rockspec` files. It reads LuaRocks `dependencies`, `build_dependencies`, and `test_dependencies` arrays, preserving LuaRocks operators such as `==`, `~=`, `>=`, `<=`, and `~>` when edits are applied. The special `lua` runtime dependency is fixed because LuaRocks treats it as the Lua interpreter version rather than a rock package.

LuaRocks dependencies query the configured LuaRocks manifest at `https://luarocks.org/manifest`. Set `versionlens.luarocks.apiUrl` for a different LuaRocks server base or manifest URL. OSV.dev does not cover LuaRocks packages, so LuaRocks dependencies skip vulnerability lookups. Sorting is disabled for rockspec manifests.

### Python / PyPI / Pipenv / Poetry / uv support

Scans `Pipfile`, `pyproject.toml`, `*requirements*.txt`, and `*constraints*.txt` files. `Pipfile` reads `[packages]` and `[dev-packages]`. Requirements and constraints files read PEP 508-style requirement lines, preserve Python comparison operators such as `==`, `===`, `~=`, `>=`, `<=`, `!=`, `>`, and `<`, handle extras and environment markers, and use index options such as `--index-url` and `--extra-index-url` for registry discovery. Include and option lines that do not identify dependencies are ignored.

`pyproject.toml` reads project `version`, `[project]` dependencies, `[project.optional-dependencies]`, PEP 735 `[dependency-groups]`, Poetry `[tool.poetry.dependencies]`, legacy `[tool.poetry.dev-dependencies]`, Poetry group dependencies under `[tool.poetry.group.*.dependencies]`, and uv source declarations under `[tool.uv.sources]`. Poetry dependency tables with `version` and `source` fields preserve their editable version ranges and use named Poetry sources when available. Local `path`, direct Git, direct URL, uv source, and other unresolved source dependencies are fixed/source dependencies without destructive PyPI update suggestions.

Python dependencies query PyPI-compatible project metadata, with the default route at `https://pypi.org/rss/project/{name}/releases.xml` and support for `versionlens.pypi.apiUrl` templates such as `/pypi/{name}/json`. Poetry `[[tool.poetry.source]]`, uv index settings, Pipfile sources, and requirements index URLs contribute package index/source URLs. OSV vulnerability lookups use the `PyPI` ecosystem and Python Package URL identity. Sorting is enabled for Python dependency groups where declaration order is not semantic.

### Ruby support

Scans `Gemfile` and `*.gemspec` files. Gemfiles support RubyGems dependencies, Bundler `source` blocks and `source:` options, groups, GitHub/Git references, and local `path:` gems. Gemspec files support `add_dependency`, `add_runtime_dependency`, and `add_development_dependency` declarations; the first version requirement is updated while preserving Ruby requirement operators.

RubyGems dependencies query `https://rubygems.org/api/v1/versions/{name}.json`. Set `versionlens.ruby.apiUrl` for a different RubyGems-compatible API endpoint. Bundler `source` blocks and per-gem `source:` options override the default registry for the affected gems. GitHub dependencies use GitHub tags or commits when a tag, branch, or ref is present; local `path:` gems and non-GitHub Git URLs are fixed source dependencies without RubyGems update suggestions. OSV vulnerability lookups use the `RubyGems` ecosystem and RubyGems Package URL identity. Sorting is enabled for Ruby dependency declarations.

### Pub / Dart / Flutter support

Scans `pubspec.yaml`, `pubspec.yml`, and `pubspec_overrides.yaml`. It reads top-level package `version`, `workspace` members, `dependencies`, `dev_dependencies`, and `dependency_overrides`; `pubspec_overrides.yaml` is routed to dependency overrides only.

Hosted Pub dependencies use the package identity from `hosted.name` when present and query the hosted repository API under `/api/packages/{name}`. Set `versionlens.pub.apiUrl` for a different pub.dev-compatible API base. `path`, `sdk`, workspace, and Git dependencies are fixed/local sources without registry update suggestions. Git dependencies that use `tag_pattern` with `version` are treated as Git-sourced dependencies. OSV vulnerability lookups use the `Pub` ecosystem and Pub Package URL identity. Sorting is enabled for Pub dependency groups where declaration order is not semantic.

### Hex / BEAM support

Scans `mix.exs`, `rebar.config`, and `gleam.toml`.

- `mix.exs`: reads dependency tuples from `deps`, including Hex packages, local path dependencies, Git dependencies, aliases, environment-scoped dependencies, and umbrella dependencies.
- `rebar.config`: reads package tuples from top-level and profile-scoped `deps`, `plugins`, and `project_plugins`, including `{pkg, name}` aliases and Git/Hg source tuples.
- `gleam.toml`: reads root `name`/`version` project metadata and `[dependencies]`, `[dev_dependencies]`, and `[dev-dependencies]` entries, including version strings, local `path`, and Git source forms.

Hex registry dependencies query `https://hex.pm/api/packages/{name}`. Set `versionlens.hex.apiUrl` for a Hex-compatible package source. Rebar manifests also honor `HEX_CDN` and top-level `{rebar_packages_cdn, "..."}` mirror configuration. Local path dependencies use local directory status, and Git/Hg dependencies are fixed source dependencies without registry update suggestions. Sorting is disabled for Hex manifests.

### OCaml / opam support

Scans `opam`, `*.opam`, and `dune-project` package definition files. It reads opam package `version` metadata and package formulas in `depends`, `depopts`, and `conflicts`; it also reads Dune `(package ...)` `version` and `depends` metadata, preserving comparison operators when updating version constraints.

opam package lookups query `https://opam.ocaml.org/packages/{name}/`. Set `versionlens.opam.apiUrl` for an opam-compatible package site base URL. OSV vulnerability lookups use the `opam` ecosystem and opam Package URL identity. Sorting is disabled for opam manifests.

### Haskell / Cabal / Stack support

Scans `*.cabal`, `cabal.project`, `stack.yaml`, and `stack.yml`. Cabal package files read package `version` metadata and dependencies from `build-depends`, `setup-depends`, and `build-tool-depends`. `cabal.project` reads `constraints`. Stack project files read Stackage LTS and Nightly `resolver` snapshots plus Hackage package identifiers from `extra-deps`, and mark local path, Git, GitHub, compiler, and custom snapshot sources as fixed sources.

Hackage registry dependencies query `https://hackage.haskell.org/package/{name}.json`, while Stack resolvers query `https://www.stackage.org/api/v1/snapshots`. Set `versionlens.hackage.apiUrl` for a Hackage-compatible base URL. OSV vulnerability lookups use the `Hackage` ecosystem for Hackage packages; Stack resolver snapshots are not queried in OSV. Sorting is disabled for Haskell manifests.

### Julia support

Scans `Project.toml`, `Manifest.toml`, and `Manifest-v{major}.{minor}.toml`. Project files read package `version`, `[deps]`, `[compat]`, and `[sources]`; Manifest files read `[[deps.Name]]` entries with `version`, `path`, or `repo-url`.

Julia registry dependencies query `https://raw.githubusercontent.com/JuliaRegistries/General/master/{first-letter}/{name}/Versions.toml`. Set `versionlens.julia.apiUrl` for a different Julia registry base URL. Local `path` entries and Git `repo-url` entries are fixed source dependencies without registry update suggestions. OSV vulnerability lookups use the `Julia` ecosystem. Sorting is disabled for Julia manifests.

### R / CRAN support

Scans `DESCRIPTION` and `renv.lock` files. DESCRIPTION files read package `Version` metadata and package references from `Depends`, `Imports`, `Suggests`, `Enhances`, and `LinkingTo`. renv lockfiles read package records under `Packages`, including repository packages and non-repository sources.

CRAN registry dependencies query `https://cran.r-project.org/src/contrib/PACKAGES`. Set `versionlens.cran.apiUrl` for a different CRAN-like repository base URL. renv packages with local, URL, Git, or other non-repository sources are fixed source dependencies without registry update suggestions. OSV vulnerability lookups use the `CRAN` ecosystem. Sorting is disabled for R manifests.

### CocoaPods support

Scans CocoaPods `Podfile` manifests. It reads `pod` declarations in root, `target`, and `abstract_target` blocks, including version requirements, source-level `source` repositories, per-pod `:source` overrides, and unsupported `:path`, `:git`, and `:podspec` source forms. Pods without an explicit version use latest-version CocoaPods behavior; local path, Git, and podspec dependencies are fixed sources without registry update suggestions.

CocoaPods dependencies query the configured Specs repository CDN using `https://cdn.cocoapods.org`. Set `versionlens.cocoapods.apiUrl` for a different CocoaPods Specs-compatible base. Edits preserve Ruby-style CocoaPods operators such as `~>`, `>=`, `<=`, and `!=` by replacing only the editable version segment. OSV vulnerability lookups use CocoaPods Package URL identity when an exact version is present. Sorting is disabled for Podfiles because target grouping and declaration order can carry project meaning. Implementation follows the CocoaPods Podfile syntax guide and Specs repository format documented by CocoaPods.

## Show version information

Open a supported manifest and select the **V** icon in the editor toolbar.

You can also use the editor `...` menu item **Show release versions**, or set `versionlens.suggestions.showOnStartup` in VS Code settings.

## Show prerelease versions

Select the **tag** icon in the editor toolbar, or use **Show prerelease versions** from the editor `...` menu.

You can also set `versionlens.suggestions.showPrereleasesOnStartup`.

![Show prereleases](images/faq/show-prereleases.gif)

## Vulnerability checks

VersionLens integrates with OSV.dev to highlight vulnerable packages in manifest files.

- **Editor diagnostics:** vulnerable versions are marked in the editor.
- **Update safeguards:** updates to known vulnerable versions require confirmation.
- **Visual indicators:** updatable versions with known vulnerabilities are marked in the CodeLens text.

Set `versionlens.suggestions.showVulnerabilities` to enable or disable vulnerability checks.

## Custom install task

Set `versionlens.customInstallCommand` to run an install task when a package document is saved. Use `versionlens.showCustomInstall` to show the command in VS Code.

## Install

Download a VSIX from the [GitHub releases page](https://github.com/xsyetopz/vscode-versionlens/releases), then install it from VS Code:

1. Open the Extensions view.
2. Select `...`.
3. Select **Install from VSIX...**.
4. Choose the downloaded `.vsix` file.

## Troubleshooting

- **No CodeLens:** ensure `"editor.codeLens": true` is set.
- **Toolbar icons missing:** ensure `"workbench.editor.editorActionsLocation": "hidden"` is not set.
- **Stale results:** run **VersionLens: Clear cache** from the Command Palette.
- **Cache duration:** set `versionlens.cacheTtlSeconds`.
- **Logs:** set log level to `debug` through **Developer: Set Log Level** or the `VersionLens` output channel.

![Extension host log](images/faq/ext-host-log.jpg)

![Extension log](images/faq/ext-log.jpg)

If the logs do not show the cause, open **Help > Toggle Developer Tools**.

## Repository

Source, issues, releases, contributor instructions, and CI: <https://github.com/xsyetopz/vscode-versionlens>.

## License

[ISC](LICENSE)

