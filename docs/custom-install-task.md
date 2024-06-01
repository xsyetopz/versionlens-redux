# How to setup a custom install task

You can define a task that will run when you save a package document. This task will only run when there are dependency changes detected.

> **NOTE**
>
> If your provider already detects changes and automatically installs packages (i.e. dotnet [c# extension](https://marketplace.visualstudio.com/items?itemName=ms-dotnettools.csharp)) then you won't need to have a custom install task

To setup the install task, it needs to be defined in your tasks.json. <br>
This is done by setting the install task label in `versionlens.{provider}.onSaveChanges` entry in your `settings.json` file.

**Example**

```js
// in your settings.json
{ 
  "versionlens.npm.onSaveChanges": "versionlens npm install" // task label name 
}
```

```js
// in your tasks.json
{
  "label": "versionlens npm install", // task label name
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
> - Ensure to set the `task.options.cwd` to the [built-in predefined variable](https://code.visualstudio.com/docs/editor/variables-reference) called `${fileDirname}` when running an install task
> - Optionally you can add the task to your **user** `tasks.json` file if you dont want to define the task for every project. This is done by pressing `ctrl+p` then selecting `Tasks: Open User Tasks`. 

> **TROUBLESHOOTING**
>
> - Versionlens needs to be enabled before **making and saving changes**
> - Will not run anything when the `onSaveChanges` setting is set to the default value of `null`
> - If the specified task is not found then vscode (by default) will prompt which task you want to run (this will never be saved in to your versionlens settings).