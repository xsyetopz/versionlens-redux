import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetDependencyChanges } from '#domain/useCases';
import type { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnProviderTextDocumentChange {

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