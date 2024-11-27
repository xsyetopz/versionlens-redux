import type {
  AuthenticationGetSessionOptions,
  AuthenticationProvider,
  AuthenticationProviderOptions,
  AuthenticationSession,
  AuthenticationSessionAccountInformation,
  AuthenticationSessionsChangeEvent,
  CancellationToken,
  Disposable,
  Event,
  InputBoxOptions,
  MessageOptions,
  QuickPickItem,
  QuickPickOptions
} from 'vscode';

export enum AuthenticationScheme {
  NotSet = 'Not consented',
  Basic = 'Basic',
  Bearer = 'Bearer',
  Raw = 'Raw'
}

export interface IAuthenticationProviderFactory {
  registerCustomAuthProvider(scheme: AuthenticationScheme, url: string): Promise<void>;
}

/***
 * Adapter interface for the vscode authentication namespace
 */
export interface IVsCodeAuthentication {
  getAccounts(providerId: string): Promise<readonly AuthenticationSessionAccountInformation[]>;

  onDidChangeSessions: Event<AuthenticationSessionsChangeEvent>

  getSession(
    providerId: string,
    scopes: readonly string[],
    options?: AuthenticationGetSessionOptions
  ): Promise<AuthenticationSession | undefined>;

  registerAuthenticationProvider(
    id: string,
    label: string,
    provider: AuthenticationProvider,
    options?: AuthenticationProviderOptions
  ): Disposable
}

/***
 * Adapter interface for the vscode window namespace
 */
export interface IVsCodeWindow {
  showInputBox(
    options?: InputBoxOptions,
    token?: CancellationToken
  ): Thenable<string | undefined>

  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany?: false },
    token?: CancellationToken
  ): Thenable<T | undefined>;

  showQuickPick<T extends QuickPickItem>(
    items: readonly T[] | Thenable<readonly T[]>,
    options?: QuickPickOptions & { canPickMany: true },
    token?: CancellationToken
  ): Thenable<T[] | undefined>;

  showInformationMessage<T extends string>(
    message: string,
    options: MessageOptions,
    ...items: T[]
  ): Thenable<T | undefined>;
}

export type UrlAuthenticationData = {
  url: string
  protocol: string
  id: string
  label: string
  scheme: AuthenticationScheme
  isCustomProvider: boolean
}

export type UrlAuthenticationSessionData = {
  consent: boolean
  retries: number
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
    scheme: AuthenticationScheme.Raw,
    label: 'Raw Value',
    description: 'Authenticate using a raw authorization value',
    custom: true
  },
];