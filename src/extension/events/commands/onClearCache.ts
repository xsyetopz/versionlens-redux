import type { IExpiryCache } from '#domain/caching';
import type { ILogger } from '#domain/logging';
import type { PackageCache } from '#domain/packages';
import { Disposable } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnClearCache extends Disposable {

  constructor(
    readonly packageCache: PackageCache,
    readonly shellCache: IExpiryCache,
    readonly urlRequestCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('packageCache', packageCache);
    throwUndefinedOrNull('shellCache', shellCache);
    throwUndefinedOrNull('urlRequestCache', urlRequestCache);
    throwUndefinedOrNull('logger', logger);
  }

  execute() {
    this.logger.debug("Clearing package caches");
    this.packageCache.clear();
    this.shellCache.clear();
    this.urlRequestCache.clear();
  }

}