# 0.4.4

- Made cold dependency checks substantially faster across providers with concurrent reusable HTTP connections, shared bodies, coalesced work, a compressed CRAN catalog, shared LuaRocks catalog requests, and smaller official npm responses while preserving complete release-history semantics.

# 0.1.0

## VersionLens Redux

- Rebranded the VS Code extension as `versionlens-redux` under publisher `xsyetopz`.
- Added conflict detection for the original `pflannery.vscode-versionlens` extension.
- Kept attribution to the original VersionLens creators while tracking this fork under the Redux identity.
- Expanded manifest support and Rust-backed analysis for additional dependency ecosystems, including broader C/C++ and JVM build file coverage.
