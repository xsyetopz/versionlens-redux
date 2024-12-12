import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnPreSaveChanges {

  constructor(
    readonly fileWatcherDependencyCache: DependencyCache,
    readonly editorDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('fileWatcherDependencyCache', fileWatcherDependencyCache);
    throwUndefinedOrNull('editorDependencyCache', editorDependencyCache);
    throwUndefinedOrNull('logger', logger);
  }

  async execute(provider: ISuggestionProvider, packageFilePath: string): Promise<void> {
    // update the file watcher dependencies
    const deps = this.editorDependencyCache.get(provider.name, packageFilePath);
    this.fileWatcherDependencyCache.set(provider.name, packageFilePath, deps)

    // remove the packageFilePath from editor dependency cache
    this.editorDependencyCache.remove(provider.name, packageFilePath);
    this.logger.debug(
      "cleared editor dependency cache for {packageFilePath}",
      packageFilePath
    );
  }

}