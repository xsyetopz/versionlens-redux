# VersionLens Redux for Zed

The Zed extension starts the shared `versionlens-lsp` server so Zed can show VersionLens Redux dependency diagnostics and code lenses.

## Package

- Extension id: `versionlens-lsp`
- Display name: `VersionLens Redux`
- Manifest: [`extension.toml`](extension.toml)
- Runtime server: `crates/versionlens-lsp`

## Build the language server

From the repository root:

```bash
cargo build -p versionlens-lsp
```

Direct-distribution packages are platform-specific and embed the matching release server binary. CI produces Linux x64/ARM64, macOS x64/ARM64, and Windows x64/ARM64 archives. The registry extension resolves the server in this order:

1. `lsp.versionlens.binary.path` in Zed settings.
2. `versionlens-lsp` on `PATH`.
3. The matching archive from the GitHub release whose tag equals the extension version.

The direct archive carries the native server selected by its platform and architecture arguments. The registry package itself does not bundle the server; Zed grants it narrowly scoped permission to download only VersionLens Redux GitHub release assets.

## Zed verification

```bash
cargo check --manifest-path packages/zed-extension/Cargo.toml --locked
cargo test --manifest-path packages/zed-extension/Cargo.toml --locked
cargo build --manifest-path packages/zed-extension/Cargo.toml --release --locked
```

The repository-wide integration gate is `bun run check`.

The Zed extension code uses the registry-supported [MIT license](LICENSE). The language server and the rest of the repository remain under [ISC](../../LICENSE).

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. Zed support uses the fork's shared Rust LSP server and the shared assets in [`../../assets/versionlens`](../../assets/versionlens) when editor media is needed.
