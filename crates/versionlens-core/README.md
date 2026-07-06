# versionlens-core

The editor-neutral domain layer that turns document text, settings, registry providers, suggestions, diagnostics, and text edits into VersionLens Redux results.

## Role

Core orchestration for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-core
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
