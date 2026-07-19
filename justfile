set shell := ["zsh", "-cu"]

export PATH := env("HOME") + "/.bun/bin:" + env("PATH")

default:
    @just --list

# Show git remotes and branch tracking.
remotes:
    git remote -v
    git branch -vv

# Make the external upstream remote fetch-only.
safe-remotes:
    git remote set-url --push upstream DISABLED
    git remote -v

# Run repository text/style checks that are cheap locally.
check-style:
    git diff --check
    bun run check:biome
    cargo fmt --all -- --check

# Run TypeScript checks.
typecheck:
    bun run typecheck

# Run Rust tests with an isolated target dir to avoid local target-path hangs.
rust-test *args:
    CARGO_TARGET_DIR=/tmp/versionlens-cargo-target cargo test --workspace --locked {{ args }}

# Run one exact versionlens-http test with output.
http-test name:
    CARGO_TARGET_DIR=/tmp/versionlens-cargo-target cargo test -p versionlens-http --locked {{ name }} -- --exact --nocapture

# Regression for large registry response bodies.
http-large-response:
    just http-test client::send::tests::reads_large_registry_response_bodies

# Build the local VSIX package.
package:
    bun run package

# Run the full repository validation. This can be slow.
check:
    bun run check
