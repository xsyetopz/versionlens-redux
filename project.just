# VersionLens recipes. The shared justfile imports this file.

# Show the remotes and the branch each local branch tracks.
remotes:
    git remote -v
    git branch -vv

# Keep upstream fetch-only.
safe-remotes:
    git remote set-url --push upstream DISABLED
    git remote -v

# Package the VS Code extension.
package:
    bun run package

# Check the release scripts, the extension build and tests, the Neovim plugin, and the editor packages.
project-verify:
    bun run test:release
    bun run check:extension-build
    bun run test:extension
    bun run package:editors
    make -C packages/neovim-plugin test
