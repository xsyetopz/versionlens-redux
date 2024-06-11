import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from '#domain/logging';
import { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import { VersionLensState } from 'presentation.extension';
import { TextDocumentChangeEvent, TextDocumentChangeReason, workspace } from 'vscode';

export type ProviderTextDocumentChangeEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
  newContent: string
) => Promise<void>;

export class OnTextDocumentChange extends AsyncEmitter<ProviderTextDocumentChangeEvent> {

  constructor(
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly state: VersionLensState,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("getSuggestionProvider", getSuggestionProvider);
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("logger", logger);

    // register the vscode workspace event
    this.disposable = workspace.onDidChangeTextDocument(this.execute, this);
  }

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