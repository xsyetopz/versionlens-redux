import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import { DependencyCache } from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';

export class OnPreSaveChanges {

  constructor(
    readonly fileWatcherDependencyCache: DependencyCache,
    readonly editorDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fileWatcherDependencyCache", fileWatcherDependencyCache);
    throwUndefinedOrNull("editorDependencyCache", editorDependencyCache);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(provider: ISuggestionProvider, packageFilePath: string): Promise<void> {
    // update the file watcher dependencies
    const deps = this.editorDependencyCache.get(provider.name, packageFilePath);
    this.fileWatcherDependencyCache.set(provider.name, packageFilePath, deps)

    // remove the packageFilePath from editor dependency cache
    this.editorDependencyCache.remove(provider.name, packageFilePath);
    this.logger.debug(
      'cleared editor dependency cache for %s',
      packageFilePath
    );
  }

}