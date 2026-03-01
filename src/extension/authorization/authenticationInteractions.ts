import { trimEndSlash } from '#domain/utils';
import {
  type UrlAuthenticationData,
  AuthPrompt,
  AuthenticationScheme,
  UrlAuthenticationStatus,
  authenticationProviders,
  basicAuthPrompt,
  chooseAuthSchemePrompt,
  confirmAuthUrlPrompt,
  createUrlAuthData
} from '#extension/authorization';
import type { IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { QuickPickItem } from 'vscode';

/**
 * Quick pick item for selecting an authentication provider.
 */
export type ProviderQuickPickItem = QuickPickItem & {
  /** The display label for the provider. */
  providerLabel: string,
  /** The authentication scheme supported by the provider. */
  providerScheme: AuthenticationScheme
}

/**
 * Handles user interactions for authentication, such as showing input boxes and quick picks.
 */
export class AuthenticationInteractions {

  /**
   * Initializes a new instance of the AuthenticationInteractions class.
   * @param window The VS Code window interface.
   */
  constructor(readonly window: IVsCodeWindow) {
    throwUndefinedOrNull('window', window);
  }

  /**
   * Prompts the user to enter an authorization URL.
   * @returns A promise resolving to the entered URL, or undefined if cancelled.
   */
  async enterAuthorizationUrl(): Promise<string | undefined> {
    const authUrl = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: confirmAuthUrlPrompt.enterAuthorizationUrl,
      placeHolder: 'Authorization url'
    });

    // check the user entered a value
    if (!authUrl) return undefined;

    // check url is (some what) valid
    new URL(authUrl);

    // remove any end slashes
    return trimEndSlash(authUrl);
  }

  /**
   * Prompts the user to confirm or modify an authorization URL for a specific request.
   * @param url The suggested authorization URL.
   * @param requestUrl The original request URL.
   * @returns A promise resolving to the confirmed URL, or undefined if cancelled.
   */
  async confirmAuthorziationUrl(url: string, requestUrl: string): Promise<string | undefined> {
    const inputUrl = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: confirmAuthUrlPrompt.enterAuthorizationUrl,
      placeHolder: 'Authorization url',
      value: url
    });
    // check the user entered a value
    if (!inputUrl) return undefined;

    // remove any end slashes
    const authUrl = trimEndSlash(inputUrl);

    // check the authUrl matches the original url domain
    const parsedRequestUrl = new URL(requestUrl);
    const parsedAuthUrl = new URL(authUrl);
    if (parsedAuthUrl.host !== parsedRequestUrl.host) {
      const retry = await this.promptRetry(confirmAuthUrlPrompt.differentDomain);
      return retry
        ? await this.confirmAuthorziationUrl(authUrl, requestUrl)
        : undefined;
    }

    // check the requestUrl starts with the auth url
    if (requestUrl.startsWith(authUrl) === false) {
      const retry = await this.promptRetry(
        confirmAuthUrlPrompt.urlPartialMismatch(requestUrl)
      );
      return retry
        ? await this.confirmAuthorziationUrl(authUrl, requestUrl)
        : undefined;
    }

    return authUrl;
  }

  /**
   * Prompts the user to choose an authentication scheme for a URL.
   * @param url The URL to authenticate.
   * @returns A promise resolving to the selected authentication data, or undefined if cancelled.
   */
  async chooseAuthenticationScheme(url: string): Promise<UrlAuthenticationData | undefined> {
    const pickItems: ProviderQuickPickItem[] = Array.from(
      authenticationProviders,
      authProviderInfo => ({
        // ui data
        label: authProviderInfo.label,
        detail: authProviderInfo.description,

        // selected item data
        providerLabel: authProviderInfo.label,
        providerScheme: authProviderInfo.scheme
      })
    );

    // determine which auth provider to use
    const selectedQuickPick = await this.window.showQuickPick(
      pickItems,
      {
        title: chooseAuthSchemePrompt.chooseAuthenticationScheme(url),
        placeHolder: "Choose an authentication provider"
      }
    );

    // check the user made a selection
    if (!selectedQuickPick) return undefined;

    // map to result
    return createUrlAuthData(
      url,
      selectedQuickPick.providerScheme,
      selectedQuickPick.providerLabel,
      UrlAuthenticationStatus.NoStatus
    );
  }

  /**
   * Prompts the user for basic authentication credentials.
   * @param url The URL to authenticate.
   * @returns A promise resolving to the base64 encoded credentials, or undefined if cancelled.
   */
  async enterBasicAuthDetails(url: string): Promise<string | undefined> {
    // prompt for the username
    const username = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: basicAuthPrompt.enterBasicAuthUsername(url),
      placeHolder: 'Basic auth username',
      password: false
    });
    if (username === undefined) return undefined;

    // validate username
    if (username.includes(':')) {
      const retry = await this.promptRetry(basicAuthPrompt.invalidBasicAuthUsername);
      if (retry === false) return undefined;

      return await this.enterBasicAuthDetails(url);
    }

    // prompt for the password
    const password = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: basicAuthPrompt.enterBasicAuthPassword(url),
      placeHolder: 'Basic auth password',
      password: true,
    });
    if (password === undefined) return undefined;

    // encode username:password
    return btoa(`${username}:${password}`);
  }

  /**
   * Prompts the user for a custom authorization value.
   * @param url The URL to authenticate.
   * @returns A promise resolving to the entered value, or undefined if cancelled.
   */
  async enterCustomAuthValue(url: string): Promise<string | undefined> {
    // prompt for the value
    const value = await this.window.showInputBox({
      ignoreFocusOut: true,
      prompt: `Enter the authorization value for ${url}`,
      placeHolder: 'Authorization value',
      password: true
    });

    return value ? value : undefined;
  }

  /**
   * Prompts the user to select which stored credentials to clear.
   * @param urlAuthData The list of stored authentication data.
   * @returns A promise resolving to the list of selected data to clear.
   */
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

        return {
          // ui data
          label: urlAuth.url,
          detail: detailBuilder.join(' '),
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
      return selected.some(item => item.label === authData.url);
    });

    return results;
  }

  /**
   * Prompts the user to retry a failed operation.
   * @param message The message to display.
   * @param detail Optional detail for the message.
   * @returns A promise resolving to true if the user chose to retry, otherwise false.
   */
  async promptRetry(message: string, detail: string = ""): Promise<boolean> {
    const choice = await this.window.showInformationMessage(
      message,
      { modal: true, detail },
      'Retry'
    );

    return !!choice;
  }

  /**
   * Prompts the user with a Yes/Cancel question.
   * @param message The message to display.
   * @param detail Optional detail for the message.
   * @returns A promise resolving to true if the user chose Yes, otherwise false.
   */
  async promptYesCancel(message: string, detail: string = ""): Promise<boolean> {
    const choice = await this.window.showInformationMessage(
      message,
      { modal: true, detail },
      'Yes'
    );

    return !!choice;
  }

  /**
   * Warns the user about using an unsecure (HTTP) URL for authentication.
   * @param url The URL in question.
   * @returns A promise resolving to true if the user chose to proceed, otherwise false.
   */
  async promptUnsecured(url: string): Promise<boolean> {
    const choice = await this.window.showWarningMessage(
      AuthPrompt.unsecureAuthorizationUrl(url),
      { modal: true },
      'Yes'
    );

    return !!choice;
  }

}