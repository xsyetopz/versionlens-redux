export enum AuthenticationScheme {
  NotSet = 'Not consented',
  Basic = 'Basic',
  Bearer = 'Bearer',
  Custom = 'Custom'
}

export interface IAuthenticationProviderFactory {
  registerCustomAuthProvider(scheme: AuthenticationScheme, url: string): Promise<void>;
}

export type UrlAuthenticationData = {
  url: string
  protocol: string
  id: string
  label: string
  scheme: AuthenticationScheme
  isCustomProvider: boolean
}

type AuthenticationProviderInfo = {
  scheme: AuthenticationScheme
  label: string
  description: string,
  custom: boolean
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