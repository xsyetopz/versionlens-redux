import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import { type KeyDictionary, Disposable } from '#domain/utils';
import {
  type AuthenticationProvider,
  type UrlAuthenticationStore,
  AuthenticationInteractions,
  createEmptyUrlAuthData
} from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for adding URL-based authentication.
 */
export class OnAddUrlAuthentication extends Disposable {

  /**
   * Initializes a new instance of the OnAddUrlAuthentication class.
   * @param authProviders Map of authentication providers.
   * @param urlAuthStore Store for URL authentication metadata.
   * @param packageCache Cache for package suggestions.
   * @param interactions UI interactions handler.
   * @param logger Logger instance.
   */
  constructor(
    readonly authProviders: KeyDictionary<AuthenticationProvider>,
    readonly urlAuthStore: UrlAuthenticationStore,
    readonly packageCache: PackageCache,
    readonly interactions: AuthenticationInteractions,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('authProviders', authProviders);
    throwUndefinedOrNull('urlAuthStore', urlAuthStore);
    throwUndefinedOrNull('packageCache', packageCache);
    throwUndefinedOrNull('interactions', interactions);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes the add authentication workflow.
   * Prompts the user for URL, scheme, and credentials, then clears the package cache.
   */
  async execute() {
    // prompt for the authorization url
    const authUrl = await this.interactions.enterAuthorizationUrl();
    if (authUrl === undefined) return;

    // prompt unsecure urls
    if (authUrl.startsWith('https:') === false) {
      const allowUnsecured = await this.interactions.promptUnsecured(authUrl);
      if (allowUnsecured === false) return;
    }

    // prompt for scheme
    const urlAuthData = await this.interactions.chooseAuthenticationScheme(authUrl);
    if (urlAuthData === undefined) return;

    // prompt for provider credentials
    const didCreate = await this.authProviders[urlAuthData.scheme].create(
      urlAuthData.url
    );

    if (didCreate)
      // save completed data
      await this.urlAuthStore.update(urlAuthData.url, urlAuthData);
    else
      // save cancelled data
      await this.urlAuthStore.update(
        urlAuthData.url,
        createEmptyUrlAuthData(urlAuthData.url)
      );

    // clear package cache
    this.logger.info('Clearing package caches');
    this.packageCache.clear();
  }

}