import type { IExpiryCache } from '#domain/caching';
import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { VulnerabilityProvider } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for clearing extension caches.
 */
export class OnClearCache extends Disposable {

  /**
   * Initializes a new instance of the OnClearCache class.
   * @param packageCache Cache for package suggestions.
   * @param shellCache Cache for shell command results.
   * @param urlRequestCache Cache for URL-based requests.
   * @param vulnerabilityProvider The vulnerability diagnostic provider.
   * @param logger Logger instance.
   */
  constructor(
    readonly packageCache: PackageCache,
    readonly shellCache: IExpiryCache,
    readonly urlRequestCache: IExpiryCache,
    readonly vulnerabilityProvider: VulnerabilityProvider,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('packageCache', packageCache);
    throwUndefinedOrNull('shellCache', shellCache);
    throwUndefinedOrNull('urlRequestCache', urlRequestCache);
    throwUndefinedOrNull('vulnerabilityProvider', vulnerabilityProvider);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Clears all managed caches.
   */
  execute() {
    this.logger.debug("Clearing package caches");
    this.packageCache.clear();
    this.shellCache.clear();
    this.urlRequestCache.clear();
    this.vulnerabilityProvider.clear();
  }

}