# Contributing

Thanks for working on VersionLens.

## Before Changing Code

1. Read `README.md` for product scope.
2. Read `AGENTS.md` for repository-specific build, test, and safety rules.
3. Inspect the affected Rust crates, TypeScript adapter code, scripts, and tests before editing.

## Development Setup

Install Bun 1.4.3 and the Rust toolchain declared in `rust-toolchain.toml`, then install dependencies:

```bash
bun install --frozen-lockfile
```

Build the VS Code extension bundle:

```bash
bun run build
```

Package a local VSIX:

```bash
bun run package
```

Build all editor packages and verify their bundled runtimes:

```bash
bun run package:editors
```

## Adding a File Type

Add one `row` to `FILE_TYPES` in `crates/versionlens-lsp/src/catalog/rows.rs`, and a `REGISTRIES` row in `crates/versionlens-lsp/src/registry.rs` when the registry is new. Editors read the globs from `versionlens-lsp --file-types` or from `capabilities.experimental.fileTypes` in the initialize result, so their packages stay unchanged; Zed needs the file's language in `packages/zed-extension/extension.toml`.

## Release

1. Bump and commit the synchronized repository version:

```bash
bun run version:bump 0.2.0
```

1. Push the commit to `master` and wait for the VS Code, Zed, and JetBrains workflows to pass.
2. Run the **Release** workflow from `master` with the same version.

The release workflow refuses stale commits, mismatched versions, existing tags, and commits without successful editor workflows. It downloads the compiled artifacts from those exact workflow runs before creating the tag and GitHub release.

## Validation

Run the smallest check that covers your change. Common checks:

```bash
cargo test -p versionlens-lsp
bun test packages/vscode-extension
just jvm::verify
make -C packages/neovim-plugin test
```

Before a release or broad refactor, run:

```bash
just verify
```

`just verify` is broad and slow: it builds every editor package. Do not run it in the background unless someone explicitly asked for that.

`just vscode-live` builds the extension and drives it in an isolated VS Code against the live registries and OSV: hints, lenses, quick fixes, hovers, diagnostics, settings, every palette command, the original 1.x command ids and settings, restart, and the Output log.

`just vscode-bench` installs the original 1.x extension from the Marketplace. It then times cold starts of both extensions on the same `requirements.txt`, in alternating fresh instances, from opening the file until every dependency line has a resolved version lens.

Both run VS Code for Linux in a Docker container (`scripts/vscode-container`), under Xvfb. VS Code on macOS brings its window to the front however it is launched, and the virtual display keeps it off the desktop. They need Docker and the network, so `just verify` leaves them out.

`just jetbrains-live` builds the server and runs `LiveTest` in a headless IntelliJ IDEA, using the plugin's own client against the live registries and OSV: hints, diagnostics, the update intention, hover, settings, every action, and restart. It needs the network and no desktop. Other test runs skip `LiveTest`.

`just jetbrains-gui` drives the plugin's GUI in IntelliJ IDEA for Linux inside a Docker container (`scripts/jetbrains-gui`), under Xvfb. It opens a `package.json` while a proxy that never answers keeps every lookup pending. It then runs Add URL Authentication from Find Action, types a URL, a user, and a password into its dialogs, and runs Remove URL Authentication, picking the URL in its URLs to Forget dialog. The IDE stores passwords in a KeePass file, because the container has no native keychain. The run turns the server trace on and ends with the VersionLens tool window open on the traced traffic. A wrapper records what the IDE sends the server. It fails when the server never starts, when the editor toolbar spinner stays still, when the server never receives the added `Basic` header or its removal, or when the IDE blames VersionLens for an error or freeze. Screenshots and `idea.log` go to `target/jetbrains-gui`. A Docker volume keeps the IDE between runs, and a new `intellijIdea` version in `build.gradle.kts` downloads it again. The recipe needs Docker and the network.

`just zed-gui` drives the Zed extension in Zed for Linux inside a Docker container (`scripts/zed-gui`), under Xvfb with Mesa's software Vulkan. It installs the built `extension.wasm`, turns on code lenses and inlay hints, sets `install.npm` so the install command has a task to run, and opens a `package.json`, a `Cargo.toml`, and a `compose.yaml` against the live registries. Each file carries a status the run waits for: deprecated `request 2.88.2`, a local directory, a missing directory, yanked `rand 0.7.1`, and `node:20-alpine` with its `change build` lens. The run then moves the lenses into the code actions menu. It chooses `change build` there with `ctrl-.`, clicks the build in the prompt that follows, and waits for the edit that writes it back. Next, it runs every VersionLens command from the menu: clearCache, refresh, showPrereleases, hidePrereleases, hide, show, sort, install, updateAll, and updateWorkspace. Zed fetches a document's code actions again only when its cursor moves, so the run moves the cursor one character before each pick. It then runs one command from the menu that the palette's `editor: toggle code actions` opens, and one from the menu of the toolbar's ⚡ Code Actions button. For VS Code's `versionlens.trace.server`, it opens `dev: open language server logs`, picks the `versionlens` server and RPC Messages, moves the cursor in `compose.yaml`, and copies the view's text with `xclip` into `rpc-messages.txt`, which must hold the `textDocument/codeAction` request and its answer. Last, it chooses Add URL authentication, clicks Header value in its prompt, types a value into the stored file the server opens, saves it, and waits for the server to reread it; then it chooses Remove URL authentication, clicks the stored URL, and checks the file is empty. A wrapper records the server traffic. The run fails when the server never starts, when a status never shows, when the menu lacks a command, when the RPC Messages view lacks the traced request, when a choice never reaches the server, when the build prompt or its edit never comes, when the stored authentication file never holds the typed value or is not emptied, or when the server answers a command with an error or Zed logs a VersionLens error. Screenshots, the traffic, and `Zed.log` go to `target/zed-gui`. A Docker volume keeps Zed between runs, and `ZED_REFRESH=1` downloads the latest stable release again. The recipe needs Docker and the network.

`just neovim-live` builds the server and runs `packages/neovim-plugin/tests/live.lua` in a headless Neovim against the live registries and OSV: hints, advisories, hover, the lens that opens the package page, the update code action, every `:VersionLens` subcommand, settings pushed by `setup()`, restart, and `libraries.hxml`. It needs the network, so `just verify` runs only the offline `tests/run.lua`.

## Pull Requests

A good pull request includes:

- a narrow description of the behavior changed;
- the files or subsystems touched;
- validation commands run and their results;
- any behavior left unverified.

Use Conventional Commits for commit messages, for example:

```text
fix(extension): stop refresh loop after toggling lenses
```

If OpenAI Codex CLI materially contributed, include:

```text
Co-authored-by: Codex <noreply@openai.com>
```

## Code Review Expectations

- Keep unrelated formatting and cleanup out of behavior changes.
- Preserve command IDs, settings keys, and extension contribution points unless the change requires a migration.
- Prefer source-backed parity with upstream VersionLens behavior when changing user-visible behavior.
- Do not commit generated VSIX files, `target/`, `dist/`, `node_modules/`, staged server binaries, or TypeScript build info.
