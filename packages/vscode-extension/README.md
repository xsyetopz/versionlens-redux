# VersionLens Redux for Visual Studio Code

VersionLens Redux shows dependency versions, update actions, and vulnerability diagnostics in VS Code manifest files.

[![VS Code CI/CD](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml/badge.svg)](https://github.com/xsyetopz/versionlens-redux/actions/workflows/vscode.yml)
[![GitHub release](https://img.shields.io/github/v/release/xsyetopz/versionlens-redux?include_prereleases)](https://github.com/xsyetopz/versionlens-redux/releases)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](../../LICENSE)

![Show releases](images/faq/show-releases.gif)

## Extension Identity

- VS Code id: `xsyetopz.versionlens-redux`
- Forked from `pflannery.vscode-versionlens`. Disable the original while this one runs, or both show versions.

## Features

- **Inlay hints** after each dependency show the newest version, or `✓` when it is current.
- **Code lenses** above each dependency offer the updates that apply: the newest version the range allows, the latest release, and a prerelease when `versionlens.prereleases` is on.
- **Hovers** show the requirement, the resolved and latest versions, a link to the registry page, and any OSV advisories.
- **Diagnostics** warn about versions with known OSV.dev vulnerabilities, and mark each outdated dependency at `versionlens.updateSeverity`.
- **Quick fixes** apply an update from the light bulb; **source actions** update, sort, and switch the whole file.
- **Editor title icons** on each manifest: show or hide release versions, show or hide prerelease versions, a spinner while lookups run, and an error icon that opens the log when a lookup fails.
- **Editor title menu (`...`)**: show or hide versions and prereleases, run the custom install task, sort dependencies alphabetically, and update dependencies to latest, major-only, minor-only, or patch-only.
- **Commands** in the palette: everything in the menu, plus refresh versions, clear cache, update dependencies in the workspace at each level, add and remove url authentication, restart the language server, and show its output.

A failed lookup shows `✗` with its reason on the dependency's line, and the output channel logs it.

## Supported Manifests

| Registry | OSV advisories | Files |
| --- | --- | --- |
| crates.io | crates.io | `Cargo.toml`, `.cargo/config.toml`, `.cargo/config` (`[patch]` tables), `Cargo.lock` |
| npm | npm | `package.json` (including `jspm` `npm:` values), `package.json5`, `pnpm-workspace.yaml`, `package.yaml`, `package.yml`, `.yarnrc.yml`, `.yarnrc.yaml`, `bower.json`, `rescript.json` and `bsconfig.json` (names only; the version lives in `package.json`), `.bun-version`, `npm:` entries of `.prototools`, and the resolved versions in `package-lock.json`, `npm-shrinkwrap.json`, `yarn.lock`, `pnpm-lock.yaml`, and `bun.lock` |
| Node.js releases | Bitnami (`node`) | `.nvmrc`, `.node-version`, `node` in `.tool-versions`, `mise.toml`, and `.prototools`, `node-version` of `actions/setup-node` |
| JSR and npm | npm; GIT for JSR packages (the GitHub repository jsr.io links) | `deno.json`, `deno.jsonc`, `import_map.json`, `jsr.json`, `jsr.jsonc`, and the published package's own `name` |
| Unity | | `Packages/manifest.json` |
| Packagist | Packagist | `composer.json`, `composer.lock` |
| PEAR and PECL channels | | `package.xml`, `package2.xml` (PEAR 2.0 `package` and `extension` dependencies) |
| Homebrew formulae and casks | | `Brewfile`, `.Brewfile`, `Brewfile.*` `brew` and `cask` entries (names only; Homebrew pins no versions, so each lens shows the current stable release) |
| Hunter | | `hunter_config(name VERSION v)` in `*.cmake` |
| PlatformIO Registry | GIT (libraries, by their repository) | `platformio.ini` `platform`, `platform_packages`, and `lib_deps` |
| ESP Component Registry | | `idf_component.yml` (`namespace/name`, `espressif` by default) |
| Typst Universe | | `#import "@preview/name:version"` in `*.typ` |
| Wally | | `wally.toml` `"scope/name@version"` values |
| mooncakes.io | GIT (the GitHub repository the module's `repository` names) | `moon.mod.json` `deps` |
| PureScript Registry | GIT (the package's GitHub location) | `spago.yaml` dependencies with a range |
| VPM, the tags of the GitHub repository it names | GIT (that repository) | `v.mod` `dependencies` |
| nim-lang/packages, the tags of the GitHub repository it names | GIT (that repository) | `*.nimble` `requires` package names |
| Alire index | | `alire.toml` `[[depends-on]]` |
| Idris pack collections | | `pack.toml` `collection` |
| raku.land | | `META6.json` `depends`, `build-depends`, and `test-depends` |
| Arduino libraries, through the PlatformIO Registry | | `library.properties` `depends` |
| PyPI | PyPI | `pyproject.toml`, `Pipfile`, `*requirements*.txt`, Pants `BUILD` `python_requirement`s, `*constraints*.txt`, `requirements/*.txt`, `requirements*.in`, `uv.toml`, `setup.cfg`, `setup.py`, `tox.ini`, the `pip:` list of `environment.yml`, `poetry.lock`, `uv.lock` |
| conda-forge | GIT (the GitHub repository its `dev_url` names) | `environment.yml`, `environment.yaml`, `pixi.toml` |
| Go proxy | Go | `go.mod`, `go.work`, `Gopkg.toml`, `glide.yaml` |
| Go releases | Go (`stdlib`) | the `go` and `toolchain` lines of `go.mod` and `go.work`, `.go-version`, `go` in `.tool-versions`, `mise.toml`, and `.prototools`, `go-version` of `actions/setup-go` |
| RubyGems | RubyGems | `Gemfile`, `gems.rb`, `*.gemspec`, `Gemfile.lock`, `gems.locked` |
| CocoaPods | GIT (the GitHub repository the podspec's `source` names) | `Podfile`, `*.podspec` |
| pub.dev | Pub | `pubspec.yaml`, `pubspec_overrides.yaml` |
| NuGet | NuGet | MSBuild SDKs and packages in `*.csproj`, `*.fsproj`, `*.vbproj`, `*.props`, `*.targets`, `packages.config`, `dotnet-tools.json`, `paket.dependencies`, `paket.references`, `project.json`, `global.json`, `*.nuspec` |
| Maven Central | Maven | `pom.xml`, `build.gradle(.kts)`, `settings.gradle(.kts)`, `libs.versions.toml`, `*.sbt`, Mill `*.mill` and `build.sc` (`mvn"…"` and `ivy"…"`), `ivy.xml` (with `override`s), Maven 1 `project.xml`, Buildr `buildfile`, `.mvn/extensions.xml`, `*.main.kts`, Pants `BUILD` `jvm_artifact`s |
| Clojars, then Maven Central | Maven | `project.clj`, `deps.edn` |
| Hex | Hex | `mix.exs`, `rebar.config`, `gleam.toml` |
| Julia General | Julia | `Project.toml`, `JuliaProject.toml`, `Manifest.toml`, `Manifest-v*.toml` |
| Julia General, then Julia's release list | Julia | `Manifest.toml` standard libraries |
| Haxelib | GIT (the GitHub repository the library page links) | `haxelib.json`, `*.hxml` |
| Elm packages | GIT (`github.com/author/package`) | `elm.json` |
| Docker Hub and any OCI registry (MCR, ghcr.io, quay.io, gcr.io, registry.k8s.io, public.ecr.aws, GitLab, private hosts) | Bitnami, by the application an official, `bitnami/`, `hashicorp/`, or self-named image packages (`nginx`, `postgres`, `grafana/grafana`) | `Dockerfile`, `Containerfile`, `*.Dockerfile`, `Earthfile`, `compose.*.yaml`, `kustomization.yaml`, `Kustomization`, Kubernetes manifests under `k8s/`, `kubernetes/`, `manifests/`, and `deploy/`, `*.k8s.yaml`, `skaffold.yaml`, `tilt.yaml`, Helm `values.yaml` (`image:` and `repository`/`tag`), `devcontainer.json`, workflow `container:`, `services:`, and `docker://` images, GitLab CI, CircleCI, Bitbucket Pipelines, Drone, Woodpecker, Cirrus, Azure Pipelines, Cloud Build, and Gitpod configs, and `oci://` chart dependencies in `Chart.yaml` |
| GitHub tags | GIT (`github.com/owner/repo` at the tag) | workflows (including nested folders), `.github/actions/**`, `action.yml`, `jspm` `github:` values, `Cartfile`, `Cartfile.private`, `Mintfile`, `shard.yml`, `CMakeLists.txt` and `*.cmake` (CPM.cmake `gh:` shorthands and named calls, FetchContent `GIT_REPOSITORY` and `URL`), `build.zig.zon`, `*.nimble`, `subprojects/*.wrap`, `WORKSPACE`, `WORKSPACE.bazel`, `.pre-commit-config.yaml`, `flake.nix`, Terraform and Terragrunt `git::` module sources with `?ref=`, Flux `GitRepository` tags, `snapcraft.yaml` `source-tag`, Flatpak manifest git `tag`s, `aqua.yaml`, `component.json`, `fpm.toml` git `tag`s, `boss.json`, `corral.json`, `qlfile` `:tag`s, `lakefile.toml` and `lakefile.lean` git `rev`s, `lake-manifest.json` `inputRev`s, `flake.lock` GitHub inputs, Buck2 `BUCK` and `TARGETS` archives and releases, `west.yml` projects with a GitHub `url`, Scoop `bucket/*.json` and winget `*.installer.yaml` release URLs, and `Pulumi.yaml` packages |
| Swift packages on GitHub | SwiftURL (`github.com/owner/repo`) | `Package.swift` |
| Helm repositories | Artifact Hub image scans | `Chart.yaml`, Argo CD `Application` charts (`argocd/`, `argo-cd/`, `clusters/`, and Kubernetes manifest folders) |
| Artifact Hub | Artifact Hub image scans | `helmfile.yaml`, `helmfile.yaml.gotmpl`, `helmfile.d/*.yaml` (`repo/chart` releases), Flux `HelmRelease` charts |
| Ansible Galaxy | GIT (the GitHub repository the collection version's `repository` names) | `requirements.yml`, `galaxy.yml` |
| Ansible Galaxy roles | GIT (the role's `github_user`/`github_repo`) | `roles:` in `requirements.yml` |
| Puppet Forge | GIT (the GitHub repository the module's `source` names) | `Puppetfile` |
| Chef Supermarket | GIT (the GitHub repository the cookbook's `source_url` names) | `Berksfile`, `metadata.rb` |
| Terraform Registry | GIT (the GitHub repository the provider or module's `source` names) | `*.tf`, `*.tofu`, `.terraform.lock.hcl`, `terragrunt.hcl`, `terragrunt.stack.hcl`, `Pulumi.yaml` `terraform-provider` and module parameters |
| Packer plugins on GitHub | Go (`github.com/owner/packer-plugin-name`) | `required_plugins` in `*.pkr.hcl` |
| Vagrant Cloud | | `config.vm.box` and `box_version` in `Vagrantfile` |
| Nixhub | | `devbox.json` |
| Spack packages | | `spack.yaml` environment specs |
| Bazel Central Registry | GIT (the GitHub repository the module's `github:` repository names) | `MODULE.bazel` |
| vcpkg | GIT (the GitHub repository the port's `vcpkg_from_github` names) | `vcpkg.json` |
| ConanCenter | GitLab Advisory Database | `conanfile.txt`, `conanfile.py` |
| CRAN | CRAN | `DESCRIPTION`, `renv.lock` |
| MetaCPAN | GIT (the GitHub repository the release's `repository` names) | `cpanfile`, `META.json`, `META.yml`, `MYMETA.json`, `MYMETA.yml`, `dist.ini` |
| DUB | GIT (the GitHub repository the package names) | `dub.json`, `dub.selections.json`, `dub.sdl` |
| opam | opam | `opam`, `*.opam`, `dune-project` |
| Hackage | Hackage | `*.cabal`, `cabal.project`, `stack.yaml` |
| GHC releases | GHC | `with-compiler: ghc-…` in `cabal.project`, and `resolver:` or `snapshot: ghc-…` in `stack.yaml` |
| LuaRocks | GIT (the GitHub repository the rockspec's `source.url` names) | `*.rockspec` |
| xmake repository | GIT (the GitHub repository the package's `xmake.lua` names) | `xmake.lua` |
| Typst releases | GIT (`typst/typst`) | `compiler` in `typst.toml` |
| Stackage snapshots | | `resolver:` and `snapshot:` `lts-…` and `nightly-…` in `stack.yaml`; a snapshot by URL or `github:` path is labelled as its source |
| Julia releases | | `julia` in a `Project.toml` `[compat]` |
| R releases (R-hub) | | `R (>= …)` in a `DESCRIPTION` |
| Racket releases | | the `"base"` `#:version` lower bound in an `info.rkt` |
| Deno releases | crates.io (`deno`) | the `deno-version` input of `setup-deno` |
| .NET SDK releases, the newest of each channel | Bitnami (`dotnet-sdk`) | the `dotnet-version` input of `setup-dotnet` |
| Temurin Java feature releases | Bitnami (`java`) | the `java-version` input of `setup-java` |
| Python, Ruby, and Gradle releases | Bitnami (`python`, `ruby`, `gradle`) | `.python-version`, `.ruby-version`, `gradle-wrapper.properties`, their `.tool-versions` and `mise.toml` entries, and the `python-version` and `ruby-version` inputs of workflow setup actions |
| Rust and Terraform releases | GIT (`rust-lang/rust`, `hashicorp/terraform`) | `rust-toolchain`, `rust-toolchain.toml`, `.terraform-version`, `rust-version` in `Cargo.toml`, and their `.tool-versions` and `mise.toml` entries |

The server lists the files it reads in its initialize result, and the extension sends it only those, so this list cannot drift from the server. OSV.dev has no ecosystem for Helm or Conan, so Helm chart versions report the CVEs that Artifact Hub's Trivy scans find in the images they deploy, and Conan versions report the GitLab Advisory Database entries whose range holds them. Registries outside OSV that name a package's GitHub repository have it looked up, and OSV's GIT ecosystem is asked about that repository at the version; a package that names no GitHub repository gets no check. Docker Hub and OCI images are asked about by the application they package in OSV's Bitnami ecosystem, so an image from any other publisher gets no check. Unity packages name only Unity's own git host, so they get no vulnerability check. Set `VERSIONLENS_NPM_USER_AGENT` and `VERSIONLENS_NPM_COOKIE` to the user agent and `cf_clearance` cookie of a browser that passed the npmjs.com challenge, and the live check sends them to npmjs.com. `just registries-live` asks every registry, package page, and OSV ecosystem for a sample package, and fails when one stops answering.

Racket's `info.rkt` stays unread because no version comparison has data to work with: its pkgs.racket-lang.org record keys `versions` by Racket release, so it carries no package release history. A bare name, as in a `Brewfile`, `rescript.json`, or npm `bundleDependencies`, shows its latest version and offers no edit, as the file has no place for one.

## Settings

| Setting | Default | Effect |
| --- | --- | --- |
| `versionlens.enabled` | `true` | Show versions and vulnerabilities at all. |
| `versionlens.codeLens` | `true` | Show update actions as code lenses. |
| `versionlens.inlayHints` | `true` | Show the newest version as an inlay hint. |
| `versionlens.prereleases` | `false` | Offer prerelease versions as updates. |
| `versionlens.vulnerabilities` | `true` | Check versions against OSV.dev, Artifact Hub, and the GitLab Advisory Database. |
| `versionlens.timeoutMs` | `10000` | Time limit for one registry request, in milliseconds. |
| `versionlens.concurrency` | `32` | Registry requests that run at once. |
| `versionlens.cacheMinutes` | `60` | How long registry answers are reused. |
| `versionlens.strictSsl` | `true` | Reject registries whose TLS certificate does not verify. |
| `versionlens.githubToken` | empty | Token for GitHub tag lookups. Falls back to `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token`. |
| `versionlens.registries` | `{}` | URL template per registry id, for mirrors and private registries, such as `{"npm": "https://npm.example.com/{enc}"}`. |
| `versionlens.providers` | `{}` | Per registry id, overrides of `cacheMinutes`, `strictSsl`, and `prereleaseTags`, such as `{"npm": {"cacheMinutes": 5}}`. |
| `versionlens.authorization` | `{}` | `Authorization` header per registry URL prefix. **Add url authentication** keeps headers in secret storage instead. |
| `versionlens.install` | `{}` | Shell command per file type or registry id, run in the manifest's folder when a save changes the dependencies and from **Run custom install task**. |
| `versionlens.files` | `{}` | Extra manifests: glob to file type id, such as `{"deps/*.list": "requirements"}`. |
| `versionlens.indicators` | glyphs | Glyph per state in lenses and hints: `latest`, `update`, `checking`, `failed`, `unreadable`, `vulnerable`, `newer`, `satisfies`, `noMatch`, `directory`, `build`, `yanked`, `deprecated`. |
| `versionlens.prereleaseTags` | `[]` | Offer only prereleases with one of these tags; empty offers all. |
| `versionlens.updateSeverity` | `info` | Diagnostic severity for an outdated dependency: `off`, `hint`, `info`, `warning`, or `error`. |
| `versionlens.stats` | `false` | A summary lens above each manifest. |
| `versionlens.fileTypes` | `[]` | Read only these file type or registry ids; empty reads all. |
| `versionlens.workspaceExclude` | `[]` | Globs the workspace update skips, such as `["examples/**"]`. |
| `versionlens.serverPath` | empty | A `versionlens-lsp` binary to run instead of the bundled one. `VERSIONLENS_LSP` takes precedence. |
| `versionlens.trace.server` | `off` | Log LSP traffic to the output channel. |

### Project Registry Configuration

The server reads the registry configuration a package manager would, from the manifest's folder up and then the user's home, and asks each dependency's registries in the order that package manager tries them, with their credentials:

| Registry | Read from |
| --- | --- |
| npm | the nearest `.npmrc` and the `.env` beside it, then `NPM_CONFIG_USERCONFIG` or `~/.npmrc`, then `npm_config_registry` and `npm_config_@scope:registry`: `registry`, `@scope:registry`, and `//host/path/:_authToken`, `:_auth`, or `:username` with `:_password`, with `${VAR}` expanded |
| NuGet | `nuget.config` in each parent folder and the user's `NuGet.Config`: `packageSources` with `<clear/>`, `disabledPackageSources`, `packageSourceCredentials`, `packageSourceMapping`, and `%VAR%`; without any, `dotnet nuget list source`. v3 feeds only. |
| Composer | `repositories` of type `composer` with `only` and `exclude`, `"packagist.org": false`, and credentials from `COMPOSER_AUTH`, `auth.json`, and `COMPOSER_HOME/auth.json` (`http-basic`, `bearer`) |
| PyPI | `[[tool.uv.index]]`, `[[tool.poetry.source]]`, Pipfile `[[source]]`, requirements `--index-url` and `--extra-index-url`, `UV_DEFAULT_INDEX`, `UV_INDEX`, `PIP_INDEX_URL`, and `pip.conf`; a dependency's `source`, `index`, or `tool.uv.sources` index picks one. Credentials come from the URL, `UV_INDEX_<NAME>_USERNAME` and `_PASSWORD`, or `POETRY_HTTP_BASIC_<NAME>_*`. The index must answer the PEP 691 JSON API. |
| Cargo | `.cargo/config.toml` in each parent folder and `CARGO_HOME`: `registries.<name>.index`, `source.<name>.replace-with`, and tokens from `CARGO_REGISTRIES_<NAME>_TOKEN`, the configuration, and `credentials.toml`, sent only when `cargo:token` is the registry's `credential-provider` or one of `registry.global-credential-providers`, as Cargo requires. Sparse indexes only. |
| Maven | `~/.m2/settings.xml` and `MAVEN_HOME/conf/settings.xml`: the mirror of `central` and the `<server>` with its id, with `${env.VAR}` expanded. Encrypted passwords are not decrypted. |

`versionlens.registries` overrides all of this for its registry id.

### Settings and Commands From VersionLens 1.x

A `settings.json` written for the original extension keeps working; a key of this version wins over the original one it replaces.

| Original | Read as |
| --- | --- |
| `versionlens.caching.duration`, `versionlens.http.strictSSL` | `cacheMinutes`, `strictSsl` |
| `versionlens.suggestions.showOnStartup`, `.showPrereleasesOnStartup`, `.showVulnerabilities`, `.showSuggestionsStats` | `enabled`, `prereleases`, `vulnerabilities`, `stats` |
| `versionlens.suggestions.indicators` `Latest`, `Updateable`, `Error`, `UpdateableVulnerable` | `indicators` `latest`, `update`, `failed`, `vulnerable` |
| `versionlens.enabledProviders` | `fileTypes`, with `golang` as `go`, `ruby` as `rubygems`, `dotnet` as `nuget`, `pnpm` as `npm`, `deno` as `jsr` |
| `versionlens.<provider>.caching.duration`, `.http.strictSSL`, `.prereleaseTagFilter` | `providers.<registry id>` |
| `versionlens.<provider>.onSaveChanges` | `install` for the provider's file type |
| `versionlens.<provider>.files` | `files`, one entry per `{a,b}` alternative |

The output channel names any `versionlens.<provider>.apiUrl`, `.dependencyProperties`, or `versionlens.dotnet.nuget.sources` it finds: registry URLs are now templates in `versionlens.registries`, and the catalog holds the dependency rules. Every original command id, such as `versionlens.editor.onShowVersionLenses`, still runs its replacement, so old keybindings work.

A changed setting applies without a restart, and the output channel logs it. Changing `versionlens.serverPath` or `versionlens.files` restarts the server.

## Troubleshooting

**VersionLens: Show Output** opens the log: the server path, each applied setting, and every failed request with its reason.

| Symptom | Check |
| --- | --- |
| Nothing shows | The file name must match a row above, and `editor.codeLens` or `editor.inlayHints.enabled` must be on. |
| Requests time out | Raise `versionlens.timeoutMs`, or lower `versionlens.concurrency` on a slow network. |
| Private registry | Check the project configuration under Project Registry Configuration, or point `versionlens.registries` at the mirror. |
| Old results | Run **VersionLens: Clear Cache**. |

## Install From VSIX

Download a VSIX from the [GitHub releases page](https://github.com/xsyetopz/versionlens-redux/releases), then run **Extensions: Install from VSIX...**. `just package` builds one locally.

## License

[ISC](../../LICENSE). VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors, and reuses its visual assets with attribution.
