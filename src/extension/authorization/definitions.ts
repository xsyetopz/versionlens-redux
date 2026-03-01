/**
 * Enum representing supported authentication schemes.
 */
export enum AuthenticationScheme {
  /** No authentication scheme set. */
  NotSet = 'NotSet',
  /** Basic Authentication (username:password). */
  Basic = 'Basic',
  /** Bearer Token authentication. */
  Bearer = 'Bearer',
  /** Custom authorization value. */
  Custom = 'Custom'
}

/**
 * Enum representing the status of URL authentication.
 */
export enum UrlAuthenticationStatus {
  /** Normal status. */
  NoStatus = 'NoStatus',
  /** The user cancelled the authentication prompt. */
  UserCancelled = 'User cancelled',
  /** Authentication failed with the provided credentials. */
  CredentialsFailed = 'Credentials failed'
}

/**
 * Represents metadata for a URL's authentication configuration.
 */
export type UrlAuthenticationData = {
  /** The authorized URL. */
  readonly url: string
  /** The authentication scheme used. */
  readonly scheme: AuthenticationScheme
  /** The URL protocol (http: or https:). */
  readonly protocol: string
  /** The display label for the authentication method. */
  readonly label?: string
  /** The current authentication status. */
  readonly status: UrlAuthenticationStatus
}

/**
 * Information about a supported authentication provider.
 */
type AuthenticationProviderInfo = {
  readonly scheme: AuthenticationScheme
  readonly label: string
  readonly description: string,
}

/**
 * List of available authentication providers.
 */
export const authenticationProviders: Array<AuthenticationProviderInfo> = [
  {
    scheme: AuthenticationScheme.Basic,
    label: 'Basic Auth',
    description: 'Authenticate using basic auth credentials'
  },
  {
    scheme: AuthenticationScheme.Custom,
    label: 'Custom Value',
    description: 'Authenticate using a custom authorization value'
  },
];

/**
 * Logging templates for authentication events.
 */
export const AuthLog = {
  authProviderInfo: "Using [{label}] authentication provider for {url}"
}

/**
 * User-facing prompts for authentication.
 */
export const AuthPrompt = {
  couldNotAuthenticate: (url: string) => {
    return `Could not authenticate credentials with ${url}.\n\n`
      + "Would you like to re-enter your credentials?"
  },
  unsecureAuthorizationUrl: (url: string) => `${url} is using the unsecure HTTP protocol.\n\n` +
    "Are you sure you want to send your credentials with this url?"
}

/**
 * Prompts for confirming or entering authorization URLs.
 */
export const confirmAuthUrlPrompt = {
  enterAuthorizationUrl: "Enter the authorization url for package requests",
  differentDomain: "The authorization url must be in the same domain as the request url",
  urlPartialMismatch: (requestUrl: string) => {
    return `The authorization url must partially match the request url ${requestUrl}`;
  },
};

/**
 * Prompts for choosing an authentication scheme.
 */
export const chooseAuthSchemePrompt = {
  chooseAuthenticationScheme: (url: string) => `Choose an authentication scheme for ${url}`
}

/**
 * Prompts for basic authentication details.
 */
export const basicAuthPrompt = {
  enterBasicAuthUsername: (url: string) => `Enter the basic auth username for ${url}`,
  enterBasicAuthPassword: (url: string) => `Enter the basic auth password for ${url}`,
  invalidBasicAuthUsername: "You cannot have a ':' character in the user name.\n\n"
    + "Do you want re-enter the username or cancel?",
};

/**
 * Prompts for custom authorization values.
 */
export const customAuthPrompt = {
  enterAuthValue: (url: string) => `Enter the authorization value for ${url}`
};