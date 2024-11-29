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
  }
}