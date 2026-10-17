# Spec Delta

## Purpose

Defines how the `versionlens-lsp` server binary is built once per platform, verified, and delivered to every supported editor package.

## ADDED Requirements

### Requirement: One binary per platform target
Each release SHALL build `versionlens-lsp` exactly once for each supported target:
- linux-x64, linux-arm64, linux-armhf
- alpine-x64, alpine-arm64
- darwin-x64, darwin-arm64
- win32-x64, win32-arm64

Every editor package SHALL use these artifacts rather than building its own.

#### Scenario: Release build
- **WHEN** a release is produced
- **THEN** exactly 9 native `versionlens-lsp` builds run, and the VS Code, JetBrains, Neovim and Zed packages contain or reference byte-identical binaries for the targets they support

### Requirement: Published checksums
Each release SHALL publish every binary as `versionlens-lsp-<version>-<target>` (archive) together with a `SHA256SUMS` file covering all of them.

#### Scenario: Verify a download
- **WHEN** a user or plugin downloads a release binary
- **THEN** its SHA-256 matches the entry in that release's `SHA256SUMS`

### Requirement: VS Code platform packages
The VS Code extension SHALL be published as one platform-specific VSIX per target, each bundling its target's binary. It SHALL keep the `>=1.75.0` engine requirement.

#### Scenario: Install on Alpine
- **WHEN** the extension is installed on alpine-x64
- **THEN** the alpine-x64 VSIX is selected and the server starts without additional downloads

### Requirement: JetBrains and Zed use release artifacts
The JetBrains plugin SHALL bundle the prebuilt binary for each platform variant, using the release artifacts instead of compiling. The Zed extension SHALL download the raw `versionlens-lsp` release archive for its version, verify it against `SHA256SUMS`, and remove binaries left over from older versions.

#### Scenario: Zed first start
- **WHEN** the Zed extension starts with no cached binary
- **THEN** it downloads the server archive for the extension's version, verifies its checksum, and launches it

### Requirement: Neovim binary resolution and download
The Neovim plugin SHALL resolve the server in this order:
1. explicit `cmd`
2. `$VERSIONLENS_LSP`
3. a bundled binary
4. a previously downloaded binary
5. `PATH`

If none is found and `auto_download` is enabled (the default), it SHALL download the matching release binary into Neovim's data directory and verify it against `SHA256SUMS`.

#### Scenario: LuaRocks install
- **WHEN** the plugin is installed without a binary and no server is on `PATH`
- **THEN** the first start downloads and verifies the binary for the plugin's version and starts the server

#### Scenario: Checksum mismatch
- **WHEN** the downloaded binary's checksum does not match
- **THEN** the binary is deleted, the server is not started, and an error names the expected and actual checksums

#### Scenario: Download disabled
- **WHEN** `auto_download = false` and no binary is found
- **THEN** no network request is made and the plugin reports how to install the server

### Requirement: Synchronized versions
All editor packages and the server SHALL carry the same release version. Version 0.5.0 SHALL be the first release with a single shared server.

#### Scenario: Version mismatch check
- **WHEN** release packaging runs
- **THEN** it fails if any editor package version differs from the workspace crate version
