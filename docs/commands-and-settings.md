# Commands & Settings

- [Clear Cache](#clear-cache)
- [Bulk Updates](#bulk-updates)
- [Sorting Dependencies](#sorting-dependencies)
- [Custom Install Task](#custom-install-task)
- [Registry Authorization](#registry-authorization)
- [Key Settings](#key-settings)

---

### Clear Cache

**Command:** `VersionLens: Clear cache`

Clears all cached package suggestions. This is useful if you suspect the registry data is out of date or if you have recently updated your registry authentication settings.

By default, VersionLens caches suggestions for 3 minutes (configurable via `versionlens.caching.duration`).

### Bulk Updates

**Command:** `VersionLens: Update dependencies to latest`

Bulk updates all dependencies in the active file to their latest available versions.

This command is available in the:
- **Editor Toolbar:** In the `...` (secondary) menu.
- **Command Palette:** `Ctrl+Shift+P` / `Cmd+Shift+P` then type `VersionLens: Update dependencies to latest`.

When executed, it will identify all dependencies with an **Updateable** status that are currently behind the **Latest** version and update their version strings in the document.

### Sorting Dependencies

**Command:** `VersionLens: Sort dependencies alphabetically`

Sorts all dependencies in the active file alphabetically within their respective groups (e.g., `dependencies`, `devDependencies`, or Ruby `group :production`).

This command is available in the:
- **Editor Toolbar:** In the `...` (secondary) menu.
- **Command Palette:** `Ctrl+Shift+P` / `Cmd+Shift+P` then type `VersionLens: Sort dependencies alphabetically`.

### Custom Install Task

**Command:** `VersionLens: Run custom install task`

Executes a custom task defined in your `tasks.json`. This command only appears if you have configured an `onSaveChanges` task for the active package manager (e.g., `versionlens.npm.onSaveChanges`).

For more details on how to set this up, see the [Custom Install Task Guide](./custom-install-task.md).

### Registry Authorization

**Commands:** 
- `VersionLens: Add url authentication`
- `VersionLens: Remove url authentication`

These commands allow you to manually manage credentials for private or restricted registries. VersionLens also provides an interactive workflow that triggers automatically when a registry returns a `401 Unauthorized` response.

For more details on managing credentials, see the [Authorization Guide](./authorization.md).

---

### Key Settings

| Setting | Default | Description |
| ------- | ------- | ----------- |
| `versionlens.enabledProviders` | `[]` | Controls which package managers versionlens should enable e.g. ['npm', 'dotnet']. If empty, all providers are enabled. |
| `versionlens.suggestions.showOnStartup` | `false` | Show version lenses when a file is opened. |
| `versionlens.suggestions.showPrereleasesOnStartup` | `false` | Show prerelease suggestions on startup. |
| `versionlens.caching.duration` | `3` | Cache duration in minutes. |
