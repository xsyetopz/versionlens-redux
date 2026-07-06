# versionlens-edits

Edit construction for dependency updates, including version replacement and dependency sorting where a manifest format supports safe ordering.

## Role

Text edits for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-edits
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
