# How to Setup a Custom Install Task

VersionLens allows you to define a task that automatically runs when you save a package document, but only when dependency changes are detected. This is useful for automatically running `npm install`, `composer install`, etc.

> **Note:** Many extensions (like the C# extension for Dotnet) already detect changes and automatically restore packages. In such cases, you do not need a custom VersionLens task.

---

## Configuration

Setting up a custom install task requires two steps: defining the task in VS Code and linking it in VersionLens settings.

### Step 1: Define the Task in `tasks.json`

Create or update your `.vscode/tasks.json` file. Ensure you set the `options.cwd` to `${fileDirname}` so the command runs in the correct directory. [built-in predefined variables reference](https://code.visualstudio.com/docs/editor/variables-reference)

**Example: NPM Install Task**
```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "VersionLens: NPM Install",
      "command": "npm",
      "type": "shell",
      "args": ["install"],
      "options": {
        "cwd": "${fileDirname}"
      },
      "presentation": {
        "echo": true,
        "reveal": "always",
        "panel": "shared",
        "clear": true
      }
    }
  ]
}
```

### Step 2: Link the Task in `settings.json`

Link the task label to the specific VersionLens provider in your VS Code `settings.json`.

```json
{
  "versionlens.npm.onSaveChanges": "VersionLens: NPM Install"
}
```

---

## Tips & Troubleshooting

*   **Global Tasks:** If you want the task to be available across all projects, add it to your **User Tasks** (`Ctrl+Shift+P` > `Tasks: Open User Tasks`).
*   **Predefined Variables:** Always use `${fileDirname}` for `cwd` to ensure the install command runs relative to the `package.json` (or equivalent) being saved.
*   **Task Not Found:** If VersionLens cannot find the task specified in `settings.json`, VS Code will display a prompt asking you to select a task. This selection will not be saved to VersionLens settings.
*   **Disabled by Default:** The `onSaveChanges` setting is `null` by default, meaning no task will run until configured.
