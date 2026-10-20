# Task 8.1 regression run (branch feat/unify-lsp-and-fix-suggestions)

## 1. Engine unit: group 2.2 fixtures
`cargo test -p versionlens-engine fixtures`
```
test session::checking_matrix::fixtures::branch_action_ref_is_fixed_without_a_lookup ... ok            (taiki-e/install-action@just)
test ...::caret_range_behind_latest_offers_a_floor_bump_without_a_diagnostic ... ok                     (^2.5.8 -> 2.5.14)
test ...::cargo_bare_version_is_a_caret_requirement ... ok                                              (Cargo "1.0.200")
test ...::cargo_exact_pin_at_latest_is_current ... ok                                                   (=0.7.0)
test ...::gradle_catalog_plugin_alias_uses_the_plugin_marker ... ok
test ...::gradle_catalog_version_alias_resolves_through_its_library ... ok                              (Gradle [versions] alias)
test ...::intermediate_releases_are_not_listed_individually ... ok                                      (starlight)
test ...::package_version_fields_are_not_dependencies ... ok                                            (package version)
test ...::python_exact_pin_at_latest_is_current ... ok                                                  (requests==2.34.2)
test ...::sha_pinned_action_with_floating_and_exact_tags_is_current ... ok                              (github-script SHA, v9/v9.0.0)
test ...::unresolvable_step_output_expression_is_skipped_silently ... ok                                (${{ steps.* }})
test ...::unsupported_setup_dotnet_syntax_is_not_supported_rather_than_an_error ... ok
test ...::vscode_engine_field_is_skipped_without_an_error ... ok                                        (engines.vscode)
test ...::workflow_env_reference_resolves_the_runtime ... ok                                            (${{ env.X }})
test result: ok. 14 passed; 0 failed; 0 ignored; 734 filtered out
```
Golden: `cargo test -p versionlens-engine golden`
```
every_update_lens_changes_the_document ... ok
engine_output_matches_the_golden_folder ... ok
golden_differences_with_the_baseline_are_documented ... ok
capture_golden_output ... ignored (writes tests/golden)
test result: ok. 3 passed; 0 failed; 1 ignored
```

## 2. Full workspace
`cargo test --workspace` (all results ok, 0 failed)
```
versionlens_engine     747 passed; 0 failed; 1 ignored
versionlens_fetch       85 passed
versionlens_lsp         79 passed (src/lib.rs unit+integration, in-process); main.rs 0
versionlens_model       40 passed; tests/github.rs 2 passed
versionlens_parsers    384 passed
versionlens_providers  155 passed
versionlens_test_support 0 + tests/fixture.rs 2 passed
versionlens_update     133 passed
doc-tests: 0 tests, all ok
```

## 3. LSP integration
`cargo test -p versionlens-lsp` -> `test result: ok. 79 passed; 0 failed`
(There is no crates/versionlens-lsp/tests dir; LSP tests live in src/ via test_support.rs.)
These cover the shared LSP path used by JetBrains, Zed and Neovim; none of them reference the 2.2 fixtures by content.

## 4. VS Code e2e
`bun run test:e2e` -> builds LSP + extension, launches VS Code (test-electron)
```
e2e: lenses, update diagnostic and quick fix OK
Exit code:   0
```
Fixture workspace: tests/fixtures/vscode-e2e/package.json (single dep alpha 1.0.0 vs stub registry).

## 5. Neovim headless
`cd packages/neovim-plugin && make test` (plenary from .deps/plenary.nvim)
```
setup_spec.lua   Success: 6  Failed: 0  Errors: 0
resolve_spec.lua Success: 11 Failed: 0  Errors: 0
support_spec.lua  (also run by make test; see tail of run, no failures)
```
Note: stderr shows deprecation "vim.lsp.codelens.refresh({ bufnr }) is deprecated" (not a failure).

## 6. LSP path: group 2.2 fixtures (branch test/8.1-lsp-fixtures)
`cargo test -p versionlens-lsp fixtures`
The LSP tests (`crates/versionlens-lsp/src/state/fixtures.rs`) open each 2.2 manifest in the server state, resolve it with the stubbed registry answers, and assert the LSP lens titles, bump versions (lens command arguments) and update diagnostics. Manifests and registry bodies are shared with the engine tests via `versionlens_test_support::regression`.
```
test result: ok. 14 passed; 0 failed; 0 ignored; 79 filtered out
```
Covered: =0.7.0, requests==2.34.2, ^2.5.8 -> 2.5.14, starlight list, github-script SHA v9/v9.0.0, install-action@just, engines.vscode, setup-dotnet unsupported, env.X, steps.*, Gradle version + plugin alias, Cargo "1.0.200", package version.
Re-run after the refactor: `cargo test -p versionlens-engine fixtures` 14 passed; `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all -- --check` clean.
Sanity check: changing one expected status made the matching test fail, then reverted.

## Coverage notes
- All 14 engine fixtures (every 2.2 item) run at engine level (checking_matrix/tests/fixtures.rs) and again through the LSP state (section 6), so JetBrains, Zed and Neovim are covered by proxy.
- VS Code e2e still exercises only the generic alpha dependency, and the Neovim specs only plugin glue; both reach the same LSP, which now covers the 2.2 fixtures.
- LSP fixture tests stub registries by response injection (no server process or transport); JetBrains/Zed have no test run of their own here.
