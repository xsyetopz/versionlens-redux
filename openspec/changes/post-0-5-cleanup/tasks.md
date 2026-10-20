## 1. Baseline and docs

- [x] 1.1 Fix the heading and table header in `docs/workspace-source-of-truth.md` from "0.4.0" to the current layer name without a version (or "0.5.0"). Verify: `grep -n "0\.4\.0" docs/workspace-source-of-truth.md` prints nothing.
- [ ] 1.2 Record the baseline counts from design.md in the final PR text, using the same `wc` commands. Verify: the totals match 48,794 Rust and 2,120 TS at the starting commit.

## 2. Spec fix

- [ ] 2.1 Confirm the delta in `specs/editor-distribution/spec.md` has each new requirement under 500 characters. Verify: `openspec validate post-0-5-cleanup --strict`.
- [ ] 2.2 After archiving, run `openspec validate --specs --strict`. Verify: 6 of 6 specs pass.

## 3. Test-server hazard

- [ ] 3.1 Move `accept_within_deadline` from `crates/versionlens-fetch/src/http/client/tests.rs` into `crates/versionlens-test-support` and use it from the fetch tests. Verify: `cargo test -p versionlens-fetch --locked`.
- [ ] 3.2 Use the helper at `fetch/src/http/client/tests/rate_limit.rs:101` and `engine/src/test_support.rs:177`. Verify: `cargo test -p versionlens-fetch -p versionlens-engine --locked`.
- [ ] 3.3 Use the helper (or `tokio::time::timeout` for async) in engine `session/resolution/tests/npm/cache.rs` (:61, :150), `tests/github.rs:22` and `fetch/http/tests.rs:109`. Verify: `cargo test -p versionlens-engine --locked`.
- [ ] 3.4 Use the helper in LSP `server/tests/updates.rs`, `responsiveness.rs`, `refresh.rs`, `authorization.rs` and `rate_limit.rs`. Verify: `cargo test -p versionlens-lsp --locked`.
- [ ] 3.5 Prove the fix: temporarily make one test never connect and confirm it fails within the deadline, then revert. Verify: `grep -rn "\.accept()" crates --include='*.rs'` lists only the helper itself and justified loops.

## 4. CI

- [ ] 4.1 Add `.github/workflows/ci.yml` with `checks`, `lsp`, `vscode`, `jetbrains`, `neovim`, `zed` and `e2e` jobs as described in design.md. `checks` runs `bun run check:manifests` and `bun run check:manifest-baseline`; `e2e` runs `bun run test:e2e`. Verify: `bun run check:manifests && bun run check:manifest-baseline` locally; `bun run test:e2e` locally.
- [ ] 4.2 Remove `pull_request` and `push` triggers from `vscode.yml`, `jetbrains.yml`, `neovim.yml` and `zed.yml`. Verify: `grep -n "pull_request\|push:" .github/workflows/{vscode,jetbrains,neovim,zed}.yml` prints nothing, and `bun run check:biome` passes.
- [ ] 4.3 Open the PR and confirm one `build-lsp` run (9 targets) on the PR checks and on the merge to master. Verify: Actions run list on the PR; update required check names in repository settings.

## 5. Size reduction (one commit per step; no behavior change)

- [ ] 5.1 Dead code pass: remove unused items reported by `cargo clippy --workspace --all-targets` and unused `pub` items found by grep. Verify: `cargo clippy --workspace --all-targets` and `cargo test --workspace --locked`.
- [ ] 5.2 Retire `update/src/choices.rs` and `resolve.rs`: route engine `fetch/latest.rs`, `docker.rs` and `resolution/latest/cache.rs` through `Plan`. Verify: `cargo test -p versionlens-update -p versionlens-engine --locked`.
- [ ] 5.3 Merge `engine/src/fetch` into `versionlens-fetch` and remove duplicated client and retry code. Verify: `cargo test -p versionlens-fetch -p versionlens-engine --locked`.
- [ ] 5.4 Unify `session/presentation`, `presentation` and `view` in `engine`. Verify: `cargo test -p versionlens-engine -p versionlens-lsp --locked`.
- [ ] 5.5 Merge the three glob matchers into one (`globset` only if already in `Cargo.lock`). Verify: `cargo test -p versionlens-engine -p versionlens-parsers --locked`.
- [ ] 5.6 Move bespoke parsers onto manifest adapters, one format per commit, largest first (`json_manifest`, `npmrc`, `nuget_config`, `gradle`, `pyproject_toml`, `cargo_toml`, `dotnet_xml`, `maven_xml`, then `cpp`, `hackage`, `rebar_config`). Verify per format: `cargo test -p versionlens-parsers --locked`, then `bun run check:manifests && bun run check:manifest-baseline` after the last one.
- [ ] 5.7 Merge micro-files and `support.rs` wrappers into parent modules. Verify: `cargo fmt --all -- --check` and `cargo test --workspace --locked`.
- [ ] 5.8 Consolidate `extension/auth/` helpers and trim `client.ts`. Verify: `bun run typecheck` and `bun run test:extension`.
- [ ] 5.9 Re-measure with the design.md commands. Target: Rust 40k or fewer, TS 1.8k or fewer. If short, record the numbers and the reason in the PR text. Verify: command output pasted in the PR.

## 6. Final check

- [ ] 6.1 Run `bun run verify` (needs Java 21 for JetBrains) and `bun run test:e2e`. Verify: both exit 0.
