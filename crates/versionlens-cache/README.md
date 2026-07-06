# versionlens-cache

Shared cache keys and in-memory cache helpers used by the VersionLens Redux core when registry and document work needs bounded reuse.

## Role

Cache primitives for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-cache
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
