export enum AuthenticationScheme {
  NotSet = 'NotSet',
  Basic = 'Basic',
  Bearer = 'Bearer',
  Custom = 'Custom'
}

export enum UrlAuthenticationStatus {
  NoStatus = 'NoStatus',
  NotConsented = 'Not consented',
  CredentialsFailed = 'Credentials failed'
}

export interface IAuthenticationProviderFactory {
  registerCustomAuthProvider(scheme: AuthenticationScheme, url: string): Promise<void>;
}

export type UrlAuthenticationData = {
  readonly url: string
  readonly protocol: string
  readonly id: string
  readonly label: string
  readonly scheme: AuthenticationScheme
  readonly status: UrlAuthenticationStatus
  readonly isCustomProvider: boolean
}

type AuthenticationProviderInfo = {
  readonly scheme: AuthenticationScheme
  readonly label: string
  readonly description: string,
  readonly custom: boolean
}

export const authenticationProviders: Array<AuthenticationProviderInfo> = [
  {
    scheme: AuthenticationScheme.Basic,
    label: 'Basic Auth',
    description: 'Authenticate using basic auth credentials',
    custom: true
  },
  {
    scheme: AuthenticationScheme.Bearer,
    label: 'Github',
    description: 'Authenticate using github',
    custom: false
  },
  {
    scheme: AuthenticationScheme.Bearer,
    label: 'Microsoft',
    description: 'Authenticate using microsoft',
    custom: false
  },
  {
    scheme: AuthenticationScheme.Custom,
    label: 'Custom Value',
    description: 'Authenticate using a custom authorization value',
    custom: true
  },
];

export const AuthLog = {
  authProviderInfo: "Using [%s] authentication provider for %s",
  couldNotAutheticateError: "Could not authenticate using '%s' for %s. %s"
}

export const AuthPrompt = {
  couldNotAuthenticate: (url: string) => {
    return `Could not authenticate credentials with ${url}.\n\n`
      + "Would you like to re-enter your credentials?"
  },
  unsecureAuthorizationUrl: (url: string) => `${url} is using the unsecure HTTP protocol.\n\n` +
    "Are you sure you want to send your credentials with this url?"
}

export const confirmAuthUrlPrompt = {
  enterAuthorizationUrl: "Enter the authorization url for package requests",
  differentDomain: "The authorization url must be in the same domain as the request url",
  urlPartialMismatch: (requestUrl: string) => {
    return `The authorization url must partially match the request url ${requestUrl}`;
  },
};

export const chooseAuthSchemePrompt = {
  chooseAuthenticationScheme: (url: string) => `Choose an authentication scheme for ${url}`
}

export const basicAuthPrompt = {
  enterBasicAuthUsername: (url: string) => `Enter the basic auth username for ${url}`,
  enterBasicAuthPassword: (url: string) => `Enter the basic auth password for ${url}`,
  invalidBasicAuthUsername: "You cannot have a ':' character in the user name.\n\n"
    + "Do you want re-enter the username or cancel?",
};

export const customAuthPrompt = {
  enterAuthValue: (url: string) => `Enter the authorization value for ${url}`
};