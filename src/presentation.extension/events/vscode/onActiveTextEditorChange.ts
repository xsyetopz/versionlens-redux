import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import { ISuggestionProvider } from 'domain/providers';
import { GetSuggestionProvider } from 'domain/useCases';
import { AsyncEmitter } from 'domain/utils';
import { TextDocument, TextEditor, window } from 'vscode';
import { VersionLensState } from '../../state/versionLensState';

export type ProviderEditorActivatedEvent = (
  activeProvider: ISuggestionProvider,
  document: TextDocument,
) => Promise<void>;

export class OnActiveTextEditorChange extends AsyncEmitter<ProviderEditorActivatedEvent> {

  constructor(
    readonly state: VersionLensState,
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("getSuggestionProvider", getSuggestionProvider);
    throwUndefinedOrNull("logger", logger);

    // register the vscode editor event
    this.disposable = window.onDidChangeActiveTextEditor(this.execute, this);
  }

  /**
  * Maintains the versionLens.providerActive state
  * each time the active editor changes
  * @param textEditor
  * @returns
  */
  async execute(textEditor?: TextEditor): Promise<void> {
    if (!textEditor || textEditor.document.uri.scheme !== 'file') {
      // disable icons when no editor
      await this.state.providerActive.change(null);
      return;
    }

    // get the active providers
    const activeProvider = this.getSuggestionProvider.execute(textEditor.document.fileName);
    if (!activeProvider) {
      // disable icons if no matches found
      await this.state.providerActive.change(null);
      return;
    }

    // update provider active state to show icons
    await this.state.providerActive.change(activeProvider.name);

    // fire activated event
    await this.fire(activeProvider, textEditor.document);
  }

}