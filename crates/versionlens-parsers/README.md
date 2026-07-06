# versionlens-parsers

Parsers for dependency manifests and editor documents. The crate owns file-specific extraction, source ranges, dependency identity, and editable version spans.

## Role

Manifest parsers for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-parsers
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
