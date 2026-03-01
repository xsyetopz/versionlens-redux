import type { ILogger } from '#domain/logging';
import { type DependencyCache, type PackageDependency, hasPackageDepsChanged } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { IStorage } from '#domain/storage';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Result of the dependency change detection.
 */
export type DependencyChangesResult = {
  /** Whether any dependencies have changed. */
  hasChanged: boolean,
  /** The new list of parsed dependencies. */
  parsedDependencies: PackageDependency[]
}

/**
 * Use case for detecting changes in package dependencies.
 */
export class GetDependencyChanges {

  /**
   * Initializes a new instance of the GetDependencyChanges class.
   * @param storage The storage provider for reading file content.
   * @param fileWatcherDependencyCache The cache for previous dependency states.
   * @param logger The logger to use.
   */
  constructor(
    readonly storage: IStorage,
    readonly fileWatcherDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("fileWatcherDependencyCache", fileWatcherDependencyCache);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes the dependency change detection.
   * @param suggestionProvider The suggestion provider for the file.
   * @param packageFilePath The path to the package file.
   * @param fileContent Optional file content (avoids re-reading from storage).
   * @returns A promise resolving to the change detection result.
   */
  async execute(
    suggestionProvider: ISuggestionProvider,
    packageFilePath: string,
    fileContent?: string
  ): Promise<DependencyChangesResult> {
    // get the cached parsed dependencies
    const currentDeps = this.fileWatcherDependencyCache.get(
      suggestionProvider.name,
      packageFilePath
    ) || [];

    // parse dependencies from the file content 
    const content = fileContent
      ? fileContent
      : await this.storage.readFile(packageFilePath);

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