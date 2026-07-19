# VersionLens Redux for Zed

The Zed extension starts the shared `versionlens-lsp` server so Zed can show VersionLens Redux dependency diagnostics and code lenses.

## Package

- Extension id: `versionlens`
- Display name: `VersionLens Redux`
- Manifest: [`extension.toml`](extension.toml)
- Runtime server: `crates/versionlens-lsp`

## Build the language server

From the repository root:

```bash
cargo build -p versionlens-lsp
```

Direct-distribution packages are platform-specific and embed the matching release server binary. CI produces Linux x64/ARM64, macOS x64/ARM64, and Windows x64/ARM64 archives. The extension code resolves the server in this order:

1. `lsp.versionlens.binary.path` in Zed settings.
2. Bundled `bin/versionlens-lsp`.
3. `versionlens-lsp` on `PATH`.
4. Repository-local debug binary at `target/debug/versionlens-lsp`.

## Development checks

```bash
cargo check --manifest-path packages/zed-extension/Cargo.toml
cargo test -p versionlens-lsp
```

Run `bun run check` from the repository root before committing broad changes.

## License

[ISC](../../LICENSE)

## Attribution

VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors. Zed support uses the fork's shared Rust LSP server and the shared assets in [`../../assets/versionlens`](../../assets/versionlens) when editor media is needed.
