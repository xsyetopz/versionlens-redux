import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { TextDocument } from 'vscode';

/**
 * Event signature for when a provider-supported document is saved.
 */
export type ProviderTextDocumentSaveEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
) => Promise<void>;

/**
 * Handles the VS Code text document save event.
 */
export class OnTextDocumentSave extends AsyncEmitter<ProviderTextDocumentSaveEvent> {

  /**
   * Initializes a new instance of the OnTextDocumentSave class.
   * @param getSuggestionProvider Use case for identifying suggestion providers.
   * @param state Extension state.
   * @param logger Logger instance.
   */
  constructor(
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("getSuggestionProvider", getSuggestionProvider);
    throwUndefinedOrNull("state", state)
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when a document is saved.
   * Fires the provider-specific save event if the document was supported and is outdated.
   * @param document The saved VS Code text document.
   */
  async execute(document: TextDocument): Promise<void> {
    // ensure we have an active provider
    if (!this.state.providerActive.value) return;

    // get the provider
    const provider = this.getSuggestionProvider.execute(document.fileName);
    if (!provider) return;

    if (this.state.showOutdated.value) {
      await this.fire(provider as ISuggestionProvider, document.uri.fsPath);
    }
  }

}