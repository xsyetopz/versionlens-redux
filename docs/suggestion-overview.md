# Suggestion Links Overview

![image](../images/docs/suggestions/suggestion-overview-diagram.svg)

### Version Types
*   **Fixed Version:** e.g. `1.2.3`. A specific, unchanging version.
*   **Ranged Version:** e.g. `^1.2.3` or `1.2.*`. A range of acceptable versions.

---

### Status Indicators

Indicators appear next to the version information to provide a quick summary of the package state.

#### Indicator Meanings

| Prefix | Indicator | Description |
| :---: | :--- | :--- |
| 🟢 | **Latest** | The version is the latest release available. |
| 🟢 | **Satisfies Latest** | The ranged version includes and satisfies the latest available version. |
| 🟡 | **Match** | A matching version was found, but a newer version exists. |
| ⚪ | **No Match** | No version was found that satisfies the specified version string. |
| 🔴 | **Error** | An error occurred while fetching or parsing version data. |
| ↑ | **Updateable** | An update action to automatically update the version string to the suggestion. |
| ≌ | **Change Build** | An update action that will present a choice of build versions available |
| 📁 | **Directory** | The dependency points to a local file or folder location. |

> **Note:** The icons used for these indicators can be customized in your `settings.json` via the `versionlens.suggestions.indicators` setting.

---

### Suggestion Links

Version suggestions are displayed as above the dependency line in your package or project configuration file. They are grouped by **Releases** and **Prereleases**, sorted from highest to lowest version (left to right).

#### Suggestion Rules

The following suggestions appear when a newer compatible version is available:

| Name | Description |
| :--- | :--- |
| **Latest** | Shows the latest release. |
| **Major** | Shows the next available major version. |
| **Minor** | Shows the next available minor version. |
| **Patch** | Shows the next available patch version. |
| **Bump** | Shows the highest version that satisfies your current version range. |
| **Prerelease** | Shows the next available prerelease versions (e.g. alpha, beta, rc), grouped by their tag name. |
