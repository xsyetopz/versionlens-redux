# Commands & Settings

- [Show & Hide Versions](#show--hide-versions)
- [Clear Cache](#clear-cache)
- [Bulk Updates](#bulk-updates)
- [Sorting Dependencies](#sorting-dependencies)
- [Custom Install Task](#custom-install-task)
- [Registry Authorization](#registry-authorization)
- [Key Settings](#key-settings)

---

### Show & Hide Versions

**Commands:** 
- `VersionLens: Show release versions` / `VersionLens: Hide release versions`
- `VersionLens: Show prerelease versions` / `VersionLens: Hide prerelease versions`

Toggles the visibility of version suggestions in the editor.

These commands are available in the:
- **Editor Toolbar:** As the **V** and **tag** icons, and in the `...` (secondary) menu.
- **Command Palette:** `Ctrl+Shift+P` / `Cmd+Shift+P` then type `VersionLens: Show...`.

### Clear Cache

**Command:** `VersionLens: Clear cache`

Clears all cached package suggestions. This is useful if you suspect the registry data is out of date or if you have recently updated your registry authentication settings.

By default, VersionLens caches suggestions for 3 minutes (configurable via `versionlens.caching.duration`).

### Bulk Updates

**Commands:** 
- `VersionLens: Update dependencies to latest`
- `VersionLens: Update dependencies (major-only)`
- `VersionLens: Update dependencies (minor-only)`
- `VersionLens: Update dependencies (patch-only)`

Bulk updates dependencies in the active file based on their available suggestions.

These commands are available in the:
- **Editor Toolbar:** In the `...` (secondary) menu.
- **Command Palette:** `Ctrl+Shift+P` / `Cmd+Shift+P` then type `VersionLens: Update dependencies...`.

#### Update Behaviors

- **Update dependencies to latest:** Identifies all dependencies with an **Updateable** status that are currently behind the **Latest** version and updates them.
- **Update dependencies (major-only):** Updates dependencies to their next available **Major** version suggestion.
- **Update dependencies (minor-only):** Updates dependencies to their next available **Minor** version suggestion.
- **Update dependencies (patch-only):** Updates dependencies to their next available **Patch** version suggestion.

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
| `versionlens.enabledProviders` | `[]` | Controls which package managers versionlens should enable e.g. ['npm', 'dotnet', 'docker']. If empty, all providers are enabled. **Requires VS Code restart when changed.** |
| `versionlens.suggestions.showOnStartup` | `false` | Show version lenses when a file is opened. |
| `versionlens.suggestions.showPrereleasesOnStartup` | `false` | Show prerelease suggestions on startup. |
| `versionlens.suggestions.showVulnerabilities` | `true` | Show package vulnerabilities as red squiggles in the editor. |
| `versionlens.caching.duration` | `3` | Cache duration in minutes. |
