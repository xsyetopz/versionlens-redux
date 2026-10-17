# Spec Delta

## Purpose

Reports available dependency updates as lint-style diagnostics with one-step quick fixes, so users can find outdated dependencies in their editor's problems list instead of scanning code lenses.

## ADDED Requirements

### Requirement: Update-available diagnostic
After a manifest is resolved, the server SHALL publish one diagnostic per dependency whose requirement does not allow the latest release, or whose floor bump to latest changes the major or minor number, with:
- range: the dependency's requirement text
- code: `update-available`
- source: `VersionLens`
- message: the current requirement and the latest offered target

#### Scenario: Outdated dependency
- **WHEN** `package.json` contains `"astro": "^7.2.2"` and latest is 7.3.5
- **THEN** a diagnostic with code `update-available` and message `^7.2.2 → ^7.3.5` covers the text `^7.2.2`

#### Scenario: Up-to-date dependency
- **WHEN** a dependency's requirement text would not change under any offered bump
- **THEN** no update-available diagnostic is published for it

#### Scenario: Patch-only floor bump
- **WHEN** `package.json` contains `"@biomejs/biome": "^2.5.8"` and latest is 2.5.14
- **THEN** no update-available diagnostic is published, and the floor bump to `^2.5.14` stays available as a code lens and a code action

### Requirement: Configurable severity
Update-available diagnostics SHALL default to Information severity. A setting `versionlens.diagnostics.updateSeverity` SHALL accept `error`, `warning`, `information`, `hint` and `off`. With `off`, no update-available diagnostics are published.

#### Scenario: Default severity
- **WHEN** the setting is not configured
- **THEN** update-available diagnostics have severity Information and appear in VS Code's Problems panel

#### Scenario: Disabled
- **WHEN** the setting is `off`
- **THEN** no update-available diagnostics are published and vulnerability diagnostics are unaffected

### Requirement: Vulnerability diagnostics keep precedence
Vulnerability diagnostics SHALL keep Error severity and their advisory code. A dependency that is both vulnerable and outdated SHALL get both diagnostics.

#### Scenario: Vulnerable and outdated
- **WHEN** a dependency has an advisory and a newer release
- **THEN** one Error diagnostic with the advisory id and one update-available diagnostic are published

### Requirement: Bump quick fix
The server SHALL answer `textDocument/codeAction` for a range inside a dependency's requirement, with or without an update-available diagnostic. It SHALL return one `quickfix` action per offered bump, attached to the diagnostic when one exists, titled `Bump to <requirement>`, each carrying the workspace edit inline. The action for the latest target SHALL be marked preferred. Producing actions SHALL NOT trigger network requests.

#### Scenario: Apply quick fix
- **WHEN** the user invokes the preferred quick fix on `"astro": "^7.2.2"`
- **THEN** the requirement text becomes `^7.3.5` and the update-available diagnostic for it is cleared after re-resolution

#### Scenario: Multiple targets
- **WHEN** a dependency has distinct minor and major bumps
- **THEN** the code action list contains one action per distinct target, latest preferred

### Requirement: Diagnostics follow document changes
Update-available diagnostics SHALL be republished for the document version they describe. Diagnostics for a closed document SHALL be cleared.

#### Scenario: Close document
- **WHEN** a manifest with update diagnostics is closed
- **THEN** the server publishes an empty diagnostic list for it
