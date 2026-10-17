# Tasks

Execution model: no more than 4 subagents run at once. Groups 1 and 2 are done serially by the lead before the lanes start. Groups 3–6 are four parallel lanes, one subagent each, and each lane owns a disjoint set of paths (listed under its heading). Groups 7–9 are serial integration by the lead after all lanes merge. Merge order into the integration branch: lane 3, then 4, then 5, then 6.

## 1. Foundation: tooling (lead, serial)

- [x] 1.1 Rename the root `check` script to `verify` in `package.json`, and update `justfile`, `lefthook.yml`, `.github/workflows/*`, `AGENTS.md` and `CONTRIBUTING.md`. Verify: `bun check --help` prints Bun's builtin help, and `grep -rn "bun run check\b" .github justfile lefthook.yml AGENTS.md CONTRIBUTING.md` finds nothing.
- [x] 1.2 Make `typecheck` run `bun check --cwd packages/vscode-extension`. Remove the `typescript` dev dependency, the root `tsconfig.json` and `tests/tsconfig.json`, `composite` in `tsconfig.base.json`, and `outDir`/`tsBuildInfoFile` in the extension tsconfig. Regenerate `bun.lock` and set `packageManager`/`engines.bun` to Bun ≥1.4.3. Verify: `bun install --frozen-lockfile && bun run typecheck` reports no type errors, and `grep -n '"typescript"' package.json bun.lock` finds nothing.
- [x] 1.3 Pin the Bun version in `release.yml` and `.github/actions/setup-bun` from `packageManager`. Verify: `actionlint` passes and the workflows contain no `bun-version: latest`.
- [x] 1.4 Delete the in-house lint scripts (`scripts/architecture_audit`, `scripts/audit_architecture.py`, `scripts/providers*`, `scripts/check/*`) and jscpd, with their `package.json` scripts, lefthook hooks, CI steps keeping a minimal `scripts/package/vsix.mjs` that checks each per-platform VSIX's native binary digest and target. Verify: `bun run typecheck` and `actionlint` pass, and a grep for the removed script names finds nothing outside OpenSpec history.

## 2. Foundation: contracts and baselines (lead, serial)

- [x] 2.1 Capture golden output from 0.4.4 napi (lens titles and ranges, diagnostics, edits per action) for every fixture manifest into `tests/golden/0.4.4/`. Verify: the capture script runs twice and the second run leaves `git diff --exit-code tests/golden` clean.
- [x] 2.2 Add regression fixtures for each reported bug: `=0.7.0` Cargo pin, `requests==2.34.2`, `^2.5.8`/2.5.14, the starlight intermediate list, `actions/github-script` SHA with `v9`/`v9.0.0`, `taiki-e/install-action@just`, `engines.vscode`, `${{ env.X }}` and `${{ steps.* }}`, the Gradle `[versions]` alias, Cargo bare `"1.0.200"`, and package `version`. Verify: `cargo test -p versionlens-core fixtures` lists them, and they fail on the current tree (expected red).
- [x] 2.3 Define the editor-neutral engine types `LensView`, `Action` (typed command enum with named fields) and `Severity`, and the `Plan { status, bumps }` signature, in the engine. Remove `versionlens-vscode-model` from the engine's public API, so only napi and lsp map them. Verify: `cargo check --workspace` passes, and `cargo tree -p versionlens-core -i versionlens-vscode-model` finds no dependency.
- [x] 2.4 Add `scripts/package/targets.json` with the 9 targets and their Rust triples, and freeze the `versionlens/status`, `versionlens/authorization` and command argument JSON shapes in `crates/versionlens-lsp/src/protocol.rs` with serde round-trip tests. Verify: `cargo test -p versionlens-lsp protocol` passes.

## 3. Lane A: suggestions, model and fetch (subagent 1)

Owns: `crates/versionlens-versions`, `crates/versionlens-model`, `crates/versionlens-suggestions`, `crates/versionlens-edits`, `crates/versionlens-cache`, `crates/versionlens-http`, `crates/versionlens-core/src/{fetch,registry,session/resolution}`, `crates/versionlens-providers`.

- [x] 3.1 Merge `versionlens-versions` into `versionlens-model`, and deduplicate the five `parse_semver` copies into one per dialect. Verify: `cargo test -p versionlens-model` passes and `grep -rn "fn parse_semver" crates | wc -l` prints 1.
- [ ] 3.2 Merge `versionlens-suggestions` and `versionlens-edits` into `versionlens-update`. Implement `update::plan` per design D3:
  - one dialect-aware currency rule
  - floor bump
  - bounded set
  - deduplication by text
  - drop unchanged text
  - parsed-version comparison
  - Cargo bare = caret

  Delete `push_intermediate_range_choices`, `range_minimum_matches_latest` and the `retain` in `session/resolution/dependency.rs`. Verify: the group 2.2 fixtures for suggestions pass with `cargo test -p versionlens-update -p versionlens-core`.
- [ ] 3.3 Fix GitHub Action refs: compare commit SHAs, pick the most specific tag on a commit, and skip lookups for non-version refs. Verify: the `github-script` and `install-action@just` fixtures pass, and a mock-HTTP test asserts zero requests for `@just`.
- [ ] 3.4 Merge `versionlens-cache`, `versionlens-http` and the core fetch I/O into `versionlens-fetch`:
  - honor `HTTPS_PROXY`/`HTTP_PROXY`/`ALL_PROXY`/`NO_PROXY`
  - add the platform trust store via `rustls-native-certs` (confirm it is already in `Cargo.lock`, otherwise record it as the one new dependency)
  - keep `versionlens.http.strictSSL`
  - keep rate-limit headers
  - add a per-host circuit until the reset time
  - use an in-memory cache fallback when the cache dir is unwritable
  - use a versioned JSON persistent cache without HMAC

  Verify: `cargo test -p versionlens-fetch` covers the proxy env, a read-only cache dir and 403 with `x-ratelimit-reset`.
- [ ] 3.5 Add credential sources in order: client-supplied, then `GITHUB_TOKEN`, then `GH_TOKEN`, then `gh auth token` (once per session, 2 s timeout, only after 401/403), then the existing config files. Add redaction tests. Verify: `cargo test -p versionlens-fetch credentials`, and a test asserts no token appears in `Debug`/log output.
- [ ] 3.6 Return `NotSupported` (skipped, no error) for runtimes with no release source, including `engines.vscode` and unsupported setup-dotnet syntax. Include the cause (status, host, error kind) in resolution errors. Verify: the `engines.vscode` fixture produces no error, and the 404 test renders `not found (404 registry.npmjs.org)`.

## 4. Lane B: language server runtime and protocol (subagent 2)

Owns: `crates/versionlens-lsp`, `crates/versionlens-core/src/{session,presentation,config}` except `session/resolution`.

- [x] 4.1 Replace `server/transport.rs` and the 200 ms tick with `lsp_server::Connection::stdio()`. Use one multi-thread tokio runtime for resolution and I/O, `panic = "unwind"` with per-task catch, and keep no blocking I/O on the protocol thread. Verify: `cargo test -p versionlens-lsp`, and an integration test with a 5 s delayed mock registry gets `codeLens` for another document and `shutdown` answered in under 500 ms.
- [ ] 4.2 Add incremental document state: diff dependencies by (name, requirement, source), keep the registry cache and discovery across edits, and restart discovery only on watched manifest or lockfile changes or folder changes. Verify: an integration test where a comment edit in `package.json` makes zero registry requests, and a single-requirement edit re-resolves only that dependency.
- [ ] 4.3 Add push-driven resolution (300 ms debounce), `workspace/codeLens/refresh` when supported, and `versionlens/status` idle/busy/error notifications. Verify: an integration test with a client lacking codeLens capability receives `publishDiagnostics`, and the status sequence is busy then idle. Also move the analysis on `didOpen` and `didChange` off the protocol thread; task 4.1 moved only the `codeLens` path.
- [ ] 4.4 Stop publishing cancellations and internal, snapshot-limit and discovery failures as diagnostics. Route them to `window/logMessage` and status, with backoff from 1 s up to 5 min. Verify: an integration test asserts no diagnostics after cancellation, and retry intervals grow.
- [ ] 4.5 Add per-folder `workspace/configuration` into one serde `Settings` struct (replacing the 16 `*Input` structs), re-read on `didChangeConfiguration`, with defaults for clients without support. Move `**/obj/**` into the exclude default. Verify: an integration test with two folders and different `showPrereleases` values, plus a no-configuration client test.
- [ ] 4.6 Add `update-available` diagnostics from `Plan.status`. Publish them only when the requirement excludes latest, with severity from `versionlens.diagnostics.updateSeverity` (default information; `off` supported), and keep vulnerability Error diagnostics. Clear diagnostics on close. Verify: integration tests for the astro case (diagnostic), the biome `^2.5.8` case (no diagnostic, lens present), `off`, vulnerable+outdated, and close.
- [ ] 4.7 Implement `textDocument/codeAction` with one `quickfix` per bump on any requirement range, `Bump to <req>`, inline edit, latest preferred, and no network. Verify: an integration test asserts the action set and zero HTTP calls during the request.
- [ ] 4.8 Add typed `workspace/executeCommand` commands: update, bulk update (latest/major/minor/patch), sort, choose build, clear cache, manifest info. Update always resolves (applies through `workspace/applyEdit` or returns an error, including the reason for `applied: false`). Never emit a no-op lens. Verify: one integration test per command, including stale-version and rejected-edit cases.
- [ ] 4.9 Add the `versionlens/authorization` server-to-client request with method-not-found fallback, without blocking other resolution. Verify: an integration test with a responding client (retry succeeds) and a non-supporting client (fallback, no hang).
- [ ] 4.10 Remove the remaining VS Code command-ID strings and u8 severity from the engine, and remove the lens-ordering hack. Ordering becomes an explicit field of `LensView`. Verify: `grep -rn "versionlens\.\(suggestion\|update\)" crates/versionlens-core` finds nothing, and the golden lens order is preserved.

## 5. Lane C: parsers and manifest table (subagent 3)

Owns: `crates/versionlens-parsers`, `crates/versionlens-core/src/{classify,document,non_registry}`, `scripts/generate/`, the editor file lists (`packages/jetbrains-plugin/**/Manifests*.kt`, `packages/neovim-plugin/lua/versionlens/support.lua`, `packages/zed-extension/extension.toml`), and the `package.json` `contributes` section.

- [x] 5.1 Add the manifest table `parsers/src/manifest/table.rs` (patterns, ecosystem, format, path queries, version-field exclusions). Replace `manifest.rs` ×3, `classify/tables.rs` and the 4 parallel lists in `ecosystem.rs` with it. Verify: `cargo test -p versionlens-parsers` passes, and the classification test over all fixtures matches the golden ecosystem per file. Done with kind, ecosystem, provider, format and patterns per row. Per-ecosystem data (config names, namespaces, provider IDs) stays in `versionlens-model` as one row table, because providers and napi read it and do not depend on the parsers. Path queries and version-field exclusions move to 5.2.
- [ ] 5.2 Add a span-preserving adapter per format (JSON, TOML, YAML, XML) plus a path-query engine. Port format-based parsers to table rows, including gleam/julia TOML and stack.yaml, and move the TOML/YAML parsing out of core. Exclude the project's own version fields. Add the dependency path queries and version-field exclusions to the 5.1 table rows. Verify: the golden dependency ranges for all format-based fixtures are unchanged, and the package-`version` fixture yields no dependency.
- [ ] 5.3 Replace the in-house glob matcher with `globset`. Verify: the matcher tests pass and `matcher.rs` is deleted.
- [ ] 5.4 Add Cartfile, `dotnet-tools.json` / `.config/dotnet-tools.json` and Cargo `[patch.*]` (in `Cargo.toml` and `.cargo/config.toml`). Verify: new fixtures, each covered by a passing parser test and an engine resolution test with mock HTTP.
- [ ] 5.5 Resolve Gradle catalog `[versions]` entries through referencing libraries (first in file order), or through a plugin marker coordinate. The edit range stays on the `[versions]` value, and unreferenced entries are unresolved with no error. Verify: the Gradle fixtures for library-ref, plugin-only and unreferenced pass.
- [ ] 5.6 GitHub Actions: resolve `${{ env.X }}` from step, then job, then workflow env, keep matrix, and silently skip unresolvable expressions before validation. Verify: the `env.NODE_VERSION` fixture resolves to Node 20, and the `steps.*.outputs` fixture produces no dependency, diagnostic or request.
- [ ] 5.7 Add `versionlens-lsp --print-manifest-table` (JSON) and `scripts/generate/manifests.mjs`. Generate the `package.json` selectors and per-ecosystem settings, the JetBrains list, Neovim `support.lua` and Zed languages. Verify: a check comparing the 0.4.4 key/type/default set with the generated set reports no removed or changed keys (245 preserved), and `bun scripts/generate/manifests.mjs && git diff --exit-code` passes after commit.
- [ ] 5.8 Add `--check` to `scripts/generate/manifests.mjs` to detect drift between generated and committed lists, and run it in `verify`. Verify: hand-editing one generated list makes `bun scripts/generate/manifests.mjs --check` fail and name the entry.

## 6. Lane D: editors and distribution (subagent 4)

Owns: `packages/vscode-extension/src`, `packages/vscode-extension/test`, `packages/neovim-plugin` (except the generated `support.lua`), `packages/zed-extension/src`, `packages/jetbrains-plugin/build.gradle.kts`, `.github/workflows/*`, `scripts/package/*`, `scripts/release/*`.

- [x] 6.1 Add `vscode-languageclient@10.1.x` and bundle it in `scripts/package/extension.mjs`, and raise the engine floor to `^1.91.0` (8.1 and 9.0 type definitions fail `bun run typecheck`). Verify: `bun run package` produces a VSIX whose bundle contains the client and whose manifest has `engines.vscode ^1.91.0`.
- [ ] 6.2 Rewrite the VS Code extension as an LSP client:
  - Keep lifecycle, command registration, SecretStorage auth (answering `versionlens/authorization`) and tasks.
  - Map `versionlens/status` to the existing busy/error context keys.
  - Advertise applyEdit, documentChanges and codeLens.refreshSupport.
  - Show every command error to the user.
  - Delete `native.ts`, `package-watchers`, `codelens.ts`, `apply.ts`, `diagnostics/*` and `session/registry.ts`.

  Verify: `bun run typecheck` and the extension host e2e test on the fixture workspace show lenses, an update diagnostic and a working quick fix.
- [ ] 6.3 Replace the about 7k lines of mock tests with client tests against the real server binary. Verify: `bun test packages/vscode-extension` passes with the server built from the workspace.
- [ ] 6.4 Add the reusable `.github/workflows/build-lsp.yml` (9 targets from `targets.json`, cargo-zigbuild 0.23.0, ziglang 0.15.2, alpine `-crt-static`, archives plus `SHA256SUMS`). Verify: `actionlint`, and a `workflow_dispatch` dry run on a branch produces 9 archives and one `SHA256SUMS` (run only after the user approves pushing the branch).
- [ ] 6.5 Point `vscode.yml` (per-target VSIX), `jetbrains.yml` (bundle prebuilt binaries), `zed.yml`, `neovim.yml`, `release.yml` (assert 9 archives + SHA256SUMS + version sync) and `publish-marketplaces.yml` (no rebuild) at `build-lsp`. Verify: `actionlint`, and `grep -rn "cargo build\|zigbuild" .github/workflows` matches only `build-lsp.yml`.
- [ ] 6.6 Make Zed download the release archive for its version, verify it against `SHA256SUMS`, remove stale versions, and read targets from `targets.json` via `build.rs`. Verify: `cargo test -p versionlens-zed` (or the extension crate) passes checksum-match, mismatch and cleanup tests.
- [ ] 6.7 Add Neovim resolution in the order `cmd`, `$VERSIONLENS_LSP`, bundled, downloaded, `PATH`, plus `auto_download` (default true) into `stdpath("data")/versionlens/<version>/` with SHA-256 verification (`sha256sum`/`shasum`/`certutil`). Delete the binary on mismatch, with an error naming both sums. Update `doc/versionlens.txt`. Verify: the busted/plenary tests pass for each resolution step, mismatch, and `auto_download = false` making no network call.
- [ ] 6.8 Make JetBrains Gradle consume prebuilt binaries from `targets.json` instead of compiling. Verify: `./gradlew buildPlugin` with staged binaries produces a plugin containing all targets.

## 7. Integration: remove napi and finish crate layout (lead, serial)

- [ ] 7.1 Delete `crates/versionlens-napi` and `crates/versionlens-vscode-model`, the napi build scripts, and the napi entries in workflows and `.vscodeignore`. Point `scripts/package/vsix.mjs` at the bundled `versionlens-lsp` binary instead of `versionlens_napi.node`. Verify: `cargo metadata` lists neither crate, and `grep -rn "napi" --include=*.{rs,ts,mjs,yml,toml,json} .` finds nothing outside CHANGELOG history.
- [ ] 7.2 Rename `versionlens-core` to `versionlens-engine`. Reduce re-exports to `pub(crate)` where there is no external user, and remove the `support.rs` wrappers and micro-files kept only for the removed audit. Verify: `cargo build --workspace` and `cargo clippy --workspace -- -D warnings` pass.
- [ ] 7.3 Compare the LSP output for the fixture corpus with `tests/golden/0.4.4`, and record every difference with the spec requirement that causes it in `tests/golden/DIFFERENCES.md`. Verify: the golden comparison test passes with only the listed differences.
- [ ] 7.4 Measure size: product Rust lines (non-test) and VS Code TS lines. Verify: the `tokei`/`wc` output is recorded in the PR description. Target is about 27k Rust and about 1.2k TS. Any overrun is listed with its reason.

## 8. Tests and documentation (lead, serial)

- [ ] 8.1 Run the full regression suite for every group 2.2 fixture across all four editors' paths (engine unit, LSP integration, VS Code e2e, Neovim headless). Verify: all pass, with the command output attached.
- [ ] 8.2 Update `README.md`, `AGENTS.md`, `CONTRIBUTING.md` (Bun 1.4.3, `bun check`, `verify`, new crate names, how to add a file type with one table row), `docs/workspace-source-of-truth.md`, and the per-editor READMEs (credentials, `updateSeverity`, Neovim download). Verify: `markdownlint` passes and every documented command runs.
- [ ] 8.3 Write `CHANGELOG.md` 0.5.0: unified server, update diagnostics, suggestion fixes, error handling, new file types, credentials, distribution, tooling. Verify: the changelog check in `scripts/release` passes.

## 9. Release 0.5.0 (lead, serial)

- [ ] 9.1 Bump every version from 0.4.4 to 0.5.0: workspace crates, the VS Code `package.json`, JetBrains `gradle.properties`, Zed `extension.toml`/`Cargo.toml`, the Neovim rockspec, and `Cargo.lock`/`bun.lock`. Verify: the release version-sync check passes and `grep -rn "0\.4\.4" --exclude=CHANGELOG.md` finds no package version.
- [ ] 9.2 Run full validation. Verify: `bun run verify` passes, with output shown.
- [ ] 9.3 Build release artifacts locally for the host target and package every editor. Verify: `bun run package` and the per-editor package commands succeed, and the host archive's checksum matches the generated `SHA256SUMS`.
- [ ] 9.4 Ask the user before tagging `v0.5.0`, pushing, or publishing to any marketplace.

## Workflow follow-up

- After merge: `openspec archive unify-lsp-and-fix-suggestions` to move the specs into `openspec/specs/`.
