import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import { type KeyDictionary, Disposable } from '#domain/utils';
import {
  type AuthenticationProvider,
  type UrlAuthenticationStore,
  AuthenticationInteractions,
  AuthenticationScheme
} from '#extension/authorization';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnRemoveUrlAuthentication extends Disposable {

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

  async execute() {
    // get all the url authentications
    const data = this.urlAuthStore.getAll();

    // sort the list
    data.sort();

    // prompt the user stored url auth data to remove
    const authDataToClear = await this.interactions.chooseUrlAuthToClear(data);
    if (authDataToClear.length === 0) return;

    // clear url authentication
    for (const authItem of authDataToClear) {
      this.logger.info(`Clearing {url} authentication`, new URL(authItem.url));

      // clear url auth persistence
      await this.urlAuthStore.remove(authItem.url);

      // clear secret auth persistence
      if (authItem.scheme !== AuthenticationScheme.NotSet) {
        await this.authProviders[authItem.scheme].remove(authItem.url);
      }
    }

    // clear package cache
    this.logger.info('Clearing package caches');
    this.packageCache.clear();
  }

}