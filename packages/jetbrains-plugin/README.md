# VersionLens Redux for JetBrains IDEs

The JetBrains plugin starts the shared `versionlens-lsp` server through the IntelliJ Platform LSP API.

## Package

- Plugin id: `com.versionlens.jetbrains`
- Display name: `VersionLens Redux`
- Build file: [`build.gradle.kts`](build.gradle.kts)
- Runtime server: `crates/versionlens-lsp`

The plugin is written in Kotlin and targets IntelliJ IDEA through the IntelliJ Platform Gradle plugin. The checked-in wrapper pins Gradle 9.6.1; dependency locks and checksum verification make resolution reproducible, and CI disables the Gradle daemon.

## Build the language server

From the repository root:

```bash
cargo build -p versionlens-lsp
```

The plugin does not compile the server. It bundles the prebuilt release binaries of every target in [`scripts/package/targets.json`](../../scripts/package/targets.json) as `bin/<target>/versionlens-lsp[.exe]` (for example `bin/darwin-arm64/`), and picks the one for the running OS and architecture (`alpine-*` on musl Linux, `linux-armhf` on ARMv7). The plugin resolves the server in this order:

1. `versionlens.lsp.path` Java system property.
2. `VERSIONLENS_LSP` environment variable.
3. Packaged `bin/<target>/versionlens-lsp` beside the installed plugin libraries.
4. A JAR-embedded `/bin/<target>/versionlens-lsp` extracted to the IDE system directory.
5. Repository-local debug binary at `target/debug/versionlens-lsp`.
6. `versionlens-lsp` on `PATH`.

## Build the plugin

Stage the release binaries as `<dir>/<rust-target>/versionlens-lsp[.exe]` from the build-lsp archives, then build. The default directory is `build/lsp-binaries`; override it with `-PversionlensBinariesDir=<dir>`.

```bash
for rust_target in $(jq -r '.targets[].rust' scripts/package/targets.json); do
  bun scripts/package/stage-lsp.mjs <archives-dir> "$rust_target" "packages/jetbrains-plugin/build/lsp-binaries/$rust_target"
done
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin buildPlugin --no-daemon
```

`buildPlugin` fails and lists every missing binary when a target is not staged. For local experiments, `-PversionlensAllowPartialTargets` bundles only the staged targets. The plugin archive is written under `packages/jetbrains-plugin/build/distributions/`. It is one archive for all platforms; no OS or architecture dependency is declared.

## Settings

The server asks the IDE for its settings with `workspace/configuration`. The plugin answers the `versionlens` section from a JSON object stored in `versionlens-redux.xml` in the IDE config directory (`settingsJson`, default `{}`), using the same keys as the VS Code `versionlens` settings (see `contributes.configuration` in [`../vscode-extension/package.json`](../vscode-extension/package.json)). There is no settings page yet. Example `settingsJson`:

```json
{
  "diagnostics": { "updateSeverity": "warn" }
}
```

`diagnostics.updateSeverity` sets the severity of update-available diagnostics: `error`, `warn`, `info` (default), `hint`, or `off`. `off` hides them; vulnerability diagnostics are unaffected.

## Private registries

The plugin does not implement the `versionlens/authorization` request, so the server uses its other credential sources:

- For `api.github.com`: `GITHUB_TOKEN`, then `GH_TOKEN`, then the output of `gh auth token`, which is only tried after GitHub answers 401, 403, or 429.
- Registry configuration files such as `.npmrc`, `.yarnrc.yml`, and `NuGet.config`.

Start the IDE with the variables set. Tokens are never written to logs, diagnostics, or the cache.

## Publishing

The release workflow builds the archive once. `publish-marketplaces.yml` downloads it from the GitHub release and runs `publishPlugin -x buildPlugin -PversionlensReleasePlugin=<zip>`, which signs that archive and uploads it without rebuilding. Run `bun run marketplaces:configure` once to store `JB_CERTIFICATE_CHAIN`, `JB_PRIVATE_KEY`, `JB_PRIVATE_KEY_PASSWORD`, and `JETBRAINS_MARKETPLACE_TOKEN` in the approval-gated GitHub environment. JetBrains requires the first plugin version to be uploaded manually before later versions can use automated publishing.

## Development checks

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin test --no-daemon
cargo test -p versionlens-lsp
```

Run `bun run check` from the repository root before committing broad changes.

## License

[ISC](../../LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. JetBrains support uses the fork's shared Rust LSP server and the shared assets in [`../../assets/versionlens`](../../assets/versionlens) when editor media is needed.
