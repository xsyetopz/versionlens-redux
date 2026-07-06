# versionlens-vscode-model

Shared serializable data structures for ranges, diagnostics, code lenses, edits, and payloads exchanged across editor adapters.

## Role

Serializable editor model for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-vscode-model
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
