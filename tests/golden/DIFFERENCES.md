# Differences from the 0.4.4 golden output

Baseline: `tests/golden/0.4.4` as captured in commit `67f3b1d4` (the `v0.4.4`
tag has no golden files). The working folder is recaptured from the engine
whenever a golden change is approved, so it already contains the differences
below. Everything not listed here is byte-identical to the baseline.

The comparison is on engine output rendered by the golden renderer
(`view/legacy.rs`, which applies the 0.4.4 range offset), not on raw LSP JSON.

Reproduce:

```sh
git diff 67f3b1d4 -- tests/golden/0.4.4
cargo test -p versionlens-engine golden
```

`engine_output_matches_the_golden_folder` checks the engine against the working
folder. `golden_differences_with_the_baseline_are_documented` fails when a
0.4.4 file differs from the baseline without a `###` heading below, when a
heading is stale (file identical or missing), or when the folder has a file the
baseline lacks. Without git history it is skipped locally and fails under `CI`. Spec paths are under
`openspec/changes/unify-lsp-and-fix-suggestions/specs/`.

## Changed files

### `gradle-catalog/gradle/libs.versions.toml.json`

- Change: `updateAllEdits` gains one edit (`2.0.0` at line 1, characters 48-53).
- Cause: commit `0884a49e`. A `[versions]` entry now resolves through the
  library or plugin that references it, so an entry below latest offers an
  update and joins update-all.
- Spec: `manifest-support/spec.md`, "Gradle version catalog references".

### `r-description/DESCRIPTION.json`

- Change: the `demo` status lens ("unresolved") no longer carries the
  `versionlens.suggestion.onUpdateDependency` command or arguments (empty
  command), and its no-op `update` action with empty edits is gone.
  The lens title still shows; only the empty-edit update command is gone.
- Cause: commit `2ae8a837`. A lens that applies no text change is no longer
  given an update command.
- Spec: `version-suggestions/spec.md`, "No suggestion without a text change"
  (scenario "Status lens without an update has no update command"), and
  tasks.md 4.8 (update always resolves, so a no-op command is not offered).

## Added files (not in the baseline)

New in 0.5.0, under `tests/golden/0.5.0/`; they have no 0.4.4 counterpart:

- `carthage/Cartfile.json`
- `dotnet-tools/.config/dotnet-tools.json.json`
- `cargo-config/.cargo/config.toml.json`

Spec: `manifest-support/spec.md`, "Additional file types".

## Unexplained

None.
