import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import type { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import type { VersionLensState } from '#extension/state';
import { TextDocumentChangeReason } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { TextDocumentChangeEvent } from 'vscode';

/**
 * Event signature for when a provider-supported document content changes.
 */
export type ProviderTextDocumentChangeEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
  newContent: string
) => Promise<void>;

/**
 * Handles the VS Code text document change event.
 */
export class OnTextDocumentChange extends AsyncEmitter<ProviderTextDocumentChangeEvent> {

  /**
   * Initializes a new instance of the OnTextDocumentChange class.
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
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when a document's content changes.
   * Filters for relevant changes and fires the provider-specific event.
   * @param e The VS Code text document change event.
   */
  async execute(e: TextDocumentChangeEvent): Promise<void> {
    // ensure we have an active provider
    if (!this.state.providerActive.value) return;

    // check if we have a change
    const shouldHandleEvent = e.reason == TextDocumentChangeReason.Redo
      || e.reason == TextDocumentChangeReason.Undo
      || e.contentChanges.length > 0

    if (shouldHandleEvent == false) return;

    // get the provider
    const provider = this.getSuggestionProvider.execute(e.document.fileName);
    if (!provider) return;

    // execute the listener
    await this.fire(
      provider,
      e.document.uri.fsPath,
      e.document.getText()
    );
  }

}