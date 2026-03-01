import type { ILogger } from '#domain/logging';
import { PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for when package dependencies change in a watched file.
 */
export class OnPackageDependenciesChanged {

  /**
   * Initializes a new instance of the OnPackageDependenciesChanged class.
   * @param state Extension state.
   * @param logger Logger instance.
   */
  constructor(
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when dependencies change.
   * Currently a placeholder for future implementation.
   * @param provider The suggestion provider.
   * @param packageFilePath The path to the package file.
   * @param newDependencies The new list of dependencies.
   */
  async execute(
    provider: ISuggestionProvider,
    packageFilePath: string,
    newDependencies: PackageDependency[]
  ): Promise<void> {


  }

}