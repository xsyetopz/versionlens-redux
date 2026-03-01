import type { IAuthorizer } from '#domain/authorization';
import type { ILogger } from '#domain/logging';
import type { KeyDictionary } from '#domain/utils';
import type { AuthenticationProvider } from '#extension/authorization';
import {
  type AuthenticationInteractions,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthLog,
  AuthPrompt,
  AuthenticationScheme,
  UrlAuthenticationStatus,
  createEmptyUrlAuthData
} from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { parse } from 'node:url';

/**
 * VS Code implementation of the IAuthorizer interface.
 */
export class Authorizer implements IAuthorizer {

  /**
   * Initializes a new instance of the Authorizer class.
   * @param urlAuthStore The store for URL authentication metadata.
   * @param providers The map of authentication providers.
   * @param interactions The user interaction handler.
   * @param logger The logger instance.
   */
  constructor(
    readonly urlAuthStore: UrlAuthenticationStore,
    readonly providers: KeyDictionary<AuthenticationProvider>,
    readonly interactions: AuthenticationInteractions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('urlAuthStore', urlAuthStore);
    throwUndefinedOrNull('providers', providers);
    throwUndefinedOrNull('interactions', interactions);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Checks if a URL has a valid authorization configuration.
   */
  hasAuthorizationUrl(url: string): boolean {
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo === undefined) return false;
    if (urlAuthInfo.scheme === AuthenticationScheme.NotSet) return false;
    if (urlAuthInfo.status !== UrlAuthenticationStatus.NoStatus) return false;

    return true;
  }

  /**
   * Resolves the canonical authorization URL for a given service URL.
   */
  getAuthorizationUrl(url: string): string {
    // look for url in the user defined registry urls
    const entries = this.urlAuthStore.getAll();
    const registryAuthUrl = entries
      .filter(x => url.toLowerCase().startsWith(x.url.toLowerCase()));
    if (registryAuthUrl.length > 0) return registryAuthUrl[0].url;

    // default to domain host
    const parsedBaseUrl = parse(url, false);
    return `${parsedBaseUrl.protocol}//${parsedBaseUrl.host}`;
  }

  /**
   * Gets the authorization token for a URL.
   */
  async getToken(url: string): Promise<string | undefined> {
    // get the persisted url auth info
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (!urlAuthInfo || urlAuthInfo.scheme === AuthenticationScheme.NotSet) {
      return undefined;
    }

    // attempt to get an existing provider session
    const accessToken = await this.providers[urlAuthInfo.scheme].get(urlAuthInfo.url);
    if (!accessToken) return undefined;

    this.logger.debug(AuthLog.authProviderInfo, urlAuthInfo.label, new URL(url));

    // return the authorization header value
    return urlAuthInfo.scheme === AuthenticationScheme.Custom
      ? accessToken
      : `${urlAuthInfo.scheme} ${accessToken}`;
  }

  /**
   * Prompts the user for credentials for a URL and a request URL.
   */
  async getCredentials(url: string, requestUrl: string): Promise<boolean> {
    // check url isn't already unconsented
    const existingUrlAuthData = this.urlAuthStore.get(url);
    if (existingUrlAuthData?.scheme === AuthenticationScheme.NotSet) {
      return false;
    }

    // confirm the authentication url
    const authUrl = await this.interactions.confirmAuthorziationUrl(url, requestUrl);
    if (authUrl === undefined) {
      // prevent re-prompting the user
      this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
      return false;
    }

    // prompt unsecure urls
    if (url.startsWith('https:') === false) {
      const allowUnsecured = await this.interactions.promptUnsecured(url);
      if (allowUnsecured === false) {
        // prevent re-prompting the user
        this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
        return false;
      }
    }

    // get the authentication type
    const urlAuthData = await this.interactions.chooseAuthenticationScheme(authUrl);
    if (urlAuthData === undefined) {
      // prevent re-prompting the user
      this.urlAuthStore.update(authUrl, createEmptyUrlAuthData(authUrl));
      return false;
    }

    return await this.authenticate(urlAuthData);
  }

  /**
   * Retries authentication for a URL.
   */
  async retryCredentials(url: string): Promise<boolean> {
    const urlAuthData = this.urlAuthStore.get(url);

    // prompt retry
    const retry = await this.interactions.promptYesCancel(
      AuthPrompt.couldNotAuthenticate(url)
    );

    if (retry === false) {
      // save 'failed credentials' data
      const failedAuthData = {
        ...urlAuthData,
        scheme: AuthenticationScheme.NotSet,
        status: UrlAuthenticationStatus.CredentialsFailed
      };
      await this.urlAuthStore.update(url, failedAuthData);
      return false;
    }

    return await this.authenticate(urlAuthData);
  }

  /**
   * Internal method to perform authentication using the selected scheme.
   */
  private async authenticate(urlAuthData: UrlAuthenticationData): Promise<boolean> {
    const didCreate = await this.providers[urlAuthData.scheme].create(urlAuthData.url);
    if (didCreate)
      // save completed data
      await this.urlAuthStore.update(urlAuthData.url, urlAuthData);
    else
      // save cancelled data
      await this.urlAuthStore.update(
        urlAuthData.url,
        createEmptyUrlAuthData(urlAuthData.url)
      );

    return didCreate;
  }

}