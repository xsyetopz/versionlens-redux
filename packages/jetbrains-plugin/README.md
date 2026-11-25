# VersionLens Redux for JetBrains IDEs

The plugin starts the shared `versionlens-lsp` server through the IntelliJ Platform LSP API, so IntelliJ IDEA and the other IDEs that ship that API show the same code lenses, inlay hints, hovers, diagnostics, and quick fixes as VS Code.

- Plugin id: `com.versionlens.jetbrains`
- Requires: an IDE build 261 (2026.1) or newer with the `com.intellij.modules.lsp` module
- Build file: [`build.gradle.kts`](build.gradle.kts), part of the root Gradle build

## Files

The plugin runs `versionlens-lsp --file-types` once per server path and starts the server for every file whose path matches one of the globs it prints. A file type added to the server reaches the IDE without a plugin change.

## Server

The plugin uses the first server it finds:

1. The `VERSIONLENS_LSP` environment variable.
2. **Server path** on the settings page.
3. The bundled `bin/<rust-target>/versionlens-lsp[.exe]` for the running OS and architecture (`*-musl` on Alpine).
4. `versionlens-lsp` on `PATH`.

When none runs, the plugin shows one notification and writes the reason to the log.

## Settings

**Settings | Tools | VersionLens** holds the server path and the `versionlens` section the server reads, with the same keys as VS Code:

| Setting | Default |
| --- | --- |
| Enabled, Code lenses, Inlay hints | on |
| Suggest prereleases | off |
| Check vulnerabilities with OSV | on |
| Verify TLS certificates | on |
| Request timeout (ms) | 10000 |
| Concurrent requests | 32 |
| Cache (minutes) | 60 |
| GitHub token | empty; stored in the IDE password safe, falls back to `GITHUB_TOKEN`, then `GH_TOKEN` |
| Summary lens above each manifest | off |
| Outdated dependency severity | info; `off`, `hint`, `info`, `warning`, or `error` |
| Prerelease tags | empty; comma-separated tags such as `rc, beta`, empty allows all |
| File types | empty; comma-separated file type or registry ids, empty reads all |
| Workspace exclude | empty; comma-separated globs a workspace update skips |
| Registries | `registry id=URL template` lines, such as `npm=https://npm.example.com/{enc}` |
| Per registry | `registry id.setting=value` lines overriding `cacheMinutes`, `strictSsl`, or `prereleaseTags` for one registry, such as `npm.cacheMinutes=5` |
| Authorization | `URL prefix=header value` lines; stored in the IDE password safe |
| Install commands | `file type or registry id=shell command` lines, run when a save changes the dependencies |
| Extra files | `glob=file type id` lines |
| Indicators | `state=glyph` lines for `latest`, `update`, `checking`, `failed`, `unreadable`, `vulnerable`, `newer`, `satisfies`, `noMatch`, `directory`, `build`, `yanked`, `deprecated` |

Applying the page sends the new settings to every running server. Changing the server path restarts it.

## Toolbar and Actions

Each manifest the server reads gets an editor toolbar, as in the VS Code editor title bar:

- The versions toggle: blue when versions are shown, grey when hidden, orange with spinning arrows while lookups run, red when lookups failed (a click opens the log).
- The prerelease toggle.
- **More**: update all dependencies or the whole workspace to latest, major, minor, or patch; sort dependencies; run the install command; refresh; clear the cache; show the log.

**Tools | VersionLens** holds the same actions plus **Restart Server**, **Add URL Authentication**, and **Remove URL Authentication**, and each one is also in Find Action. The two authentication actions edit the Authorization lines of the settings page, kept in the IDE password safe, and send them to the running servers.

## Build

Run these from the repository root. `just jvm::gradle` finds a JDK that can start Gradle, and Gradle downloads the compiler JDK itself.

```bash
just jvm::verify                                    # format check, compile, lint, tests
cargo build -p versionlens-lsp --release
mkdir -p build/lsp/aarch64-apple-darwin && cp target/release/versionlens-lsp build/lsp/aarch64-apple-darwin/
just jvm::gradle :jetbrains-plugin:buildPlugin -PversionlensBinariesDir=build/lsp
```

`-PversionlensBinariesDir` names a directory with one `<rust-target>/versionlens-lsp[.exe]` per target. Release CI stages every target in [`scripts/package/targets.json`](../../scripts/package/targets.json) there. The zip is written to `packages/jetbrains-plugin/build/distributions/versionlens-redux-jetbrains-plugin-<version>.zip`.

## Publishing

`publish-marketplaces.yml` downloads the zip from the GitHub release and runs `./gradlew :jetbrains-plugin:publishPlugin -x buildPlugin -PversionlensReleasePlugin=<zip>`, which signs that zip and uploads it without rebuilding. `bun run marketplaces:configure` stores `JB_CERTIFICATE_CHAIN`, `JB_PRIVATE_KEY`, `JB_PRIVATE_KEY_PASSWORD`, and `JETBRAINS_MARKETPLACE_TOKEN` in the approval-gated GitHub environment.

## License

[ISC](../../LICENSE). VersionLens Redux is a fork of the original VersionLens extension by Peter Flannery and contributors.
