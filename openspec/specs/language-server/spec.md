# language-server Specification

## Purpose

Defines the single VersionLens language server that every supported editor (VS Code, JetBrains, Neovim, Zed) uses to analyze manifests, present version information and apply updates.

## Requirements

### Requirement: Single engine for all editors
Every supported editor SHALL obtain dependency analysis, suggestions, diagnostics and edits exclusively from the `versionlens-lsp` server over the Language Server Protocol on stdio. No editor package SHALL load a native library in-process.

#### Scenario: VS Code uses the language server
- **WHEN** the VS Code extension activates on a workspace containing a supported manifest
- **THEN** it starts `versionlens-lsp` as a child process and receives code lenses and diagnostics from it

#### Scenario: Same behavior across editors
- **WHEN** the same manifest is opened in VS Code and in Neovim with equivalent settings
- **THEN** both editors receive the same set of suggestions and update-available diagnostics for it

### Requirement: Editing does not discard resolved state
Changing the content of an open document SHALL NOT clear cached registry results or restart workspace discovery. Only dependencies whose name, requirement or source changed SHALL be resolved again.

#### Scenario: Typing in a manifest
- **WHEN** a user edits a comment line in an open `package.json` whose dependencies were already resolved
- **THEN** no registry request is made and the existing suggestions remain displayed

#### Scenario: Changing one requirement
- **WHEN** a user changes one dependency's requirement in an open manifest
- **THEN** only that dependency's presentation is recomputed and other dependencies keep their resolved state

### Requirement: Responsive protocol loop
The server SHALL keep answering protocol messages while resolution, file I/O and network work run. The server SHALL NOT perform blocking file or network I/O on the thread that reads protocol messages.

#### Scenario: Slow registry
- **WHEN** a registry request takes longer than 5 seconds
- **THEN** the server still answers `textDocument/codeLens` for other documents and `shutdown` without waiting for the request

### Requirement: Push-driven resolution
The server SHALL start resolving an opened or changed manifest without waiting for a code lens request, debounced after changes, and SHALL publish diagnostics when resolution completes.

#### Scenario: Client without code lens support
- **WHEN** a client that never sends `textDocument/codeLens` opens a supported manifest with an outdated dependency
- **THEN** the server publishes an update-available diagnostic for it

### Requirement: Internal failures are not shown as diagnostics
The server SHALL NOT publish cancellations, internal errors, snapshot limits or discovery I/O failures as document diagnostics. It SHALL report them through `window/logMessage` and through the status notification. Failed resolution SHALL be retried with exponential backoff, not at a fixed short interval.

#### Scenario: Cancelled resolution
- **WHEN** a resolution is cancelled because the document changed
- **THEN** no diagnostic is published for the cancellation

#### Scenario: Persistent failure
- **WHEN** resolution for a document fails repeatedly
- **THEN** retries are spaced with increasing delay and the failure is visible in the server log

### Requirement: Per-folder configuration
The server SHALL read settings with `workspace/configuration` scoped to each document's workspace folder, using the same setting keys the VS Code extension contributes (`versionlens.*`). It SHALL re-read them on `workspace/didChangeConfiguration`. Clients that do not support configuration requests SHALL get documented defaults.

#### Scenario: Different folders, different settings
- **WHEN** two workspace folders set different `versionlens.suggestions.showPrereleases` values
- **THEN** documents in each folder are presented according to their own folder's value

#### Scenario: Client without configuration support
- **WHEN** a client does not advertise `workspace.configuration`
- **THEN** the server uses default settings and does not error

### Requirement: Status reporting
The server SHALL send a `versionlens/status` notification whenever a document's state changes between idle, busy and error, so clients can show busy and error indicators.

#### Scenario: Busy indicator
- **WHEN** resolution of the active document starts and later completes
- **THEN** the client receives a busy status followed by an idle status for that document

### Requirement: Editor-neutral commands
The server SHALL expose its operations as `workspace/executeCommand` commands with typed, named arguments:
- update one dependency
- bulk update (latest, major, minor, patch)
- sort dependencies
- choose build
- clear cache
- query manifest information

Command arguments SHALL NOT be positional string lists.

#### Scenario: Bulk update
- **WHEN** a client executes the bulk update command with `{ uri, level: "minor" }`
- **THEN** the server applies one workspace edit that updates every dependency in that document that has a minor update

#### Scenario: Clear cache
- **WHEN** a client executes the clear cache command
- **THEN** subsequent resolutions fetch fresh registry data

### Requirement: Applying an update always resolves
When an update command is executed, the server SHALL either apply the edit through `workspace/applyEdit` or return an error describing why it could not. It SHALL never leave the request pending.

#### Scenario: Stale document version
- **WHEN** the user triggers an update after the document changed since the lens was produced
- **THEN** the server recomputes the edit against the current document and applies it, or returns an error the client shows to the user

#### Scenario: Client rejects edit
- **WHEN** the client responds to `workspace/applyEdit` with `applied: false`
- **THEN** the command returns an error containing the client's failure reason
