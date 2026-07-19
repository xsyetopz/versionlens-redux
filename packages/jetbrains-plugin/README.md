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
3. Bundled release binary extracted to the IDE system directory.
4. Repository-local debug binary at `target/debug/versionlens-lsp`.
5. `versionlens-lsp` on `PATH`.

## Build the plugin

```bash
packages/jetbrains-plugin/gradlew -p packages/jetbrains-plugin buildPlugin --no-daemon
```

The built plugin artifact is written under `packages/jetbrains-plugin/build/distributions/`.

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
