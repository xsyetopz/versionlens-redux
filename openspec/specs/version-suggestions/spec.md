# version-suggestions Specification

## Purpose

Defines which version suggestions VersionLens offers for a dependency requirement, how its status is classified, and how suggestions are labeled, consistently across every ecosystem and editor.

## Requirements

### Requirement: One dialect-aware currency rule
Status classification and bump generation SHALL use the same rule, according to the ecosystem's version dialect (semver, PEP 440, Maven, NuGet, and so on), to decide whether a requirement already targets the latest release. Exact pins such as `=x`, `==x` and `[x]` SHALL count as targeting `x`.

#### Scenario: Cargo exact pin at latest
- **WHEN** `Cargo.toml` contains `zed_extension_api = "=0.7.0"` and the latest release is 0.7.0
- **THEN** the status is `latest 0.7.0` and no bump suggestion is offered

#### Scenario: Python exact pin at latest
- **WHEN** a requirements file contains `requests==2.34.2` and the latest release is 2.34.2
- **THEN** the status is `latest 2.34.2` and no bump suggestion is offered

### Requirement: No suggestion without a text change
A bump suggestion SHALL be offered only if applying it changes the dependency's requirement text. Each offered suggestion SHALL produce a non-empty edit.

#### Scenario: Redundant latest suppressed
- **WHEN** applying the latest target would produce the identical requirement text
- **THEN** that suggestion is not offered

#### Scenario: Status lens without an update has no update command
- **WHEN** a dependency's lens only reports a status and no suggestion with a non-empty edit exists
- **THEN** the lens carries no update command

### Requirement: Floor bump for satisfied ranges
When a range requirement satisfies the latest release but its minimum version is lower than latest, a bump to the same range operator with latest as the minimum SHALL be offered. Versions SHALL be compared by version semantics, not by string prefix.

#### Scenario: Caret range behind latest
- **WHEN** `package.json` contains `"@biomejs/biome": "^2.5.8"` and latest is 2.5.14
- **THEN** the status is `satisfies latest 2.5.14` and a bump to `^2.5.14` is offered

#### Scenario: Prefix-similar versions
- **WHEN** a requirement's minimum is 1.2.1 and latest is 1.2.10
- **THEN** the requirement is not classified as current and a bump to 1.2.10 is offered

### Requirement: Bounded suggestion set
For each dependency, the offered bumps SHALL be at most: the newest version satisfying the current requirement (when it differs), the newest patch, the newest minor, the newest major, and latest. Duplicate targets SHALL be merged. Intermediate releases SHALL NOT be listed individually.

#### Scenario: Many intermediate releases
- **WHEN** `"@astrojs/starlight": "^0.41.7"` has releases 0.41.11, 0.42.0 through 0.42.5, and latest 0.42.6
- **THEN** the suggestions are `satisfies 0.41.11`, a bump to `^0.41.11`, and `latest ^0.42.6`, with no per-version entries for 0.42.0 through 0.42.5

### Requirement: Floating and pinned Git refs
For a GitHub Action pinned to a commit SHA with a version comment, the current version SHALL be the most specific tag on that commit. A latest suggestion SHALL be offered only if the latest tag points to a different commit.

#### Scenario: SHA already at latest
- **WHEN** `uses: actions/github-script@3a2844b7… # v9.0.0` and tags `v9` and `v9.0.0` both point to that commit
- **THEN** the status is `latest v9.0.0` and no bump is offered

#### Scenario: Non-version ref
- **WHEN** `uses: taiki-e/install-action@just` references a branch or non-version tag
- **THEN** the dependency is shown as a fixed ref, no update lookup is made, and no error is shown

### Requirement: Ecosystem requirement semantics
Requirement interpretation SHALL follow each ecosystem's rules. In Cargo, a bare version such as `"1.0.200"` SHALL be treated as a caret requirement, not as a fixed pin.

#### Scenario: Cargo bare version
- **WHEN** `Cargo.toml` contains `serde = "1.0.200"` and latest is 1.0.229
- **THEN** the status is `satisfies latest 1.0.229` and a bump to `1.0.229` is offered

### Requirement: Project version is not a dependency
A manifest's own package version field (for example `version` in `package.json` or `[package].version` in `Cargo.toml`) SHALL NOT receive status or bump suggestions.

#### Scenario: Package version field
- **WHEN** `package.json` contains `"version": "0.4.4"`
- **THEN** no code lens, diagnostic or suggestion is produced for that field

### Requirement: Label formatting
Suggestion labels SHALL use single spacing between the indicator and the text.

#### Scenario: Update label
- **WHEN** a latest bump is rendered
- **THEN** its label is `↑ latest <version>`, with exactly one space after the arrow
