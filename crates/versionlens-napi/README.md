# versionlens-napi

The native Node-API boundary loaded by the VS Code extension. It maps TypeScript inputs to the Rust core and maps Rust outputs back into VS Code-facing data objects.

## Role

Node-API bridge for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-napi
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
