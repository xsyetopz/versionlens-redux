# VersionLens recipes. The shared justfile imports this file.

# Show the remotes and the branch each local branch tracks.
remotes:
    git remote -v
    git branch -vv

# Keep upstream fetch-only.
safe-remotes:
    git remote set-url --push upstream DISABLED
    git remote -v

# Run one versionlens-fetch HTTP test by its full name, with its output.
http-test name:
    cargo test -p versionlens-fetch --locked {{ name }} -- --exact --nocapture

# Run the large registry response test.
http-large-response:
    just http-test client::send::tests::reads_large_registry_response_bodies

# Time cold provider lookups against a release build of the server.
provider-cold-benchmark *args:
    cargo build -p versionlens-lsp --release --locked
    python3 scripts/bench/provider-cold.py {{ args }}

# Package the VS Code extension.
package:
    bun run package

# Check the editor file lists, release scripts, extension build and tests, and editor packages.
project-verify:
    bun run check:manifests
    bun run test:release
    bun run check:extension-build
    bun run test:extension
    bun run package:editors
