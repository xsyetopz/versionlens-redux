set shell := ["zsh", "-cu"]

export PATH := env("HOME") + "/.bun/bin:" + env("PATH")

default:
    @just --list

remotes:
    git remote -v
    git branch -vv

safe-remotes:
    git remote set-url --push upstream DISABLED
    git remote -v

check-style:
    git diff --check
    bun run check:biome
    cargo fmt --all -- --check

typecheck:
    bun run typecheck

rust-test *args:
    CARGO_TARGET_DIR=/tmp/versionlens-cargo-target cargo test --workspace --locked {{ args }}

http-test name:
    CARGO_TARGET_DIR=/tmp/versionlens-cargo-target cargo test -p versionlens-http --locked {{ name }} -- --exact --nocapture

http-large-response:
    just http-test client::send::tests::reads_large_registry_response_bodies

package:
    bun run package

check:
    bun run check
