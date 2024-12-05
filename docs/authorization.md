# Authorization

> **NOTE**
>
> The following isn't available for Npm because npm uses npmrc files to authorize requests

By default, when versionlens detects a 401 status code it will prompt you for authentication info.

- `Step 1:` **Confirm the authorization url**

  This url is used to match package request urls and provide them with your credentials.

  Urls must:
    - be in the same domain as the package request url
    - partially match the package request url e.g. packageRequestUrl.startsWith(authorizationUrl)

  The default value is the domain host of the package request url.

  > **NOTE**
  >
  > - The default value should work fine in a lot of cases but you will need to override this value if you need to use a registry provider that hosts multiple registries on the same domain e.g. gitlab
  >
  >   example:
  >   ```js
  >   // package request url
  >   'https://gitlab.com/api/v4/projects/some-user/some-project-id/packages/nuget/download/some.package.name/index.json'
  >
  >   // authorization url (entered in this prompt step)
  >   'https://gitlab.com/api/v4/projects/some-user/some-project-id'
  >   ```

- `Step 2:` **Choose an authentication scheme**

  Supported schemes are
  - Basic Auth (username and password)
  - Custom (raw authentication header value e.g. `Bearer {YOUR_API_TOKEN}`)

- `Step 3:` **Enter your authentication credentials**

  Depending on the scheme you have choose you will be prompted for consent and your credentials e.g. username and password

## Clearing Url Authentication Data

To clear credentials

- Press `ctrl+p` then type `Remove url authentication data`.

  ![alt text](../images/docs/authorization/remove-url-authentication-data.png)

- Choose the url(s) you want to clear and press `ok`

  > **NOTE**
  >
  > If you have a project\package file opened when running this command and one of the packages needs re-authorization with the same removed url then you will be prompted to re-enter authorization. (if you dismiss this prompt then the url will be re-added to the url authentication data and marked as unconsented)

## Url Authentication Data

### How your data is stored

  - Credentials are stored in the [ExtensionContext.secrets](https://code.visualstudio.com/api/extension-capabilities/common-capabilities#data-storage) storage

  - Non-sensitive authentication info (per url) is stored in the [ExtensionContext.workspaceState](https://code.visualstudio.com/api/extension-capabilities/common-capabilities#data-storage) storage (e.g. unique per workspace).<br> Use `versionlens.authorization.removeUrlAuthentication` to clear authentication info

### What data is stored

```js
{
  url: string
  protocol: string
  id: string
  label: string
  scheme: AuthenticationScheme
}
```
