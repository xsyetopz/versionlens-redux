# Releasing and Marketplace publication

VersionLens Redux separates GitHub release creation from external Marketplace publication. Both workflows are manually dispatched from trusted repository state; neither receives publishing credentials on pull requests.

## 1. Prepare the version

Update every version-bearing manifest with the repository command, then update `CHANGELOG.md`:

```bash
bun run version:bump -- 0.4.1
bun run test:release
```

The version command updates the Rust workspace and lockfile, Bun workspace and lockfile, VS Code package, Zed crate and manifest, JetBrains Gradle build, and Neovim runtime version together.

## 2. Configure GitHub environment secrets

Authenticate GitHub CLI with an account that can administer Actions secrets for this repository, then run:

```bash
bun run marketplaces:configure
```

The command creates or reuses the `marketplaces` GitHub environment, prompts for every required value, and sends values directly to GitHub CLI. It does not create a local `.env` file or print secret values.

It configures:

| Destination | GitHub environment secrets |
| --- | --- |
| VS Code Marketplace | `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_SUBSCRIPTION_ID` |
| JetBrains Marketplace | `JETBRAINS_MARKETPLACE_TOKEN`, `JB_CERTIFICATE_CHAIN`, `JB_PRIVATE_KEY`, `JB_PRIVATE_KEY_PASSWORD` |
| Zed extension registry | `ZED_EXTENSIONS_FORK`, `ZED_EXTENSIONS_TOKEN` |
| Neovim / LuaRocks | `LUAROCKS_API_KEY` |

Configure required reviewers on the `marketplaces` GitHub environment before publication.

### One-time provider setup

- **VS Code:** create a Microsoft Entra workload identity trusted by this repository environment, add it to the `xsyetopz` Marketplace publisher, and configure the GitHub OIDC federated subject for the `marketplaces` environment.
- **JetBrains:** create the Marketplace plugin and upload its first version manually, create a permanent token, and generate the signing key and certificate chain.
- **Zed:** fork `zed-industries/extensions` to the account recorded by `ZED_EXTENSIONS_FORK`. The token must be able to push branches to that fork and create a pull request against the public upstream registry.
- **LuaRocks:** create an account and API key. Neovim itself recommends LuaRocks for publishing versioned Lua plugins.

## 3. Create the GitHub release

Wait for successful `master` runs of all four editor workflows for the release commit. Dispatch `.github/workflows/release.yml` with the exact repository version. It verifies the current remote `master`, downloads the successful editor artifacts for that commit, creates `v<version>`, and attaches the complete package set to the GitHub release.

## 4. Publish externally

Dispatch `.github/workflows/publish-marketplaces.yml` with the released version. Each destination can be selected independently:

- VS Code publishes the nine released platform VSIX files with Microsoft Entra authentication.
- JetBrains rebuilds, signs, and publishes six Marketplace-selectable native variants sequentially on their matching runners.
- Zed creates or updates a single-extension pull request in `zed-industries/extensions`; registry publication occurs after Zed maintainers merge it.
- Neovim publishes the Lua adapter to LuaRocks. GitHub release archives remain the installation route that bundles a native language server.

Re-running a destination that rejects duplicate versions is expected to fail rather than silently overwrite an existing Marketplace release.
