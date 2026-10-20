# Unify on versionlens-lsp and fix suggestion bugs (0.5.0)

Draft PR body for openspec change `unify-lsp-and-fix-suggestions`. Not yet opened.

## Summary

The Rust `versionlens-lsp` binary becomes the only engine for VS Code, JetBrains, Neovim and Zed, and the reported suggestion, error and click bugs are fixed. Release 0.5.0 also aims to shrink the codebase. The size target is not met (see Size).

TODO: fill in before opening the PR.

## Size

Measured at commit 4579859d. Task 7.4 target: about 27k Rust lines and about 1.2k TypeScript lines. The target is missed. Rust is about 47.3k (about 1.75 times the target) and TypeScript is about 2.0k. Rust did not shrink from 0.4.4.

Method: `tokei` is not installed here, so the counts come from a Python line counter. Code lines are non-blank lines that are not comments: `//` lines in Rust, and `//` and `/* */` block lines in TypeScript. Excluded: `tests/` directories, `tests.rs`, `test_support.rs`, `*_test.rs` and `*_tests.rs`, external `cfg(test)` modules, and inline `#[cfg(test)] mod` blocks. TypeScript also excludes `*.test.ts` and `__tests__`.

Cross-check with `wc`/`grep`, run from the repo root. These count non-blank lines, so they include comments and a few `cfg(test)` modules:

```
git ls-files -- crates | grep '\.rs$' | grep -v -E '/tests?/|tests\.rs$|_tests?\.rs$|test_support' | xargs cat | grep -cv '^\s*$'          # 48,332 (572 files)
git ls-files -- packages/vscode-extension/src | grep '\.ts$' | grep -v -E '\.test\.ts$|__tests__' | xargs cat | grep -cv '^\s*$'   # 2,120 (36 files)
```

Baseline 0.4.4 (67f3b1d4), same filters: 46,499 Rust and 3,945 TypeScript non-blank lines.

Rust code lines by crate (0.4.4 names in parentheses):

| Crate | Now | 0.4.4 | Change |
|---|---:|---:|---:|
| parsers | 18,249 | 17,357 | +892 |
| engine | 13,865 | 13,877 (core) | -12 |
| providers | 5,893 | 5,852 | +41 |
| lsp | 3,257 | 2,252 | +1,005 |
| model | 2,213 | 2,235 (model, versions) | -22 |
| update | 2,125 | 1,947 (edits, suggestions) | +178 |
| fetch | 1,703 | 1,416 (cache, http) | +287 |
| napi | deleted | 1,452 | -1,452 |
| Total | 47,305 | 46,388 | +917 |

Rust grew by 917 lines net, even after deleting napi.

### Overrun reasons (Rust)

- parsers (+892): `manifest/table.rs` (1,100) and `manifest/adapter` (619) were added, but the bespoke walkers stayed (`pyproject_toml`, `cargo_toml`, `dotnet_xml`, `maven_xml`, the Gradle catalog, `npmrc`, `nuget_config`). `json_manifest` (1,260) duplicates the JSON adapter. 206 files.
- engine (-12): fetch I/O that design D8 moves to `versionlens-fetch` stayed in `engine/src/fetch` (about 1,850 lines). TOML and YAML parsing remain in `registry/context*.rs`. Presentation code is still split across `session/presentation`, `presentation` and `view`.
- providers (+41): not in the plan's scope. `response/` alone is about 3k lines.
- lsp (+1,005): parity work (commands, config, status, authorization, code actions). Expected growth.
- update (+178): `plan.rs` was added (D3), but the old `choices.rs` and `resolve.rs` are still live. `engine` calls them from `fetch/latest.rs`, `docker.rs` and `resolution/latest/cache.rs`.
- fetch (+287): rate-limit circuit, proxy/TLS handling, and the HMAC persistent cache (507 lines).
- model (-22): merged without reduction.

### Planned vs actual

- Done: napi and vscode-model crates deleted; versions merged into model, suggestions and edits into update, cache and http into fetch; settings structs consolidated; TypeScript non-blank lines down from 3,945 to 2,120.
- Not done: `globset` adoption (D9). The in-house matchers remain in `engine` `workspace/discovery/pattern.rs`, `manifest/patterns.rs` and `path_patterns.rs`. Also not done: moving parsers onto adapters, deleting the old suggestion logic after the plan, moving fetch I/O out of `engine`, and the micro-file merge (fragmentation rules in design.md Context).

### TypeScript

`packages/vscode-extension/src`, non-test: 36 files, 2,043 code lines against a target of about 1.2k. Largest: `extension/auth/set.ts` (229), `extension/client.ts` (200), `extension/auth/store.ts` (164). The `extension/auth/` folder totals 742 lines. That is SecretStorage UX kept on purpose (D1, D7).

### Follow-up

The overrun is recorded here and not fixed in this PR. Cleanup goes to a separate follow-up change. Rough savings, in lines:

1. Move the remaining format parsers onto manifest adapters: 4k to 6k.
2. Route engine `latest` and `docker` through `Plan`, and delete `choices.rs` and `resolve.rs`: 0.6k to 0.9k.
3. Merge `engine/fetch` into `versionlens-fetch`, and unify the presentation layers: 1k to 1.5k.
4. Use `globset` for the three glob matchers: 0.3k.
5. Merge the micro-files and `support.rs` wrappers: 1k to 2k.
6. Consolidate TypeScript `auth/` helpers: 0.2k to 0.3k.

Together these give about 7k to 11k. Rust would land at about 36k to 40k, which is still above 27k with the LSP parity and fetch features added. The follow-up should set a revised target instead of 27k.
