# Version Lens for Visual Studio Code

[![Badge for version for Visual Studio Code extension](https://vsmarketplacebadges.dev/version/pflannery.vscode-versionlens.png?color=blue&style=?style=for-the-badge&logo=visual-studio-code)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens&wt.mc_id=vscode-versionlens-gitlab)
[![Installs](https://vsmarketplacebadges.dev/installs-short/pflannery.vscode-versionlens.png?color=blue&style=flat-square)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens)
[![Rating](https://vsmarketplacebadges.dev/rating/pflannery.vscode-versionlens.png?color=blue&style=flat-square)](https://marketplace.visualstudio.com/items?itemName=pflannery.vscode-versionlens)
[![The ISC license](https://img.shields.io/badge/license-ISC-orange.png?color=blue&style=flat-square)](http://opensource.org/licenses/ISC)

This project is `active`, not sponsored and not funded.

[![BuyMeACoffee](https://www.buymeacoffee.com/assets/img/custom_images/purple_img.png)](https://www.buymeacoffee.com/peterf)

This extension shows __version__ information when opening a package or project for one of the following:

- cargo https://doc.rust-lang.org/cargo/ 
- composer https://getcomposer.org/
- dotnet https://www.dotnetfoundation.org/
- dub https://code.dlang.org/
- maven https://maven.apache.org/
- npm https://www.npmjs.com/
  - jspm https://jspm.org/
  - pnpm https://pnpm.io/
- pub https://pub.dev/
- python https://pypi.org/

Version lens abides to [semver rules](https://semver.org/) and uses the [Node Semver](https://github.com/npm/node-semver) package to compare and sort versions.

## Contents

- [How do I install this extension?](#how-do-i-install-this-extension)
- [How do I see version information?](#how-do-i-see-version-information)
- [Can I see pre-release versions?](#can-i-see-pre-release-versions)
- [Will this extension install packages?](#will-this-extension-install-packages)
- [Can I install this extension manually?](#can-i-install-this-extension-manually)
- [I'm not able to install this extention](#im-not-able-to-install-this-extention)
- [How do I troubleshoot this extension?](#how-do-i-troubleshoot-this-extension)

## How do I install this extension?

Follow this link on [how to install vscode extensions](https://code.visualstudio.com/docs/editor/extension-gallery)

## How do I see version information?

Click the V icon in the package\project file toolbar.

You can also choose the default startup state by setting `versionlens.suggestions.showOnStartup`

![Show releases](https://gitlab.com/versionlens/vscode-versionlens/-/raw/master/images/faq/show-releases.gif)

## Can I see pre-release versions?

Yes, click on the tag icon in the package\project file toolbar.

You can also choose the default startup state by setting `versionlens.suggestions.showPrereleasesOnStartup`

![Show prereleases](https://gitlab.com/versionlens/vscode-versionlens/-/raw/master/images/faq/show-prereleases.gif)

## Will this extension install packages?

You can define a task that will run when you save a package document. (runs only when there are dependency changes)

The install task needs to be defined in your tasks.json.

You then set the `versionlens.{provider}.onSaveChanges` setting to the your install task label.

**Example**

```js
// in your settings.json snippet
{ "versionlens.npm.onSaveChanges": "versionlens npm install" }
```

```js
// tasks.json
{
  "label": "versionlens npm install",
  "command": "npm",
  "type": "shell",
  "args": ["install"],
  "options": {
    // sets the cwd to the current file dir
    "cwd": "${fileDirname}"
  },
  // customizable settings
  "presentation": {
    "echo": true,
    "reveal": "always",
    "panel": "shared",
    "clear": true,
  },
}
```

> **NOTE**
>
> - If your provider already detects changes then installs packages (i.e. dotnet [c# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp)) then you won't need to have a custom install task
> - Versionlens will need to be enabled before **making and saving changes**
> - Will not run anything when the `onSaveChanges` setting is set to the default value of `null`
> - Optionally you can add the task to your user `tasks.json` file if you dont want to define the task for every project. This is done by pressing `ctrl+p` then selecting `Tasks: Open User Tasks`. 
> - If the specified task is not found then vscode (by default) will prompt which task you want to run (this will never be saved in to your versionlens settings).
> - Ensure to set the `task.options.cwd` to the [built-in predefined variable](https://code.visualstudio.com/docs/editor/variables-reference) called `${fileDirname}` when running an install task

## Can I install this extension manually?

Yes goto the [release page for instructions](https://gitlab.com/versionlens/vscode-versionlens/-/releases)

## I'm not able to install this extention

Try a clean install:

- Shut down vscode
- Delete the extension folder `{home}/.vscode/extensions/pflannery.vscode-versionlens*`
- Open vscode and try reinstalling the extension again

If that fails then have a look in the `Log (Extension Host)` channel. Report it here if that doesn't help.

![image](https://gitlab.com/versionlens/vscode-versionlens/-/raw/master/images/faq/ext-host-log.png)

## How do I troubleshoot this extension?

- Ensure you have `"editor.codeLens": true` set in your settings.

- Ensure that the package\project file open is using the correct file type. i.e. json instead of jsonc

  ![image](https://gitlab.com/versionlens/vscode-versionlens/-/raw/master/images/faq/json-file-type.png)

- Version lens writes a log to an output channel in vscode.

  If your experiencing issues please set your `versionlens.logging.level` to `debug` (vscode needs to be restarted) 
    
  Then open the channel like:
    
  ![image](https://gitlab.com/versionlens/vscode-versionlens/-/raw/master/images/faq/ext-log.png)

- In the worst case no logs are output. There maybe an error in the developer tools of vscode. You can open the dev tools from the `help menu` in vscode (Ctrl+Shift+I)

## License

Licensed under ISC

Copyright &copy; 2016+ [contributors](https://gitlab.com/versionlens/vscode-versionlens/-/graphs/master)
