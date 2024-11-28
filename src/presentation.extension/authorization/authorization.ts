import type { IAuthorization } from '#domain/authorization';
import type { ILogger } from '#domain/logging';
import {
  type AuthenticationInteractions,
  type IAuthenticationProviderFactory,
  type IVsCodeAuthentication,
  type UrlAuthenticationStore,
  AuthenticationScheme,
  createEmptyUrlAuthData
} from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class Authorization implements IAuthorization {

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

  isUrlAuthorized(url: string): boolean {
    const urlAuthInfo = this.urlAuthStore.get(url);
    if (urlAuthInfo === undefined) return false;
    return urlAuthInfo.scheme !== AuthenticationScheme.NotSet;
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

      this.logger.info(
        "Using [%s] authentication provider for %s",
        urlAuthInfo.label,
        url
      );

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
      this.urlAuthStore.update(url, authType);

    } catch (error) {
      this.logger.error(
        "Could not authenticate using '%s' for %s. %s",
        authType.label,
        url,
        error
      );
      this.urlAuthStore.update(url, createEmptyUrlAuthData(url));
    }

    return consent;
  }

}