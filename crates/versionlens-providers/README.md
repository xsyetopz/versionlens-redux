# versionlens-providers

Provider selection and registry request construction for supported ecosystems such as crates.io, npm-compatible registries, NuGet, Maven, Docker/OCI, and others.

## Role

Registry providers for the Rust workspace in VersionLens Redux. This crate is part of the repository implementation and is not documented as an independent public API.

## Validation

```bash
cargo test -p versionlens-providers
```

Run `bun run check` from the repository root before merging broad workspace changes.

## License

[ISC](../../LICENSE)
