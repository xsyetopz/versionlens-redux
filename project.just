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

# Drive the extension in a real VS Code window against the live registries; needs a desktop and the network.
vscode-live: package
    bun scripts/vscode-live.mjs

# Drive the JetBrains plugin in a headless IntelliJ IDEA against the live registries; needs the network.
jetbrains-live:
    cargo build -p versionlens-lsp
    just jvm::gradle :jetbrains-plugin:test --tests com.versionlens.jetbrains.LiveTest -PversionlensLiveServer={{justfile_directory()}}/target/debug/versionlens-lsp --rerun

# Drive the Neovim plugin in a headless Neovim against the live registries; needs the network.
neovim-live:
    make -C packages/neovim-plugin live

# Check the release scripts, the extension build and tests, the Neovim plugin, and the editor packages.
project-verify:
    bun run test:release
    bun run check:extension-build
    bun run test:extension
    bun run package:editors
    make -C packages/neovim-plugin test
