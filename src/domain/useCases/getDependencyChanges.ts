import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import { DependencyCache, PackageDependency, hasPackageDepsChanged } from '#domain/packages';
import { ISuggestionProvider } from 'domain/providers';
import { IStorage } from 'domain/storage';

export type DependencyChangesResult = {
  hasChanged: boolean,
  parsedDependencies: PackageDependency[]
}

export class GetDependencyChanges {

  constructor(
    readonly storage: IStorage,
    readonly fileWatcherDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fileWatcherDependencyCache", fileWatcherDependencyCache);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(
    suggestionProvider: ISuggestionProvider,
    packageFilePath: string,
    fileContent: string = undefined
  ): Promise<DependencyChangesResult> {
    // get the cached parsed dependencies
    const currentDeps = this.fileWatcherDependencyCache.get(
      suggestionProvider.name,
      packageFilePath
    ) || [];

    // parse dependencies from the file content 
    const content = fileContent ?
      fileContent :
      await this.storage.readFile(packageFilePath);

    const parsedDependencies = suggestionProvider.parseDependencies(packageFilePath, content);

    // check if there is a change
    const hasChanged = hasPackageDepsChanged(currentDeps, parsedDependencies);

    // return the parsed dependencies and changed state
    return {
      parsedDependencies,
      hasChanged,
    };
  }

}