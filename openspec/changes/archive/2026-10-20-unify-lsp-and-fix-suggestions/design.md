# Design

## Context

VersionLens Redux 0.4.4 ships two front ends over one Rust core:

- **VS Code** loads `versionlens_napi.node` in-process. It uses the `versionlens-vscode-model` payload types, and its TypeScript re-implements watchers, lens rendering, diagnostics and apply logic.
- **JetBrains, Neovim and Zed** spawn `versionlens-lsp`. It honors about 8 of the 245 settings and is missing VS Code features (clear cache, bulk update, sort, authorization, status, tasks).

Audit findings that shape this design:

- **Core leaks VS Code concerns.** Command IDs are hard-coded in `core/presentation/mod.rs`, and lens payloads use positional string arguments that the LSP decodes again in `lsp/state/presentation.rs`. There is also a lens-ordering hack and u8 severity magic.
- **Editing destroys state.** `didOpen` and `didChange` bump the workspace epoch, clear caches and restart discovery (`lsp/server.rs:403-418` → `core/session/checking.rs:94-113` → `workspace.rs:51-86`). File I/O runs on the protocol thread.
- **Failures become diagnostics.** Cancellations and internal errors are published as WARNING diagnostics and retried every second (`lsp/server/checking.rs:131-158, 322-336`).
- **Threading is hand-rolled.** A custom stdio transport duplicates `lsp_server::Connection::stdio`, alongside a 200 ms tick loop, two tokio runtimes and `panic = "abort"`.
- **"Is current" is decided in four places that disagree.** `resolve.rs`, `choices.rs`, `range/update.rs` and `dependency.rs:329-334` cause the reported duplicate-latest, missing-floor-bump and intermediate-spam bugs.
- **Fragmentation.** Over 233 core files are forced by `scripts/architecture_audit`, which sets a 120-line microfile limit and a 12-file flat-cluster limit. Ecosystem tables repeat about 10 times, and a new file type touches about 11 files.
- **Builds.** Each release makes 27 native builds, with target maps duplicated in 5 places and no checksums.

## Goals / Non-Goals

**Goals:**
- One engine (`versionlens-lsp`) for all four editors, with feature parity for what VS Code offers today.
- Fix every reported suggestion, error and click bug, with regression tests built from the reported manifests.
- Update-available diagnostics and quick fixes in every LSP client.
- One 9-target build matrix with checksums. All editors at 0.5.0.
- Bun-native tooling (`bun check`), without the `typescript` package.
- Reduce product Rust to about 27k lines and VS Code TypeScript to about 1.2k, with no new runtime dependencies beyond `vscode-languageclient`, `lsp-server` (already used) and `globset`.
  - Target missed: measured about 47.3k Rust and about 2.0k TS at commit 4579859d. See `pr-description.md` in this change folder. Reductions are deferred to a follow-up change.

**Non-Goals:**
- Rewriting the engine in TypeScript.
- Changing settings keys or command IDs. The VS Code engine floor rises from `>=1.75.0` to `^1.91.0`, the minimum of `vscode-languageclient` 10.
- New ecosystems. New file types are added only within existing ecosystems.
- A web or remote extension host build.

## Decisions

### D1. VS Code becomes an LSP client; napi is deleted
VS Code uses `vscode-languageclient@8.1.x`, the last major that supports engine `^1.67`. 9.x requires 1.82. The client is bundled by `scripts/package/extension.mjs`, because vsce runs with `--no-dependencies`. The extension keeps lifecycle, command registration, SecretStorage auth and tasks, and replaces everything else with the server.

*Alternative:* keep napi for VS Code and share more code. This was rejected because it keeps two runtimes and 9 extra builds, and every fix would still need two code paths.

### D2. Editor-neutral presentation in the engine
The engine returns typed values: `LensView { range, kind, title, action: Option<Action> }`, `Action` (an enum of typed commands with named fields), and `Severity` as an enum. Only `versionlens-lsp` maps them to LSP types and command IDs. Command IDs stay the same strings that VS Code contributes today, so keybindings and menus keep working. `versionlens-vscode-model` is deleted.

### D3. One currency rule and one suggestion builder
A single function, `update::plan(req, dialect, versions) -> Plan { status, bumps }`, replaces the logic in `resolve.rs`, `choices.rs`, `range/update.rs` and the `retain` in `dependency.rs`:

- `status` uses the dialect's `minimum_version` and `satisfies`. Exact pins count as targeting their version.
- `bumps` is at most {newest satisfying, newest patch, newest minor, newest major, latest}, deduplicated by rendered text, and drops any bump whose text equals the current requirement.
- Version comparisons use parsed versions, never string prefixes.
- GitHub refs compare commit SHAs and pick the most specific tag on the commit.

Code lenses, diagnostics and code actions are all built from the same `Plan`, so they cannot disagree.

### D4. Update diagnostics and code actions
`update-available` diagnostics are published when the requirement does not allow latest, or when its floor bump to latest changes the major or minor number (`^7.2.2` → `^7.3.5`). Patch-only floor bumps (`^2.5.8` → `^2.5.14`) remain lenses and code actions but raise no diagnostic, so ranges a few patches behind do not fill the Problems panel. The severity comes from `versionlens.diagnostics.updateSeverity`, which takes `error`, `warn`, `info`, `hint` or `off` and defaults to `info`. Hint is not the default because VS Code's Problems panel does not list Hint. Vulnerability diagnostics keep Error. `textDocument/codeAction` answers for any requirement range, reads the last `Plan` for the document version, with no network access, and returns quick fixes with inline edits. The latest bump is marked preferred.

### D5. Runtime: lsp_server stdio, one tokio runtime, incremental state
- **Transport:** use `lsp_server::Connection::stdio()` and delete `server/transport.rs` and the 200 ms tick.
- **Threads:** the protocol thread only routes messages. A single multi-thread tokio runtime does resolution, file I/O and HTTP. Results return through a channel and are published by the protocol thread.
- **Incremental state:** documents are keyed by URI and version. On change, re-parse the document and diff its dependencies by (name, requirement, source), then resolve only the changed ones. The registry cache, workspace discovery and epoch are kept. Discovery restarts only on `workspace/didChangeWatchedFiles` for manifest or lockfile paths, or when workspace folders change.
- **Push-driven:** resolution starts on open/change, debounced 300 ms. Diagnostics are published on completion, and `workspace/codeLens/refresh` is sent when the client supports it.
- **Failures:** cancellations are dropped. Internal and discovery failures go to `window/logMessage` and `versionlens/status`, with exponential backoff from 1 s up to 5 min.
- **Panics:** set `panic = "unwind"` and catch panics per task, so one bad manifest cannot kill the server.

*Alternative:* tower-lsp. Rejected because it adds a dependency and its async model would need restructuring. `lsp-server` is already used.

### D6. Configuration via workspace/configuration
The server pulls the `versionlens` section for each workspace folder and caches it until `didChangeConfiguration`. A single `Settings` struct is deserialized with serde defaults. It replaces 16 `*Input` structs and the 3–4 separate mappings. Clients without configuration support get the defaults. The hard-coded `**/obj/**` exclusion moves into the default of the existing exclude setting.

### D7. Credentials and rate limits
- **Credentials:**
  - Credential order: client-supplied (`versionlens/authorization`), then `GITHUB_TOKEN`/`GH_TOKEN`, then `gh auth token`, then the existing registry config files (`.npmrc`, `pip.conf`, `nuget.config` and so on).
  - `versionlens/authorization` is a server-to-client request with `{ host, status }`, answered with `{ scheme, token } | null`. VS Code answers from SecretStorage and prompts at most once per host per session. Other clients answer method-not-found, and the server falls back to the other sources.
- **Rate limits:**
  - Rate-limit headers are kept through the fetch layer.
  - A per-host circuit holds requests until the reset time.
  - One `window/showMessage` is sent per reset period.

### D8. Crate layout
| 0.4.4 | 0.5.0 |
|---|---|
| versions, model | `versionlens-model` (types, dialects, comparators) |
| suggestions, edits | `versionlens-update` (plan, bumps, text edits) |
| cache, http, core fetch I/O | `versionlens-fetch` (HTTP, proxy/TLS, rate limit, persistent cache) |
| core | `versionlens-engine` (sessions, resolution, presentation views) |
| parsers (+ TOML/YAML parsing from core) | `versionlens-parsers` |
| providers, lsp, test-support | unchanged names |
| napi, vscode-model | deleted |

- **Proxy and TLS:** `versionlens-fetch` honors `*_PROXY`/`NO_PROXY`. Its `reqwest` client verifies certificates against the platform trust store through `rustls-platform-verifier`, so no new dependency is needed (verified in task 3.4).
- **Cache:** the persistent cache stays one versioned JSON file. Its record keys stay a keyed HMAC, because they hash the session config, including credentials (user decision in task 3.4). It falls back to memory, with one warning, if its directory is unwritable.

### D9. Declarative manifest table and generated editor lists
- **Manifest table:** `parsers/src/manifest/table.rs` holds one row per file type: patterns, ecosystem, format, dependency path queries and version-field exclusions.
- **Parsing:** format-based file types use a shared span-preserving adapter per format (JSON, TOML, YAML, XML) and a small path-query engine. Bespoke formats such as Gemfile, Cartfile and go.mod keep hand-written parsers but register through the same table.
- **Generated outputs:** `scripts/generate/manifests.mjs` writes `package.json` document selectors and per-ecosystem settings (all 245 keys preserved), the JetBrains Kotlin list, Neovim `support.lua` and Zed `extension.toml` languages. The table is exported through `versionlens-lsp --print-manifest-table`, which emits JSON.
- **Drift check:** `bun scripts/generate/manifests.mjs --check` compares the generated output with the committed files and names each stale entry. The glob matching uses `globset` instead of the 486-line in-house matcher.

### D10. Distribution
- **Build workflow:** a reusable `.github/workflows/build-lsp.yml` (`workflow_call`) builds the 9 targets with cargo-zigbuild 0.23.0 and ziglang 0.15.2. Musl and MSVC targets build static executables (`+crt-static`), and gnu targets build with a glibc 2.28 floor. It uploads `versionlens-lsp-<version>-<target>.{tar.gz,zip}` and `SHA256SUMS`.
- **Targets:** one target map, `scripts/package/targets.json`, read by the scripts, Gradle, Zed (via `build.rs` include) and Neovim (generated).
- **Editors:**
  - **VS Code:** builds one VSIX per target.
  - **JetBrains:** bundles all binaries in one plugin.
  - **Zed:** downloads and verifies the archive.
  - **Neovim:** resolves `cmd`, then `$VERSIONLENS_LSP`, then a bundled binary, then (development checkouts only) the repository's `target/debug` binary, then a downloaded one, then `PATH`. If none is found and `auto_download` is on, it downloads into `stdpath("data")/versionlens/<version>/` and verifies against `SHA256SUMS`, using `curl`, `tar` (Windows: `System32\tar.exe`) and `sha256sum`/`shasum`/`certutil`, whichever is present. The download is asynchronous with a per-process scratch directory, buffers opened meanwhile start when it finishes, and a failure is remembered until restart. The target-to-triple map in `download.lua` is generated from `targets.json` by `scripts/generate/manifests.mjs`; Alpine is detected through `/etc/alpine-release` and gets the musl build.
- **Release:** `release.yml` asserts 9 archives plus `SHA256SUMS`, pins the Bun version from `package.json` `packageManager`, and fails on version drift between crates and editor packages.

### D11. Bun tooling
- **Type checking:** `bun check` (Bun 1.4.3, typescript-go) replaces `tsc`. `typecheck` becomes `bun check --cwd packages/vscode-extension`. Remove the `typescript` dev dependency, the root `tsconfig.json`, `tests/tsconfig.json`, `composite`, `outDir` and `tsBuildInfoFile`.
- **Script rename:** the root `check` script is renamed to `verify` so it stops shadowing the builtin. The justfile, lefthook, workflows, AGENTS.md and CONTRIBUTING.md are updated with it.
- **Minimum version:** set `engines.bun`/`packageManager` to `>=1.4.3`.

### D12. Remove the in-house lint scripts
Delete the custom lint and audit scripts: `scripts/architecture_audit`, `scripts/audit_architecture.py`, `scripts/providers*` (capability audit), and `scripts/check/*` (code-quality, data-object, napi-boundary, VS Code adapter, editor, Neovim and VSIX checks), plus jscpd. Remove their `package.json` scripts, lefthook hooks and CI steps. `scripts/` keeps only build, packaging, release, test and benchmark scripts. One packaging check stays: `scripts/package/vsix.mjs` confirms that each per-platform VSIX contains the binary built for its target. Boundaries are enforced by crate dependencies, Biome, clippy, rustfmt and the type checker. Then merge the micro-files and `support.rs` wrappers that only existed to satisfy the removed audit.

## Risks / Trade-offs

- [VS Code behavior regressions during the client rewrite] → Before deleting napi, capture the 0.4.4 lens, diagnostic and edit output for the fixture corpus as golden files, and require the LSP path to match them, except where a spec in this change requires a difference.
- [Child-process startup latency in VS Code vs in-process napi] → The binary starts in under 50 ms. Lenses appear on first resolution, as they do today. Measure activation-to-first-lens on the fixture workspace and record it in the PR.
- [Per-platform VSIX misses a platform users had] → All 9 napi targets are kept as LSP targets, including armhf and alpine. A universal fallback VSIX is not published.
- [Raising the engine floor to 1.91 drops VS Code 1.75–1.90] → `vscode-languageclient` 8.1 and 9.0 type definitions fail `bun run typecheck`, and 10.1 is the first that passes. 10.1 needs VS Code `^1.91.0`.
- [Generated `package.json` settings could change defaults] → A check diffs the 0.4.4 key/type/default set against the generated set (manifest-support spec).
- [Large parallel refactor causes merge conflicts] → Work is split into 4 lanes with disjoint file ownership and fixed integration points (tasks.md). Merges go through one integration branch in a fixed order.
- [Gradle catalog lookup picks the wrong coordinate when one version is shared by unrelated libraries] → Use the first library in file order, and plugins only when there is no library. Document the behavior and cover it with a fixture.
- [`gh auth token` subprocess cost or hangs] → Run it once per session with a 2 s timeout, and only after a 401/403 or rate limit from GitHub.

## Migration Plan

1. Land the tooling changes and the editor-neutral presentation first. Both are compatible with napi.
2. Land the runtime, suggestion and diagnostics fixes in the engine and LSP. Napi still works, so VS Code users of interim builds keep working.
3. Bring the LSP to parity (commands, config, status, authorization), switch VS Code to the client, then delete napi and vscode-model.
4. Merge crates and generate editor lists.
5. Switch CI to `build-lsp` and release 0.5.0 to all marketplaces at once.

Rollback: 0.4.4 artifacts stay published. If 0.5.0 must be pulled, re-publish 0.4.4 as 0.5.1 from its tag. Settings, command IDs and SecretStorage keys are unchanged, so user state carries over both ways.

## Open Questions

None blocking. Gradle catalog coordinate selection (see Risks) can be refined after release if users report mismatches.
