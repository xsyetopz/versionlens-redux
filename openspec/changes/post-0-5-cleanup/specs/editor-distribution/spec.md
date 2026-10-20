## ADDED Requirements

### Requirement: JetBrains uses release artifacts
The JetBrains plugin SHALL be one plugin that bundles the prebuilt binary of every target in `targets.json` under `bin/<target>/`, using the release artifacts instead of compiling, and SHALL pick the binary for the running OS and architecture at start. Publishing SHALL upload the released plugin archive instead of rebuilding it.

#### Scenario: JetBrains plugin build
- **WHEN** the plugin is built with every target's binary staged
- **THEN** the archive contains `bin/<target>/versionlens-lsp[.exe]` for all targets, and the build fails naming any missing target

### Requirement: Zed uses release artifacts
The Zed extension SHALL download the raw `versionlens-lsp` release archive for its version, verify it against `SHA256SUMS`, and remove binaries left over from older versions.

#### Scenario: Zed first start
- **WHEN** the Zed extension starts with no cached binary
- **THEN** it downloads the server archive for the extension's version, verifies its checksum, and launches it

## REMOVED Requirements

### Requirement: JetBrains and Zed use release artifacts
**Reason**: The requirement text is over 500 characters and fails `openspec validate --strict`. It states two behaviors, one per editor.
**Migration**: Replaced by "JetBrains uses release artifacts" and "Zed uses release artifacts". Wording and scenarios are unchanged.
