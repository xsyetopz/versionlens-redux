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

# Drive the extension in a real VS Code against the live registries; needs Docker and the network.
vscode-live: (vscode-container "scripts/vscode-live.mjs")

# Time cold starts of the original 1.x extension against this one; needs Docker and the network.
vscode-bench: (vscode-container "scripts/vscode-bench.mjs")

# Run a VS Code script in a Linux container under Xvfb: VS Code on macOS brings its window in
# front whatever its launcher asks, and the virtual display keeps it off the desktop. Named
# volumes keep the Rust toolchain, the crates, and the Linux build between runs.
[private]
vscode-container script:
    bun run build
    docker build -q -t versionlens-vscode scripts/vscode-container
    docker run --rm --init -v "{{justfile_directory()}}:/work" -w /work \
        -v versionlens-rustup:/usr/local/rustup -v versionlens-cargo:/usr/local/cargo/registry \
        -v versionlens-target:/target -e CARGO_TARGET_DIR=/target -e VERSIONLENS_BENCH_RUNS \
        -e VERSIONLENS_LSP=/target/release/versionlens-lsp versionlens-vscode \
        sh -c 'cargo build -q --release -p versionlens-lsp && xvfb-run -a bun {{script}}'

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
