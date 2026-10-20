# Changelog

All notable changes to VersionLens Redux are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and releases follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - Unreleased

### Added

- Added update diagnostics with the code `update-available`, shown in the problems list of every editor, and a "Bump to x.y.z" quick fix for each. `versionlens.diagnostics.updateSeverity` sets their severity (`error`, `warn`, `info`, `hint` or `off`, default `info`) and does not affect vulnerability diagnostics.
- Added support for Carthage `Cartfile` and `Cartfile.private`, .NET `dotnet-tools.json` local tool manifests, and Cargo `[patch.*]` tables in `Cargo.toml` and `.cargo/config.toml`.
- Added Gradle `libs.versions.toml` `[versions]` entries, resolved through the libraries and plugins that reference them, so an entry behind latest offers an update and joins update-all.
- Added workflow `env` references (`${{ env.X }}`) to runtime resolution in GitHub Actions.
- Added credential sourcing for registry requests: a credential supplied by the editor, then `GITHUB_TOKEN` or `GH_TOKEN`, then `gh auth token` (tried once after a GitHub 401, 403 or rate limit), then the existing registry configuration files. VS Code supplies its credentials from SecretStorage and asks at most once per host and session; other editors fall back to the other sources. Secrets are redacted from logs and errors.
- Added rate-limit handling: the server stops requesting a rate-limited host until its reset time and shows one warning per reset with how to authenticate, instead of an error on every dependency.
- Added support for the standard proxy environment variables in registry requests, and a fall back to an in-memory cache with one warning when the cache directory is not writable.
- Added a Neovim server resolver (`cmd`, `$VERSIONLENS_LSP`, bundled, downloaded, `PATH`) with an `auto_download` option and a SHA-256-verified download, plus a `settings` option that answers `workspace/configuration`.
- Added a Zed download of the release archive for the extension's version, verified against `SHA256SUMS`, with older downloads removed.
- Added SHA-256 checksums (`SHA256SUMS`) for every release binary.

### Changed

- **Breaking:** Read settings only through `workspace/configuration`, once per workspace folder. The server ignores `initializationOptions` and `didChangeConfiguration` payloads, so an editor that pushed `versionlens` settings through either must now answer the `workspace/configuration` request. A client that does not answer it gets the defaults.
- **Breaking:** Moved the hard-coded `**/obj/**` exclusion into the default of the exclude setting. It now hides manifests of every ecosystem, not only .NET,.
- **Breaking:** Raised the VS Code engine floor from `>=1.75.0` to `^1.91.0`.
- **Breaking:** Removed the N-API native module. VS Code now talks to the same `versionlens-lsp` binary as the other editors. Commands and settings keys are unchanged.
- Made every editor an LSP client of the one `versionlens-lsp` server. VS Code ships one platform-specific VSIX per target with that binary, and the single JetBrains plugin bundles the prebuilt binary of every target and answers `workspace/configuration` from its settings.
- Built `versionlens-lsp` once per release for 9 targets in a reusable workflow, replacing 27 native builds, and pointed the editor workflows at it. Marketplace publishing no longer rebuilds the JetBrains plugin or the binaries.
- Resolved each workspace folder with its own settings.
- Made editing a manifest keep its resolved state and workspace discovery instead of discarding them, and ran heavy work off the server's main loop.
- Pushed results to the editor when a manifest is opened or edited, without waiting for a lens request.
- Offered a bounded set of bumps (patch, minor, major and latest) instead of one entry for every intermediate version.
- Treated an `=x.y.z` or `==x.y.z` pin and a Cargo bare version as the requirement they are, and counted an exact pin at latest as current.
- Reported a failed check as a log entry with retry backoff instead of a diagnostic.
- Made the VS Code client stop asking to re-enter credentials after a 401 or 403 and retry. A changed credential takes effect after the server restarts. A stored credential now applies to every request to the same hostname, whatever the path or port.
- Replaced `tsc` and the `typescript` dev dependency with `bun check` (Bun 1.4.3) and renamed the root `check` script to `verify`. Removed the custom architecture and code-quality audit scripts, `jscpd`, and unused scripts.
- Grew these `versionlens.*` defaults for the manifest support added in 0.5.0. A setting you have set yourself keeps its value and does not gain the new entries, so add them by hand to get the new files:
  - `versionlens.cargo.files` gained `**/.cargo/config.toml` and `**/.cargo/config`.
  - `versionlens.dotnet.files` gained `**/dotnet-tools.json`.
  - `versionlens.swift.files` gained `**/Cartfile` and `**/Cartfile.private`.
  - `versionlens.cargo.dependencyProperties` gained `patch.*` and `patch.*.*`.
- Generated the editor file lists (VS Code selectors and per-ecosystem settings, JetBrains, Neovim, Zed) from the manifest table with `bun scripts/generate/manifests.mjs`. Every 0.4.4 setting key and type is preserved. The Neovim and JetBrains file filters no longer accept `BUILD`, `BUILD.bazel`, and arbitrary `*.gradle`, `*.json5`, `*.kts`, `*.lock`, `*.pom` and `*.tfvars` files, which no parser reads.

### Removed

- Removed the `versionlens_napi.node` native module and the 27-artifact native build matrix.
- Stopped reading npm and Cargo project `version` fields as dependencies, so a package's own version no longer gets a lens.
- Removed the no-op update command from status lenses that apply no text change, such as an unresolved R `DESCRIPTION` dependency. The lens title still shows.

### Fixed

- Stopped offering a bump that would not change the requirement text, which removed the duplicate `latest v9 | ↑ latest v9` and `=0.7.0` suggestions.
- Restored the floor bump for ranges that already satisfy latest, such as `^2.5.8` to `^2.5.14`.
- Resolved a GitHub Actions reference to its most specific tag and treated a SHA pin with floating and exact tags as current. Branch and other non-version action refs are left as they are.
- Skipped inputs that cannot be resolved without an error: GitHub Actions `${{ }}` expressions such as step outputs, unsupported `setup-dotnet` syntax, and runtimes with no release source such as `engines.vscode`.
- Stopped publishing cancellations and internal failures as diagnostics, and named the cause of a remaining error.
- Made a click on a bump in VS Code either apply the edit or report why it could not.
- Kept an update lens working after it moved during an edit.
- Hid update diagnostics together with the lenses in VS Code.
- Reported a missing server binary once, and serialized server start, stop and restart in VS Code.
- Sent a credential with an empty scheme as the whole header value.

## [0.4.4] - 2026-09-15

### Fixed

- Made cold dependency checks substantially faster across providers with concurrent reusable HTTP connections, shared response bodies, request and resolution coalescing, a compressed CRAN catalog, shared LuaRocks catalog requests, abbreviated official npm metadata, and complete crates.io release histories for fixed pins, yanked releases, build choices, and edits.
- Restored VS Code dependency checks and updates by invoking asynchronous native commands with their session receiver.
- Declared receiver requirements for every native session method to prevent detached calls across the TypeScript boundary.

## [0.4.3] - 2026-09-12

### Added

- Added whole-workspace background checking, including unopened files, unsaved configuration overlays, provider exclusions, automatic expiry checks, and retries after temporary discovery failures.
- Added persistent caches isolated by source, canonical reference, requirement, policy, and authentication context, with bounded storage, atomic writes, corruption recovery, and coordinated invalidation.
- Added upstream version checks for Node.js, Bun, Rust toolchains, and package managers while preserving runtime constraint minimums during automatic checking.

### Changed

- Moved workspace scheduling and freshness into Rust, sharing bounded workers and workspace snapshots across checks.
- Required JetBrains IDE build 261.26222.65 or newer, corresponding to the 2026.1.4 minimum.

### Fixed

- Avoided suppressing runtime updates for compatible selectors while keeping exact current pins to one commandless lens.
- Accepted numeric one- through four-component Temurin Java releases while rejecting nonnumeric GA names.
- Parsed GitHub Actions structurally, including flow mappings, reusable workflows, Docker actions, commit pins, local references, and supported runtime inputs, without reading script bodies as dependencies.
- Preserved GitHub tag families and immutable pins, paginated tag lookups, and verified annotated tags through their commit targets.
- Made native update commands asynchronous and bounded; implemented LSP update commands, validated workspace edits, background refresh, cancellation, and clean shutdown.
- Kept vulnerability-service errors distinct from successful checks with no advisories.
- Corrected workspace URI handling, Zed x86-64 downloads, and operator and quote preservation in Nim, LuaRocks, CPAN, and Dune edits.
- Resolved Cargo workspace-inherited package versions from their root declaration and expanded pnpm workspace pattern matching, with explicit failures for unsupported negative patterns.
- Revalidated local action manifests after restart and handled Windows canonical drive and UNC paths in workspace URIs.

### Removed

- Removed downgrade choices, indicators, settings, presentation branches, and their dedicated tests and resources. Update commands reject older selected versions.
- Removed the package-only cache fallback, adapter completion bookkeeping, and disk-only registry context path after migrating their consumers.

## [0.4.1] - 2026-09-01

### Added

- Added approval-gated Marketplace publishing for VS Code, JetBrains IDEs, Zed, and Neovim through GitHub Actions, plus an interactive command that stores every required publishing value as a GitHub environment secret.
- Added LuaRocks packaging for the Neovim plugin and automated Zed registry pull-request creation for new and updated releases.

### Changed

- Kept all Rust crates and VS Code, Zed, Neovim, and JetBrains packages aligned on version 0.4.1.
- Made the Zed registry extension download the matching language-server archive from its exact GitHub release and licensed the Zed extension code under the registry-supported MIT license while leaving the rest of the repository under ISC.

### Fixed

- Made Marketplace secret setup resumable and preflighted, preserving existing secrets by default, validating all selected values before updates, and expanding home-relative signing file paths.
- Limited dependency downgrade lenses to the nearest older stable release so version history does not overwhelm each dependency line.
- Resolved both annotated-tag object SHAs and peeled commit SHAs for immutable GitHub Action pins, including Azure Login releases.

## [0.4.0] - 2026-09-01

### Added

- Added workspace-aware local package resolution for npm, Bun, pnpm, Yarn, and Cargo projects while keeping ambiguous or external packages on their existing registry paths.
- Added validated multi-document workspace edit plans that preserve open-document text and reject stale, duplicate, or overlapping edits before applying coordinated package-version changes.
- Added version lenses for additional versionable metadata, including package runtime engines, Cargo Rust toolchains, Kotlin Gradle plugins, and surfaced workspace or catalog references.
- Added actionable downgrade lenses for every known older stable release, with a configurable downward indicator and existing prerelease visibility rules.
- Added a production-ready Neovim plugin with automatic LSP startup, code-lens refresh, health checks, Plenary tests, platform-specific release archives, and stable/nightly CI coverage.

### Changed

- Made the shared Rust model the source of truth for dependency, project-version, runtime-constraint, workspace-reference, and ecosystem-handle classification across native and editor boundaries.
- Made open workspace documents authoritative over stale on-disk manifests during local package discovery and coordinated edits.
- Kept all Rust crates and VS Code, Zed, Neovim, and JetBrains packages aligned on version 0.4.0.
- Made JetBrains release archives Marketplace-selectable by OS and architecture, added plugin icons and verifier metadata, and hardened native language-server extraction.
- Resolved GitHub Actions tags through their canonical semantic version while preserving repository-specific prefixes such as `v`, `action-v`, and path-qualified tag namespaces.

### Fixed

- Removed redundant latest-version actions when a dependency or accepted range already resolves to the true latest release.
- Treated exact GitHub Action tags with `v` or `V` prefixes as current when they match the latest release, while preserving the prefix in displayed and applied version changes.
- Kept selected lower versions actionable through the existing safe edit path instead of rejecting valid downgrades.
- Tracked full and abbreviated GitHub Actions commit pins when an adjacent version comment identifies the exact repository tag, updating the commit SHA and comment together without replacing the immutable pin with a mutable tag.
- Rejected mismatched, ambiguous, malformed, bare-SHA, branch, and local Action references unless their identity and edit semantics are proven.
- Preserved established safe behavior for indirect references, including npm and Ruby Git refs, Maven properties, Gradle catalogs, workspace references, and immutable Docker digest pins.
- Completed the workspace and edit-plan boundary tests and architecture ownership required for the 0.4.0 release gates.

## [0.3.0] - 2026-08-28

### Added

- Added GitHub Actions workflow and reusable-workflow version detection through repository tag lookups.
- Added repository-owned capability and architecture audits for supported parsers, providers, native boundaries, and editor packages.

### Changed

- Split parser, provider, edit, suggestion, core-session, native, and editor responsibilities into explicit shared contracts and independently checked boundaries.
- Centralized release version management, packaging helpers, toolchain setup, fixtures, and editor package validation.

## [0.2.0] - 2026-07-19

### Added

- Added actionable latest, major, minor, patch, range-bump, prerelease, build, and vulnerability-aware upgrade choices across supported manifests.
- Added first-class parsing and provider coverage for additional native, JVM, scripting, infrastructure, and package-manager ecosystems.
- Added an explicit shared model crate and typed Rust, N-API, and VS Code host boundaries.

### Changed

- Reorganized the Rust workspace, provider pipeline, N-API bindings, extension adapter, and test ownership around enforceable module boundaries.
- Updated the Rust and Bun dependency sets and migrated the repository to the current Biome configuration without lint suppressions.
- Hardened release packaging, editor-package checks, dependency caching, registry resolution, and authentication handling.

### Fixed

- Restored upward-arrow upgrade lenses for outdated fixed versions, ranges, invalid requirements, and Python project dependencies.
- Preserved standard status and upgrade glyphs when configured indicators are missing, empty, or whitespace-only.
- Made every displayed upgrade choice apply the selected version while preserving valid Python, Ruby, SemVer, and manifest syntax.
- Preserved quoted, escaped, and nested manifest structures when sorting dependencies.
- Kept Rust and TypeScript authorization outputs synchronized and required across the native boundary.

## [0.1.2] - 2026-07-12

### Added

- Added a single command to build the VS Code, Zed, and JetBrains packages.
- Bundled and verified each editor package's required native runtime.
- Added a gated GitHub release workflow that tags successful master builds and attaches all three editor packages.
- Added target-specific VSIX packages for every native VS Code desktop target.

### Fixed

- Report incompatible VS Code native runtimes explicitly instead of failing activation without a VersionLens message.
- Package the correct native runtime for Windows, Linux, Alpine Linux, and macOS across x64, ARM64, and Linux ARMv7 targets.

## [0.1.1] - 2026-07-12

### Added

- Added a strict SemVer repository version-bump command covering Rust, Bun, VS Code, Zed, and JetBrains manifests and lockfiles.

### Fixed

- Parsed parenthesized PEP 508 requirements without including version syntax in Python package names.
- Escaped unsafe registry URL bytes and replaced every configured URL template placeholder.
- Made the VS Code version-lens toggle resolve the active document directly.
- Updated LSP response construction for lsp-server 0.9.

## [0.1.0] - 2026-07-11

### Added

- Introduced VersionLens Redux as the versionlens-redux VS Code extension under the xsyetopz publisher.
- Added conflict detection for the original pflannery.vscode-versionlens extension.
- Added Rust-backed dependency analysis across the supported manifest ecosystems, including C/C++ and JVM build files.
- Preserved attribution to the original VersionLens authors.

[0.4.4]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.3...v0.4.4
[0.4.3]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.2...v0.4.3
[0.4.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/xsyetopz/versionlens-redux/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/xsyetopz/versionlens-redux/releases/tag/v0.1.0
