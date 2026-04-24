# VersionLens for Visual Studio Code

[![Version](https://img.shields.io/visual-studio-marketplace/v/pflannery.vscode-versionlens?style=flat-square&color=blue)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens)
[![Installs](https://img.shields.io/visual-studio-marketplace/i/pflannery.vscode-versionlens?style=flat-square&color=blue)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens)
[![Rating](https://img.shields.io/visual-studio-marketplace/r/pflannery.vscode-versionlens?style=flat-square&color=blue)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens)
[![The ISC license](https://img.shields.io/badge/license-ISC-orange.png?style=flat-square&color=blue)](http://opensource.org/licenses/ISC)

This project is `active`, not sponsored or funded.

[![BuyMeACoffee](https://www.buymeacoffee.com/assets/img/custom_images/purple_img.png)](https://www.buymeacoffee.com/peterf)

VersionLens shows __version__ information when opening a package or project file in VS Code. It abides by [semver rules](https://semver.org/) and uses the [Node Semver](https://github.com/npm/node-semver) package to compare and sort versions.

![Show releases](images/faq/show-releases.gif)

## Contents

- [Supported Languages & Ecosystems](#supported-languages--ecosystems)
- [How do I see version information for a package?](#how-do-i-see-version-information-for-a-package)
- [Pre-release versions](#can-i-see-pre-release-versions)
- [Can I check for package vulnerabilities?](#can-i-check-for-package-vulnerabilities)
- [What do the suggestion links mean?](./docs/suggestion-overview.md)
- [Will this extension install packages for me?](#will-this-extension-install-packages-for-me)
- [How do I authorize packages that need credentials?](./docs/authorization.md)
- [How do I install this extension?](#how-do-i-install-this-extension)
- [How do I troubleshoot this extension?](#how-do-i-troubleshoot-this-extension)
- [Commands & Settings](./docs/commands-and-settings.md)
- [Development Guide](./docs/development.md)

### Supported Languages & Ecosystems

VersionLens supports a wide range of ecosystems:

- **Cargo (Rust):** [crates.io](https://doc.rust-lang.org/cargo/)
- **Composer (PHP):** [packagist.org](https://getcomposer.org/)
- **Deno (JSR/NPM):** [deno.com](https://deno.com/)
- **Docker:** [Docker Hub / MCR](https://www.docker.com/)
- **Dotnet:** [NuGet](https://www.dotnetfoundation.org/)
- **Dub (D):** [code.dlang.org](https://code.dlang.org/)
- **Go:** [proxy.golang.org](https://go.dev/)
- **Maven (Java):** [Maven Central](https://maven.apache.org/)
- **NPM (Node):** [npmjs.com](https://www.npmjs.com/), JSPM, PNPM
- **Pub (Dart/Flutter):** [pub.dev](https://pub.dev/)
- **Python:** [PyPI](https://pypi.org/)
- **Ruby:** [rubygems.org](https://rubygems.org/)

> **Note:** TOML files require an extension that registers the TOML language with VS Code (e.g., [Even Better TOML](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)).

## How do I see version information for a package?

Select the **V** icon in the package/project file toolbar. 

> You can also find this in the editor `...` (secondary) menu as **Show release versions**.

You can also set the default startup state using `versionlens.suggestions.showOnStartup` in your settings.

## Can I see pre-release versions?

Yes, select the **tag** icon in the package/project file toolbar.

> You can also find this in the editor `...` (secondary) menu as **Show prerelease versions**.

You can also set the default startup state using `versionlens.suggestions.showPrereleasesOnStartup`.

![Show prereleases](images/faq/show-prereleases.gif)

## Can I check for package vulnerabilities?

VersionLens integrates with [OSV.dev](https://osv.dev) to highlight vulnerable packages directly in your manifest files. 

*   **Editor Diagnostics:** If a vulnerability is found for your current version, a **red squiggle** will appear. Hovering over the version will show the vulnerability details and a direct link to the advisory.
*   **Update Safeguards:** If you click to update to a version that is known to be vulnerable, a **modal confirmation** will appear to prevent accidental updates to insecure versions.
*   **Visual Indicators:** Updatable versions with known vulnerabilities are marked with a `⚠️` indicator in the CodeLens title.

> **Note:** This feature is enabled by default and can be toggled using the `versionlens.suggestions.showVulnerabilities` setting.

## Will this extension install packages for me?

Yes, you can define a custom task that runs when you save a package document.

To set this up, follow the [Custom Install Task Guide](./docs/custom-install-task.md).

## How do I install this extension?

Install via the [VS Code Extension Gallery](https://code.visualstudio.com/docs/editor/extension-gallery).

### Manual Installation
Visit the [Releases page](https://gitlab.com/versionlens/vscode-versionlens/-/releases) for VSIX instructions.

### Installation Troubleshooting
If you are unable to install the extension, try a clean install:

1. Shut down VS Code.
2. Delete the extension folder: `{home}/.vscode/extensions/pflannery.vscode-versionlens`.
3. Restart VS Code and reinstall.

> **Note:** Check the `Log (Extension Host)` in the Output channel for specific errors during installation.
> ![image](images/faq/ext-host-log.png)

## How do I troubleshoot this extension?

*   **CodeLens Enabled:** Ensure `"editor.codeLens": true` is set in your settings.
*   **Action Icons:** Ensure `"workbench.editor.editorActionsLocation": "hidden"` is **not** set.
*   **Clear Cache:** Open the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and type `VersionLens: Clear cache`.
    *   Cache duration is managed by `versionlens.caching.duration` (default is 3 minutes).
*   **Logging:** Set your log level to `debug` via the Command Palette (`Developer: Set Log Level`) or the `VersionLens` output channel.
    ![image](images/faq/ext-log.png)
*   **Developer Tools:** If no logs are visible, check for errors in the VS Code Developer Tools (`Help > Toggle Developer Tools`).

## Commands & Settings

For a full list of available commands and configuration options, see the [Commands & Settings Guide](./docs/commands-and-settings.md).

## License

Licensed under [ISC](http://opensource.org/licenses/ISC).

Copyright &copy; 2016+ [contributors](https://gitlab.com/versionlens/vscode-versionlens/-/graphs/master)
