import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for when a provider-supported text document is closed.
 */
export class OnProviderTextDocumentClose {

  /**
   * Initializes a new instance of the OnProviderTextDocumentClose class.
   * @param editorDependencyCache Cache for editor-based dependencies.
   * @param logger Logger instance.
   */
  constructor(
    readonly editorDependencyCache: DependencyCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("editorDependencyCache", editorDependencyCache);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when the document is closed.
   * Clears the editor-based dependency cache for the file.
   * @param provider The provider associated with the document.
   * @param packageFilePath The path to the closed package file.
   */
  async execute(provider: ISuggestionProvider, packageFilePath: string): Promise<void> {
    // remove the packageFilePath from editor dependency cache
    this.editorDependencyCache.remove(provider.name, packageFilePath);

    this.logger.debug(
      'cleared editor dependency cache for {packageFilePath}',
      packageFilePath
    );
  }

}