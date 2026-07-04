# VersionLens for Visual Studio Code

VersionLens shows package version information directly in VS Code dependency manifests, using a Rust core for parsing, version comparison, and registry lookups.

[![CI](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml/badge.svg)](https://github.com/xsyetopz/vscode-versionlens/actions/workflows/ci.yml)
[![GitHub release](https://img.shields.io/github/v/release/xsyetopz/vscode-versionlens?include_prereleases)](https://github.com/xsyetopz/vscode-versionlens/releases)
[![License: ISC](https://img.shields.io/badge/license-ISC-blue.svg)](LICENSE)

![Show releases](images/faq/show-releases.gif)

## Use it when

- you want CodeLens version hints while editing dependency files;
- you want to see whether a dependency is current, outdated, fixed, or constrained;
- you want prerelease versions available on demand;
- you want vulnerability warnings from OSV.dev surfaced in the editor.

## Supported languages and ecosystems

VersionLens supports these manifest ecosystems in this repository:

| Ecosystem | Registry/source |
| --- | --- |
| Cargo / Rust | crates.io |
| Composer / PHP | Packagist |
| Deno | Deno, JSR, npm |
| Docker | Docker Hub / Microsoft Container Registry |
| .NET | NuGet |
| Dub / D | code.dlang.org |
| Go | proxy.golang.org |
| Maven / Java | Maven Central |
| npm / Bun / pnpm / JSPM | npm-compatible registries |
| Pub / Dart / Flutter | pub.dev |
| Python | PyPI |
| Ruby | RubyGems |

TOML files require a VS Code extension that registers the TOML language, such as Even Better TOML.

## Show version information

Open a supported manifest and select the **V** icon in the editor toolbar.

You can also use the editor `...` menu item **Show release versions**, or set `versionlens.suggestions.showOnStartup` in VS Code settings.

## Show prerelease versions

Select the **tag** icon in the editor toolbar, or use **Show prerelease versions** from the editor `...` menu.

You can also set `versionlens.suggestions.showPrereleasesOnStartup`.

![Show prereleases](images/faq/show-prereleases.gif)

## Vulnerability checks

VersionLens integrates with OSV.dev to highlight vulnerable packages in manifest files.

- **Editor diagnostics:** vulnerable versions are marked in the editor.
- **Update safeguards:** updates to known vulnerable versions require confirmation.
- **Visual indicators:** updatable versions with known vulnerabilities are marked in the CodeLens text.

This feature is controlled by `versionlens.suggestions.showVulnerabilities`.

## Custom install task

VersionLens can run a custom install task when you save a package document. Configure it with the `versionlens.customInstallCommand` and related `versionlens.showCustomInstall` settings in VS Code.

## Install

Download a VSIX from the [GitHub releases page](https://github.com/xsyetopz/vscode-versionlens/releases), then install it from VS Code:

1. Open the Extensions view.
2. Select `...`.
3. Select **Install from VSIX...**.
4. Choose the downloaded `.vsix` file.

## Troubleshooting

- **No CodeLens:** ensure `"editor.codeLens": true` is set.
- **Toolbar icons missing:** ensure `"workbench.editor.editorActionsLocation": "hidden"` is not set.
- **Stale results:** run **VersionLens: Clear cache** from the Command Palette.
- **Cache duration:** configure `versionlens.cacheTtlSeconds`.
- **Logs:** set log level to `debug` through **Developer: Set Log Level** or the `VersionLens` output channel.

![Extension host log](images/faq/ext-host-log.jpg)

![Extension log](images/faq/ext-log.jpg)

If logs do not explain the issue, check VS Code Developer Tools with **Help > Toggle Developer Tools**.

## Repository

Source, issues, releases, contributor instructions, and CI live at <https://github.com/xsyetopz/vscode-versionlens>.

## License

ISC. See [LICENSE](LICENSE).
