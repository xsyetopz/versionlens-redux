# Authorization

## Contents

- [Automatic Authentication](#automatic-authentication-steps)
- [Manual Authentication (Registries without 401 errors)](#manual-authentication-registries-without-401-errors)
- [Clearing Stored Credentials](#clearing-stored-credentials)
- [Data Storage & Security](#data-storage--security)

---

## Automatic Authentication

By default, when VersionLens detects a `401 Unauthorized` status code during a package request, it will automatically prompt you for credentials.

> **Note:**
> *   **NPM Registries:** Interactive authorization is not available because NPM uses its own configuration (`.npmrc`) for authentication. However, it **can** be used for `github:` prefixed packages in `package.json`.
> *   **GitHub API:** The GitHub API does not emit 401 status codes for private packages. If you are working with private GitHub repositories, see the [Manual Authentication](#manual-authentication-registries-without-401-errors) section.

### Automatic Authentication Steps

1.  **Confirm the Authorization URL**
    This URL is used to match package requests with your credentials.
    *   It must be in the same domain as the package request.
    *   The request URL must start with the authorization URL e.g. `packageRequestUrl.startsWith(authorizationUrl)`.
    *   The default value is the domain host of the package request url.
        >
        > The default values should work fine in a lot of cases but you will need to override this value if you need to use a registry provider that hosts multiple registries on the same domain e.g. gitlab

    *Example:*
    ```js
    // Authorization URL (Overridden)
    'https://gitlab.com/api/v4/projects/user/project'

    // Example Package Request URL made by version lens
    'https://gitlab.com/api/v4/projects/user/project/packages/nuget/download/some.package.name/index.json'
    ```

2.  **Choose an Authentication Scheme**
    *   **Basic Auth:** Standard username and password.
    *   **Custom:** A raw authentication header value e.g. `Bearer YOUR_API_TOKEN`.

3.  **Enter Credentials**
    Provide the requested username, password, or token. VersionLens will securely store these for future requests. [See data storage & security for more info](#data-storage--security)

---

## Manual Authentication (Registries without 401 errors)

If a registry fails silently (without emitting a 401 status code), you can manually add an authorization entry:

1.  Open the **Command Palette** (`Ctrl+Shift+P` or `Cmd+Shift+P`).
2.  Type and select `VersionLens: Add url authentication`.
    >
    > Authorization urls must:
    > - be in the same domain as the package request url
    > - partially match the package request url e.g. `packageRequestUrl.startsWith(authorizationUrl)`
    >
    >   example:
    >   ```js
    >   // package request url
    >   'https://api.github.com/repos/owner-or-org/some-repo/tags'
    >
    >   // authorization url (entered in this prompt step)
    >   'https://api.github.com/repos/owner-or-org/some-repo'
    >   ```

3.  Follow the [**Automatic Authentication Steps**](#automatic-authentication-steps) starting from **Step 2**.

---

## Clearing Stored Credentials

To remove stored credentials:

1.  Open the **Command Palette** (`Ctrl+Shift+P` or `Cmd+Shift+P`).
2.  Type and select `VersionLens: Remove url authentication`.
    ![Remove Auth](../images/docs/authorization/remove-url-authentication-data.png)
3.  Select the URL(s) you wish to clear and press **OK**.

> If you have a file open that requires the removed credentials, VersionLens will prompt you to re-authorize when it needs to fetch suggestions. (if you dismiss this prompt then the url will be re-added to the url authentication data and marked as unconsented)

---

## Data Storage & Security

VersionLens handles your credentials with care:

*   **Sensitive Data:** Credentials (tokens, passwords) are stored in the [VS Code SecretStorage](https://code.visualstudio.com/api/extension-capabilities/common-capabilities#data-storage), which uses the OS keychain e.g. Keychain Access on macOS, Windows Credential Locker.
*   **Non-Sensitive Metadata:** Metadata (URLs, labels, schemes) is stored in the [VS Code WorkspaceState](https://code.visualstudio.com/api/extension-capabilities/common-capabilities#data-storage), making it unique to each workspace.

### Stored Metadata Schema
```json
{
  "url": "string",
  "scheme": "string",
  "protocol": "string",
  "label": "string",
  "status": "string"
}
```
