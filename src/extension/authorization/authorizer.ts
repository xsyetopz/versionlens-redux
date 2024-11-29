import type { IAuthorizer } from '#domain/authorization';
import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type UrlAuthenticationStore,
  AuthLog,
  AuthPrompt,
  AuthenticationScheme,
  UrlAuthenticationStatus,
  createEmptyUrlAuthData
} from '#extension/authorization';
import type { IVsCodeAuthentication } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

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

  async getConsent(url: string): Promise<boolean> {
    // check url isn't already unconsented
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo?.scheme === AuthenticationScheme.NotSet) {
      return false;
    }

    // get the authentication type
    const authType = await this.interactions.chooseAuthenticationType(url);
    if (authType === undefined) {
      this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
      return false;
    }

    // ensure custom providers are registered
    if (authType.isCustomProvider) {
      await this.providerFactory.registerCustomAuthProvider(authType.scheme, url);
    }

    // check the user has given consent
    let consent: boolean = false;
    try {
      await this.authentication.getSession(authType.id, [], { forceNewSession: true });
      consent = true;

      // persist the url auth type
      await this.urlAuthStore.update(url, authType);
    } catch (error) {
      this.logger.error(
        AuthLog.couldNotAutheticateError,
        authType.label,
        url,
        error
      );
      await this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
    }

    return consent;
  }

  async retryCredentials(url: string): Promise<boolean> {
    const retry = await this.interactions.promptYesCancel(
      AuthPrompt.couldNotAuthenticate(url)
    );
    if (retry === false) {
      // save 'failed credentials' data
      const urlAuthData = this.urlAuthStore.get(url);
      const failedAuthData = {
        ...urlAuthData,
        scheme: AuthenticationScheme.NotSet,
        status: UrlAuthenticationStatus.CredentialsFailed
      };
      await this.urlAuthStore.update(url, failedAuthData);
      return false;
    }

    // remove url auth data for re-attempt
    this.urlAuthStore.remove(url);

    return true;
  }

}