# Design

## Context

Counts below use the method from the archived `pr-description.md`: git-tracked files, excluding `tests/` directories, `tests.rs`, `*_test(s).rs` and `test_support*`, counting non-blank lines (comments included). Run at commit d00908b2:

```
git ls-files -- crates/<crate> | grep '\.rs$' | grep -v -E '/tests?/|tests\.rs$|_tests?\.rs$|test_support' | xargs cat | grep -cv '^\s*$'
git ls-files -- packages/vscode-extension/src | grep '\.ts$' | grep -v -E '\.test\.ts$|__tests__' | xargs cat | grep -cv '^\s*$'
```

| Crate | Lines |
|---|---:|
| parsers | 18,448 |
| engine | 14,260 |
| providers | 5,903 |
| lsp | 3,723 |
| model | 2,237 |
| update | 2,165 |
| fetch | 1,767 |
| test-support | 291 |
| Total | 48,794 |
| VS Code TS (36 files) | 2,120 |

The archived figure of 47.3k came from a counter that also dropped comment lines and inline `#[cfg(test)]` blocks. The `wc` total here (48.8k, or 48.5k without test-support) is the number to track from now on, because anyone can reproduce it. The archived `lsp` figure (3,257) is lower than today's 3,723 for the same reason.

There are 168 product Rust files under 25 lines.

## Target

- Rust: 40k or fewer by the `wc` method (about 18% down), stretch 37k. Do not chase 27k. The 27k goal was set before LSP parity work added about 1k lines to `lsp` and about 0.3k to `fetch`, and before the rate-limit circuit and HMAC cache were kept. The savings found below sum to about 7k to 10k, so 38k to 41k is what the code supports.
- TypeScript: 1.8k or fewer, stretch 1.7k. The `extension/auth/` folder (about 740 lines) is SecretStorage UX kept on purpose, so only helper overlap is cut. 1.2k would need removing features.
- Each step below must keep `cargo test --workspace --locked` and the golden and e2e tests green. A step that cannot cut lines without behavior change is dropped, not forced.

## Candidates by file

1. Format parsers onto manifest adapters (largest, about 3k to 5k). `parsers/src/manifest/table.rs` (1,108) and `manifest/adapter/` exist, but bespoke walkers remain: `json_manifest`, `pyproject_toml`, `cargo_toml`, `dotnet_xml`, `maven_xml`, `gradle/catalog.rs` (378), `gradle/build.rs` (376), `npmrc/mod.rs` (486), `dotnet_sources/nuget_config.rs` (386). Also `cpp.rs` (486), `hackage.rs` (439), `rebar_config/mod.rs` (432), and `support.rs` (431) with `support/`. Move formats one at a time, in order of size, and compare output with the existing parser tests and golden snapshots.
2. Retire `update/src/choices.rs` (470) and `resolve.rs` by routing engine `fetch/latest.rs`, `docker.rs` and `resolution/latest/cache.rs` through `Plan` (about 0.6k to 0.9k).
3. Merge `engine/src/fetch` (about 1.85k: `http`, `latest`, `runtime`) into `versionlens-fetch`, and drop the duplicated client and retry code. Unify the three presentation layers (`session/presentation`, `presentation`, `view`). About 1k to 1.5k.
4. Adopt `globset` for `engine` `workspace/discovery/pattern.rs`, `manifest/patterns.rs` and `parsers/src/path_patterns.rs` (about 0.3k). `globset` is a new dependency. AGENTS.md says no new dependencies unless required, so check whether it is already in `Cargo.lock` first. If it is not, drop this item and merge the three matchers into one in-house module instead.
5. Merge micro-files (168 files under 25 lines) and `support.rs` wrappers into their parent modules (about 1k to 2k, mostly `mod` and `use` lines).
6. Dead code: run `cargo clippy --workspace --all-targets` with `dead_code` and `unused` visible, and remove unused `pub` items found by grep across crates (size unknown; take what is found).
7. TypeScript: merge overlapping helpers in `extension/auth/` (`set.ts` 249, `store.ts` 180, `authorization.ts` 100, `value.ts` 95, `remove.ts` 88) and trim `client.ts` (234). About 0.2k to 0.3k.

Large single files such as `lsp/server/work.rs` (729), `engine/session/discovery.rs` (606) and `storage.rs` (578) are not cut unless step 3 or 5 shows overlap. Splitting them would add lines.

## Test-server hazard

`c41fbee7` added `accept_within_deadline` to `fetch/src/http/client/tests.rs` (nonblocking accept polled for 5 s, returning `None` on timeout). Move it to `versionlens-test-support` (already a dev-dependency of every crate) and use it everywhere. Verified `accept()` sites:

- `versionlens-fetch/src/http/client/tests/rate_limit.rs:101` (`serve_in_order`, `.unwrap()` then `join`)
- `versionlens-engine/src/test_support.rs:177`
- `versionlens-engine/src/session/resolution/tests/npm/cache.rs:61` and `:150`
- `versionlens-engine/src/session/resolution/tests/github.rs:22`
- `versionlens-engine/src/fetch/http/tests.rs:109` (async `accept().await`; wrap in `tokio::time::timeout`)
- `versionlens-lsp/src/server/tests/updates.rs:92`, `responsiveness.rs:18`, `refresh.rs:10`, `authorization.rs:43`, `rate_limit.rs:31`

A helper that returns `None` on timeout lets each server thread end, so `join` returns and the test's own assertions fail with a message. Sites that already loop on `let Ok(..) else break` are checked case by case.

## CI

Today `vscode.yml`, `jetbrains.yml` and `neovim.yml` each trigger on `pull_request` and push to master, and each calls `build-lsp.yml`. That is 3 builds of the 9-target matrix per event. `zed.yml` does not build the server. `release.yml` already shows the right pattern: build once, pass `lsp-artifact` to each editor workflow.

Plan:

- Add `.github/workflows/ci.yml` with triggers `pull_request`, `push` to master and `workflow_dispatch`. Jobs: `checks` (Bun, Rust, `bun install --frozen-lockfile`, `check:manifests`, `check:manifest-baseline`), `lsp` (calls `build-lsp.yml` once), `vscode`, `jetbrains`, `neovim` (each `uses` its workflow with `lsp-artifact: ${{ needs.lsp.outputs.archives-artifact }}`), `zed`, and `e2e` (needs `checks`, runs `bun run test:e2e`).
- Remove `pull_request` and `push` from the four editor workflows; keep `workflow_dispatch` and `workflow_call`. A manual dispatch still builds its own server since `lsp-artifact` is empty.
- The e2e script runs `cargo build -p versionlens-lsp` and `bun run build`, so `e2e` does not need the cross-built archives.
- The `if:` guards that accept `skipped` for `needs.lsp` stay, since they cover the manual path.

If the reusable-workflow output name differs from `archives-artifact`, use the one `build-lsp.yml` declares (see `release.yml` line 73).

## Spec delta

`editor-distribution` requirement "JetBrains and Zed use release artifacts" is 560 characters. A REMOVED delta drops it. Two ADDED requirements, "JetBrains uses release artifacts" and "Zed uses release artifacts", hold the JetBrains and Zed text. Scenarios move with their subject. Wording is otherwise unchanged. No other capability changes, since parser and fetch refactors keep behavior.

## Risks

- Parser moves can shift ranges or edge cases. Mitigation: one format per commit, existing parser tests and golden snapshots unchanged.
- Moving workflow triggers can leave branch protection waiting on old check names. Mitigation: list required checks before merge and rename them in the repository settings.
