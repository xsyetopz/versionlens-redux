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
- **Diagnostics** warn about versions with known OSV.dev vulnerabilities.
- **Quick fixes** apply an update from the light bulb.
- **Commands**: Refresh Versions, Update All Dependencies in File, Toggle Version Lenses, Clear Cache, Restart Server, and Show Output.

A failed lookup shows `✗` with its reason on the dependency's line, and the output channel logs it.

## Supported Manifests

| Registry | Files |
| --- | --- |
| crates.io | `Cargo.toml`, `.cargo/config.toml`, `.cargo/config` (`[patch]` tables) |
| npm | `package.json`, `package.json5`, `pnpm-workspace.yaml`, `package.yaml`, `package.yml`, `.yarnrc.yml`, `.yarnrc.yaml`, `.nvmrc`, `.node-version`, `.bun-version` |
| JSR and npm | `deno.json`, `deno.jsonc`, `import_map.json`, `jsr.json`, `jsr.jsonc` |
| Unity | `Packages/manifest.json` |
| Packagist | `composer.json` |
| PyPI | `pyproject.toml`, `Pipfile`, `*requirements*.txt`, `*constraints*.txt`, `requirements/*.txt`, `requirements*.in` |
| Go proxy | `go.mod`, `go.work` |
| RubyGems | `Gemfile`, `*.gemspec` |
| CocoaPods | `Podfile` |
| pub.dev | `pubspec.yaml`, `pubspec_overrides.yaml` |
| NuGet | `*.csproj`, `*.fsproj`, `*.vbproj`, `*.props`, `*.targets`, `packages.config`, `dotnet-tools.json`, `paket.dependencies`, `project.json`, `global.json` |
| Maven Central | `pom.xml`, `build.gradle(.kts)`, `settings.gradle(.kts)`, `libs.versions.toml`, `build.sbt`, `deps.edn`, `.mvn/extensions.xml`, `*.main.kts` |
| Clojars | `project.clj` |
| Hex | `mix.exs`, `rebar.config`, `gleam.toml` |
| Julia General | `Project.toml`, `JuliaProject.toml`, `Manifest.toml`, `Manifest-v*.toml` |
| Haxelib | `haxelib.json`, `*.hxml` |
| Docker registries | `Dockerfile`, `Containerfile`, `*.dockerfile`, `compose.*.yaml`, `kustomization.yaml`, `Kustomization`, `devcontainer.json`, GitLab CI and CircleCI configs |
| GitHub tags | workflows (including nested folders), `.github/actions/**`, `action.yml`, `Package.swift`, `Cartfile`, `Cartfile.private`, `CMakeLists.txt`, `*.cmake`, `build.zig.zon`, `*.nimble`, `subprojects/*.wrap`, `WORKSPACE`, `WORKSPACE.bazel`, `.pre-commit-config.yaml`, `flake.nix` |
| Helm repositories | `Chart.yaml` |
| Ansible Galaxy | `requirements.yml`, `galaxy.yml` |
| Terraform Registry | `*.tf`, `*.tofu` |
| Bazel Central Registry | `MODULE.bazel` |
| vcpkg | `vcpkg.json` |
| ConanCenter | `conanfile.txt`, `conanfile.py` |
| CRAN | `DESCRIPTION`, `renv.lock` |
| MetaCPAN | `cpanfile` |
| DUB | `dub.json`, `dub.selections.json`, `dub.sdl` |
| opam | `opam`, `*.opam`, `dune-project` |
| Hackage | `*.cabal`, `cabal.project`, `stack.yaml` |
| Runtimes | `rust-toolchain`, `rust-toolchain.toml`, `.ruby-version`, `.python-version`, `.terraform-version`, `.tool-versions`, `mise.toml`, `gradle-wrapper.properties` |
| LuaRocks | `*.rockspec` |
| xmake repository | `xmake.lua` |

The server lists the files it reads in its initialize result, and the extension sends it only those, so this list cannot drift from the server.

## Settings

| Setting | Default | Effect |
| --- | --- | --- |
| `versionlens.enabled` | `true` | Show versions and vulnerabilities at all. |
| `versionlens.codeLens` | `true` | Show update actions as code lenses. |
| `versionlens.inlayHints` | `true` | Show the newest version as an inlay hint. |
| `versionlens.prereleases` | `false` | Offer prerelease versions as updates. |
| `versionlens.vulnerabilities` | `true` | Check versions against OSV.dev. |
| `versionlens.timeoutMs` | `10000` | Time limit for one registry request, in milliseconds. |
| `versionlens.concurrency` | `32` | Registry requests that run at once. |
| `versionlens.cacheMinutes` | `60` | How long registry answers are reused. |
| `versionlens.strictSsl` | `true` | Reject registries whose TLS certificate does not verify. |
| `versionlens.githubToken` | empty | Token for GitHub tag lookups. Falls back to `GITHUB_TOKEN`, then `GH_TOKEN`. |
| `versionlens.registries` | `{}` | URL template per registry id, for mirrors and private registries, such as `{"npm": "https://npm.example.com/{enc}"}`. |
| `versionlens.serverPath` | empty | A `versionlens-lsp` binary to run instead of the bundled one. `VERSIONLENS_LSP` takes precedence. |
| `versionlens.trace.server` | `off` | Log LSP traffic to the output channel. |

A changed setting applies without a restart, and the output channel logs it. Changing `versionlens.serverPath` restarts the server.

## Troubleshooting

**VersionLens: Show Output** opens the log: the server path, each applied setting, and every failed request with its reason.

| Symptom | Check |
| --- | --- |
| Nothing shows | The file name must match a row above, and `editor.codeLens` or `editor.inlayHints.enabled` must be on. |
| Requests time out | Raise `versionlens.timeoutMs`, or lower `versionlens.concurrency` on a slow network. |
| Private registry | Point `versionlens.registries` at the mirror. |
| Old results | Run **VersionLens: Clear Cache**. |

## Install From VSIX

Download a VSIX from the [GitHub releases page](https://github.com/xsyetopz/versionlens-redux/releases), then run **Extensions: Install from VSIX...**. `just package` builds one locally.

## License

[ISC](../../LICENSE). VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors, and reuses its visual assets with attribution.
