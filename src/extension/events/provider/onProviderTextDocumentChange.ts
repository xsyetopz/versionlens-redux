import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetDependencyChanges } from '#domain/useCases';
import type { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for when a provider-supported text document is changed.
 */
export class OnProviderTextDocumentChange {

  /**
   * Initializes a new instance of the OnProviderTextDocumentChange class.
   * @param state Extension state.
   * @param getDependencyChanges Use case for detecting dependency changes.
   * @param editorDependencyCache Cache for editor-based dependencies.
   * @param logger Logger instance.
   */
  constructor(
    readonly state: VersionLensState,
    readonly getDependencyChanges: GetDependencyChanges,
    readonly editorDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('getDependencyChanges', getDependencyChanges);
    throwUndefinedOrNull('editorDependencyCache', editorDependencyCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes when the document content changes.
   * Updates the editor dependency cache and the outdated state.
   * @param suggestionProvider The provider associated with the document.
   * @param packageFilePath The path to the package file.
   * @param newContent The updated content of the document.
   */
  async execute(suggestionProvider: ISuggestionProvider, packageFilePath: string, newContent: string): Promise<void> {
    this.logger.trace(
      "{suggestionProviderName} provider text document change",
      suggestionProvider.name
    );

    const result = await this.getDependencyChanges.execute(
      suggestionProvider,
      packageFilePath,
      newContent
    );

    // update the editor cache
    this.editorDependencyCache.set(
      suggestionProvider.name,
      packageFilePath,
      result.parsedDependencies
    );

    this.logger.trace("has changes = {changed}", result.hasChanged);
    await this.state.showOutdated.change(result.hasChanged);
  }

}