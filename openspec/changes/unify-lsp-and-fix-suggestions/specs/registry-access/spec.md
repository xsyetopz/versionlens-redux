# Spec Delta

## Purpose

Defines how the server authenticates registry requests, where it obtains credentials, and how it reports authorization, rate-limit, network and cache failures.

## ADDED Requirements

### Requirement: GitHub credentials
For requests to `api.github.com`, the server SHALL send a token from the first available source:
1. a credential supplied by the client through `versionlens/authorization`
2. `GITHUB_TOKEN`
3. `GH_TOKEN`
4. the output of `gh auth token`, when `gh` is installed

Tokens SHALL NOT be written to logs, diagnostics or the persistent cache.

#### Scenario: Token from environment
- **WHEN** `GITHUB_TOKEN` is set and a workflow references `actions/checkout@v4`
- **THEN** the GitHub request carries an `Authorization` header with that token

### Requirement: Authorization request to the client
When a registry answers 401 or 403 for a host with no working credential, the server SHALL send the client a `versionlens/authorization` request naming the host. A client that supports it MAY answer with a credential, which the server uses for that host for the rest of the session. A client that does not support it, or declines, SHALL cause the server to fall back to environment and configuration-file credentials. The request SHALL NOT block other resolution.

#### Scenario: VS Code supplies a stored secret
- **WHEN** a private npm registry answers 401 and the VS Code client has a secret stored for that host
- **THEN** the client answers with it and the dependency is resolved on retry

#### Scenario: Client without support
- **WHEN** a Neovim client receives `versionlens/authorization`
- **THEN** it responds with a method-not-found error and the server continues with other credential sources

### Requirement: Rate-limit reporting
When a registry reports a rate limit (HTTP 403 or 429 with rate-limit headers), the server SHALL stop requesting that host until the reset time, mark affected dependencies as rate-limited, and send one `window/showMessage` warning per reset period. The warning SHALL include the reset time and how to authenticate. Rate limits SHALL NOT be published as per-dependency error diagnostics.

#### Scenario: GitHub unauthenticated limit
- **WHEN** GitHub answers 403 with `x-ratelimit-remaining: 0` and `x-ratelimit-reset`
- **THEN** one warning states the reset time and suggests setting `GITHUB_TOKEN` or running `gh auth login`, and no further GitHub requests are made until the reset time

### Requirement: No lookups for non-version refs
The server SHALL NOT make registry requests for references that are not versions, such as branch names, non-version tags and bare commit SHAs without a version comment.

#### Scenario: Branch ref
- **WHEN** a workflow uses `taiki-e/install-action@just`
- **THEN** no GitHub API request is made for it

### Requirement: Proxy and trust store
HTTP requests SHALL honor `HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY` and `NO_PROXY` (case-insensitive). TLS verification SHALL trust the operating system's certificate store in addition to the bundled roots, and SHALL keep honoring `versionlens.http.strictSSL`.

#### Scenario: Corporate proxy with custom CA
- **WHEN** `HTTPS_PROXY` points at a proxy whose CA is installed in the system trust store
- **THEN** registry requests succeed through the proxy

### Requirement: Cache failures are not fatal
If the persistent cache directory cannot be created or written, the server SHALL keep running with an in-memory cache and log one warning.

#### Scenario: Read-only cache directory
- **WHEN** the cache directory is read-only
- **THEN** the server starts, resolves dependencies, and logs that the persistent cache is disabled

### Requirement: Errors include their cause
When a dependency cannot be resolved because of a registry, network or parse failure, its error lens and log entry SHALL include the cause: HTTP status and host, the network error kind, or the parse error.

#### Scenario: Registry 404
- **WHEN** npm answers 404 for a package
- **THEN** the error lens reads `not found (404 registry.npmjs.org)` rather than a generic error
