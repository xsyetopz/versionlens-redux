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

The packaged plugin builds and embeds a release version of the server. Release artifacts are platform-specific for Linux x64/ARM64, macOS x64/ARM64, and Windows x64/ARM64. The plugin resolves the server in this order:

1. `versionlens.lsp.path` Java system property.
2. `VERSIONLENS_LSP` environment variable.
3. Packaged `bin/versionlens-lsp` beside the installed plugin libraries.
4. A legacy JAR-embedded binary extracted to the IDE system directory.
5. Repository-local debug binary at `target/debug/versionlens-lsp`.
6. `versionlens-lsp` on `PATH`.

## Build the plugin

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin buildPlugin --no-daemon
```

The built plugin artifact is written under `packages/jetbrains-plugin/build/distributions/`.

The Gradle build embeds the platform-specific native LSP launcher selected by `versionlensRustTarget`.

## Native variant archives

IntelliJ Platform 2026.1 (`sinceBuild` `261`) selects native compatibility through explicit
OS/architecture module dependencies. Each native build is a separate Marketplace-selectable
plugin variant; publishing is intentionally not performed by this project. The archive keeps
`bin/<executable>` beside `lib/`, rather than embedding the server in the plugin JAR.

The six supported target triples are:

```text
linux-x86_64    linux-arm64    mac-x86_64
mac-arm64       windows-x86_64 windows-arm64
```

Build one target, for example the current macOS ARM64 host. `buildPlugin` derives the variant
from the host when the property is omitted:

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin \
  buildPlugin --no-daemon
# packages/jetbrains-plugin/build/distributions/
#   versionlens-jetbrains-plugin-0.4.0-mac-arm64.zip
```

On the corresponding native runner, provide the Rust target triple explicitly:

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin \
  -PversionlensRustTarget=x86_64-unknown-linux-gnu buildPlugin --no-daemon
```

The output is `build/distributions/versionlens-jetbrains-plugin-0.4.0-<os>-<arch>.zip`.
Run `buildPlugin` once per target to produce all six archives:

```text
x86_64-unknown-linux-gnu  -> ...-linux-x86_64.zip
aarch64-unknown-linux-gnu -> ...-linux-arm64.zip
x86_64-apple-darwin       -> ...-mac-x86_64.zip
aarch64-apple-darwin     -> ...-mac-arm64.zip
x86_64-pc-windows-msvc    -> ...-windows-x86_64.zip
aarch64-pc-windows-msvc   -> ...-windows-arm64.zip
```

Marketplace signing and upload are configured but never run automatically. Supply
`JB_CERTIFICATE_CHAIN`, `JB_PRIVATE_KEY`, and `JB_PRIVATE_KEY_PASSWORD` to sign a variant,
or `JETBRAINS_MARKETPLACE_TOKEN` to publish one explicitly from its matching native runner.

## Development checks

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin buildPlugin --no-daemon
cargo test -p versionlens-lsp
```

Run `bun run check` from the repository root before committing broad changes.

## License

[ISC](../../LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. JetBrains support uses the fork's shared Rust LSP server and the shared assets in [`../../assets/versionlens`](../../assets/versionlens) when editor media is needed.
