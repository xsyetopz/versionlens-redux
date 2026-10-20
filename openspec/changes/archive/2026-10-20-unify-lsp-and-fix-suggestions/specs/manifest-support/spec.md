# Spec Delta

## Purpose

Defines which manifest files VersionLens recognizes, how dependencies and version references are extracted from them, how unresolvable inputs are treated, and how new file types are registered.

## ADDED Requirements

### Requirement: Descriptor-driven file registration
Each supported file type SHALL be declared once, in a single manifest table that holds its file patterns, ecosystem, format and dependency paths. The server's file classification, the VS Code `package.json` document selectors and per-ecosystem settings, and the JetBrains, Neovim and Zed file lists SHALL be generated from or checked against that table.

#### Scenario: Adding a format-based file type
- **WHEN** a contributor adds a file type whose format (JSON, TOML, YAML, XML) already has a parser
- **THEN** the change is one table row plus fixtures, and the editor file lists and settings are regenerated without hand edits

#### Scenario: Drift detection
- **WHEN** an editor file list or `package.json` selector differs from the table
- **THEN** the repository check fails and names the mismatched entry

### Requirement: Settings keys preserved
Generating `package.json` settings from the manifest table SHALL keep every existing `versionlens.*` setting key, its type and its default.

#### Scenario: Key set unchanged
- **WHEN** the settings are regenerated
- **THEN** the set of contributed setting keys equals the 0.4.4 set, plus only keys added by this change

### Requirement: Additional file types
The server SHALL recognize:
- `Cartfile`, as GitHub and Git dependencies
- `.config/dotnet-tools.json` and `dotnet-tools.json`, as NuGet tool packages
- Cargo `[patch.*]` dependency tables in `Cargo.toml` and `.cargo/config.toml`

#### Scenario: Dotnet tools manifest
- **WHEN** `.config/dotnet-tools.json` contains `"dotnet-ef": { "version": "8.0.0" }`
- **THEN** a NuGet suggestion for `dotnet-ef` is shown on `8.0.0`

#### Scenario: Cartfile entry
- **WHEN** a `Cartfile` contains `github "Alamofire/Alamofire" ~> 5.8`
- **THEN** the dependency is resolved against the repository's version tags

### Requirement: Gradle version catalog references
For `libs.versions.toml`, each `[versions]` entry SHALL be resolved through a library or plugin that references it with `version.ref`. Plugin references SHALL use the plugin marker coordinate `<id>:<id>.gradle.plugin`. The suggestion and edit range SHALL be on the `[versions]` value. Entries no library or plugin references SHALL be shown as unresolved without an error.

#### Scenario: Shared version entry
- **WHEN** `[versions] kotlin = "2.0.0"` is referenced by `kotlin-stdlib = { module = "org.jetbrains.kotlin:kotlin-stdlib", version.ref = "kotlin" }`
- **THEN** the `kotlin` entry shows the latest `kotlin-stdlib` version and a bump edits `"2.0.0"`

#### Scenario: Plugin-only reference
- **WHEN** a `[versions]` entry is referenced only by `[plugins] ksp = { id = "com.google.devtools.ksp", version.ref = "ksp" }`
- **THEN** it is resolved as `com.google.devtools.ksp:com.google.devtools.ksp.gradle.plugin`

### Requirement: GitHub Actions expressions
For runtime versions in GitHub Actions workflows, `${{ env.NAME }}` SHALL be resolved from step, job and workflow `env`, in that precedence. `${{ matrix.* }}` SHALL keep resolving from the matrix. Any expression that cannot be resolved statically SHALL be skipped silently, without a diagnostic or error lens.

#### Scenario: Workflow env reference
- **WHEN** a workflow sets `env: { NODE_VERSION: "20" }` and a step uses `node-version: ${{ env.NODE_VERSION }}`
- **THEN** the step's runtime is resolved as Node.js `20`

#### Scenario: Unresolvable expression
- **WHEN** a step uses `dotnet-version: ${{ steps.read.outputs.version }}`
- **THEN** no diagnostic, error lens or registry request is produced for it

### Requirement: Unsupported runtimes are skipped
A runtime or engine key with no supported release source SHALL be skipped without an error. This applies, for example, to `engines.vscode` in `package.json` and to setup-dotnet inputs with unsupported syntax.

#### Scenario: VS Code engine field
- **WHEN** `package.json` contains `"engines": { "vscode": "^1.75.0" }`
- **THEN** no error diagnostic or error lens is shown for it
