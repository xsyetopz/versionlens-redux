import type { IAuthorizer } from '#domain/authorization';
import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationData,
  type UrlAuthenticationStore,
  AuthLog,
  AuthPrompt,
  AuthenticationScheme,
  UrlAuthenticationStatus,
  createEmptyUrlAuthData
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { parse } from 'url';

export class Authorizer implements IAuthorizer {

  constructor(
    readonly interactions: AuthenticationInteractions,
    readonly urlAuthStore: UrlAuthenticationStore,
    readonly providerFactory: IAuthenticationProviderFactory,
    readonly authentication: IVsCodeAuthentication,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('interactions', interactions);
    throwUndefinedOrNull('urlAuthStore', urlAuthStore);
    throwUndefinedOrNull('providerFactory', providerFactory);
    throwUndefinedOrNull('authentication', authentication);
    throwUndefinedOrNull('logger', logger);
  }

  urlHasAuthConsent(url: string): boolean {
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo === undefined) return false;
    if (urlAuthInfo.scheme === AuthenticationScheme.NotSet) return false;
    if (urlAuthInfo.status !== UrlAuthenticationStatus.NoStatus) return false;

    return true;
  }

  getRegistryAuthUrl(url: string): string {
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

    // create the custom provider unless built-in
    if (['github', 'microsoft'].includes(urlAuthInfo.id) === false) {
      await this.providerFactory.registerCustomAuthProvider(urlAuthInfo.scheme, url);
    }

    try {
      // attempt to get an existing provider session
      const sessionInfo = await this.authentication.getSession(urlAuthInfo.id, []);
      if (!sessionInfo || !sessionInfo.accessToken) return undefined;

      this.logger.info(AuthLog.authProviderInfo, urlAuthInfo.label, url);

      // return the authorization header value
      return urlAuthInfo.scheme === AuthenticationScheme.Custom
        ? sessionInfo.accessToken
        : `${urlAuthInfo.scheme} ${sessionInfo.accessToken}`;
    }
    catch (e) { }

    return undefined;
  }

  async getConsent(url: string, requestUrl: string): Promise<boolean> {
    // check url isn't already unconsented
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo?.scheme === AuthenticationScheme.NotSet) {
      return false;
    }

    // confirm the authentication url
    const authUrl = await this.interactions.confirmAuthorziationUrl(url, requestUrl);
    if (authUrl === undefined) {
      // prevent re-prompting the user
      this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
      return false;
    }

    // get the authentication type
    const authData = await this.interactions.chooseAuthenticationType(authUrl);
    if (authData === undefined) {
      // prevent re-prompting the user
      this.urlAuthStore.update(authUrl, createEmptyUrlAuthData(authUrl));
      return false;
    }

    return await this.authenticate(authData);
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
    // ensure custom providers are registered
    if (urlAuthData.isCustomProvider) {
      await this.providerFactory.registerCustomAuthProvider(urlAuthData.scheme, urlAuthData.url);
    }

    // attempt to get a new session
    let consent: boolean = false;
    try {
      await this.authentication.getSession(urlAuthData.id, [], { forceNewSession: true });
      consent = true;
      // save the url auth data
      await this.urlAuthStore.update(urlAuthData.url, urlAuthData);
    } catch (error) {
      this.logger.error(
        AuthLog.couldNotAutheticateError,
        urlAuthData.label,
        urlAuthData.url,
        error
      );
      // save the unconsented auth data
      await this.urlAuthStore.update(urlAuthData.url, createEmptyUrlAuthData(urlAuthData.url));
    }

    return consent;
  }

}