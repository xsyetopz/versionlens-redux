import { throwUndefinedOrNull } from '@esm-test/guards';
import { IExpiryCache } from '#domain/caching';
import { ILogger } from '#domain/logging';
import { PackageCache } from '#domain/packages';
import { Disposable } from '#domain/utils';
import { SuggestionCommandFeatures } from 'presentation.extension';
import { commands } from 'vscode';

export class OnClearCache extends Disposable {

  constructor(
    readonly packageCache: PackageCache,
    readonly processesCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("packageCache", packageCache);
    throwUndefinedOrNull("processesCache", processesCache);
    throwUndefinedOrNull("logger", logger);

    // register the vscode command
    this.disposable = commands.registerCommand(
      SuggestionCommandFeatures.OnClearCache,
      this.execute,
      this
    );
  }

  /**
   * Clears all suggestion provider caches
   */
  execute() {
    this.logger.debug("Clearing packages cache");
    this.packageCache.clear();
    this.processesCache.clear();
  }

}