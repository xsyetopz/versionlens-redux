import type { ILogger } from '#domain/logging';
import {
  type UrlAuthenticationData,
  AuthenticationScheme,
  UrlAuthenticationStatus,
  authenticationProviders,
  createCustomProviderId,
  createUrlAuthData
} from '#extension/authorization';
import type { IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { QuickPickItem } from 'vscode';

export class AuthenticationInteractions {

  constructor(readonly window: IVsCodeWindow, readonly logger: ILogger) {
    throwUndefinedOrNull('window', window);
    throwUndefinedOrNull('logger', logger);
  }

  async chooseAuthenticationType(url: string): Promise<UrlAuthenticationData | undefined> {
    const pickItems: QuickPickItem[] = Array.from(
      authenticationProviders,
      authProviderInfo => (<any>{
        // ui data
        label: authProviderInfo.label,
        detail: authProviderInfo.description,

        // selected item data
        providerLabel: authProviderInfo.label,
        providerScheme: authProviderInfo.scheme,
        providerBuiltIn: authProviderInfo.custom === false,
        providerId: authProviderInfo.custom
          ? createCustomProviderId(authProviderInfo.scheme, url)
          : authProviderInfo.label.toLowerCase()
      })
    );

    // determine which auth provider to use
    const selectedQuickPick = await this.window.showQuickPick(
      pickItems,
      {
        title: `Authentication is required for ${url}`,
        placeHolder: "Choose an authentication provider"
      }
    );

    // check the user made a selection
    if (selectedQuickPick === undefined) return undefined;

    // extract the selection data
    const {
      providerBuiltIn,
      providerId: id,
      providerLabel: label,
      providerScheme: scheme
    } = <any>selectedQuickPick;

    // map to result
    return createUrlAuthData(
      url,
      id,
      label,
      scheme,
      UrlAuthenticationStatus.NoStatus,
      providerBuiltIn === false
    );
  }

  async enterBasicAuthDetails(url: string): Promise<string | undefined> {
    // prompt unsecure urls
    if (url.startsWith('https:') == false) {
      const allowUnsecured = await this.promptUnsecured(url);
      if (allowUnsecured === undefined) return undefined;
    }

    // prompt for the username
    const username = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: `Enter the basic auth username for ${url}`,
      placeHolder: 'Basic auth username',
      password: false
    });
    if (username === undefined) return undefined;

    // validate username
    if (username.includes(':')) {
      const retry = await this.promptRetry(
        "You cannot have a ':' character in the user name. Do you want re-enter the username or cancel?",
        `Basic auth username for ${url}`,
      );
      if (retry === undefined) return undefined;

      return await this.enterBasicAuthDetails(url);
    }

    // prompt for the password
    const password = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: `Enter the basic auth password for ${url}`,
      placeHolder: 'Basic auth password',
      password: true,
    });
    if (password === undefined) return undefined;

    // encode username:password
    return btoa(`${username}:${password}`);
  }

  async enterRawAuthDetails(url: string): Promise<string | undefined> {
    // prompt unsecure urls
    if (url.startsWith('https:') == false) {
      const allowUnsecure = await this.promptUnsecured(url);
      if (allowUnsecure === undefined) return undefined;
    }

    // prompt for the value
    const value = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: `Enter the authorization value for ${url}`,
      placeHolder: 'Authorization value',
      password: true
    });

    return value ? value : undefined;
  }

  async chooseUrlAuthToClear(urlAuthData: UrlAuthenticationData[]): Promise<UrlAuthenticationData[]> {
    const pickItems: QuickPickItem[] = Array.from(
      urlAuthData,
      urlAuth => {
        const detailBuilder = [];

        if (urlAuth.scheme !== AuthenticationScheme.NotSet) {
          detailBuilder.push(urlAuth.protocol === 'http:' ? 'Unsecured' : 'Secure');
          detailBuilder.push(urlAuth.label);
        }

        if (urlAuth.status !== UrlAuthenticationStatus.NoStatus) {
          detailBuilder.push(`(${urlAuth.status})`);
        }

        return <any>{
          // ui data
          label: urlAuth.url,
          detail: detailBuilder.join(' '),
          // selected item data
          _id: urlAuth.id
        };
      }
    );

    // determine which auth provider to use
    const selected = await this.window.showQuickPick(
      pickItems,
      {
        canPickMany: true,
        title: 'Clear url authentication',
        placeHolder: "Choose which urls to remove"
      }
    );

    if (selected === undefined) return [];

    // filter the url auth data by selected
    const results = urlAuthData.filter(authData => {
      return selected.some((item: any) => item._id === authData.id);
    });

    return results;
  }

  async promptRetry(message: string, detail: string): Promise<boolean | undefined> {
    const choice = await this.window.showInformationMessage(
      message,
      { modal: true, detail: detail },
      'Retry'
    );

    if (choice === undefined) return undefined;

    return true;
  }

  async promptYesCancel(message: string, detail: string = ""): Promise<boolean> {
    const choice = await this.window.showInformationMessage(
      message,
      { modal: true, detail: detail },
      'Yes'
    );

    if (choice === undefined) return false;

    return true;
  }

  private async promptUnsecured(url: string): Promise<boolean | undefined> {
    const choice = await this.window.showInformationMessage(
      `${url} is using the unsecured HTTP protocol.\n\n` +
      "Are you sure you want to send your credentials using this url?",
      { modal: true },
      'Yes'
    );

    if (choice === undefined) return undefined;

    return true;
  }

}