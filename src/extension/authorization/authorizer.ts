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
import { parse } from 'url';

export class Authorizer implements IAuthorizer {

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

  hasAuthorizationUrl(url: string): boolean {
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo === undefined) return false;
    if (urlAuthInfo.scheme === AuthenticationScheme.NotSet) return false;
    if (urlAuthInfo.status !== UrlAuthenticationStatus.NoStatus) return false;

    return true;
  }

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

  async getToken(url: string): Promise<string | undefined> {
    // get the persisted url auth info
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (!urlAuthInfo || urlAuthInfo.scheme === AuthenticationScheme.NotSet) {
      return undefined;
    }

    // attempt to get an existing provider session
    const accessToken = await this.providers[urlAuthInfo.scheme].get(urlAuthInfo.url);
    if (!accessToken) return undefined;

    this.logger.info(AuthLog.authProviderInfo, urlAuthInfo.label, url);

    // return the authorization header value
    return urlAuthInfo.scheme === AuthenticationScheme.Custom
      ? accessToken
      : `${urlAuthInfo.scheme} ${accessToken}`;
  }

  async getConsent(url: string, requestUrl: string): Promise<boolean> {
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

  private async authenticate(urlAuthData: UrlAuthenticationData): Promise<boolean> {
    const didCreate = await this.providers[urlAuthData.scheme].create(urlAuthData.url);
    if (didCreate)
      // save the url auth data
      await this.urlAuthStore.update(urlAuthData.url, urlAuthData);
    else
      // save as unconsented auth data
      await this.urlAuthStore.update(urlAuthData.url, createEmptyUrlAuthData(urlAuthData.url));

    return didCreate;
  }

}