import { IExpiryCache } from '#domain/caching';
import { ILogger } from '#domain/logging';
import { PackageCache } from '#domain/packages';
import { Disposable } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnClearCache extends Disposable {

  constructor(
    readonly packageCache: PackageCache,
    readonly shellCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("packageCache", packageCache);
    throwUndefinedOrNull("shellCache", shellCache);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Clears all suggestion provider caches
   */
  execute() {
    this.logger.debug("Clearing packages cache");
    this.packageCache.clear();
    this.shellCache.clear();
  }

}