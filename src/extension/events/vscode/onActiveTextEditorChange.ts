import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { TextDocument, TextEditor } from 'vscode';

/**
 * Event signature for when a provider-supported editor is activated.
 */
export type ProviderEditorActivatedEvent = (
  activeProvider: ISuggestionProvider,
  document: TextDocument,
) => Promise<void>;

/**
 * Handles the VS Code active text editor change event.
 */
export class OnActiveTextEditorChange extends AsyncEmitter<ProviderEditorActivatedEvent> {

  /**
   * Initializes a new instance of the OnActiveTextEditorChange class.
   * @param state Extension state.
   * @param getSuggestionProvider Use case for identifying suggestion providers.
   * @param logger Logger instance.
   */
  constructor(
    readonly state: VersionLensState,
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("getSuggestionProvider", getSuggestionProvider);
    throwUndefinedOrNull("logger", logger);
  }

  /**
  * Maintains the versionLens.providerActive state each time the active editor changes.
  * Fires the activated event if a provider matches the new editor.
  * @param textEditor The newly active text editor.
  */
  async execute(textEditor?: TextEditor): Promise<void> {
    if (!textEditor || textEditor.document.uri.scheme !== 'file') {
      // disable icons when no editor
      await this.state.providerActive.change(null);
      await this.state.showCustomInstall.change(false);
      await this.state.showSortAlphabetically.change(false);
      return;
    }

    // get the active providers
    const activeProvider = this.getSuggestionProvider.execute(textEditor.document.fileName);
    if (!activeProvider) {
      // disable icons if no matches found
      await this.state.providerActive.change(null);
      await this.state.showCustomInstall.change(false);
      await this.state.showSortAlphabetically.change(false);
      return;
    }

    // update provider active state to show icons
    await this.state.providerActive.change(activeProvider.name);

    // defrost configuration to ensure we have the latest values
    this.state.suggestionOptions.defrost();

    // update custom install state
    const hasCustomInstall = !!activeProvider.config.onSaveChangesTask;
    await this.state.showCustomInstall.change(hasCustomInstall);

    // update sort alphabetically state
    const showSortAlphabetically = activeProvider.config.dependencyProperties?.length !== 0;
    await this.state.showSortAlphabetically.change(showSortAlphabetically);

    // fire activated event
    await this.fire(activeProvider, textEditor.document);
  }

}
