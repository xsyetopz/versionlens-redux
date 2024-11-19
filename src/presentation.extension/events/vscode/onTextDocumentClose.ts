import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { TextDocument } from 'vscode';

export type ProviderTextDocumentClosedEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string
) => Promise<void>;

export class OnTextDocumentClose extends AsyncEmitter<ProviderTextDocumentClosedEvent> {

  constructor(
    readonly getSuggestionProvider: GetSuggestionProvider,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("getSuggestionProvider", getSuggestionProvider);
    throwUndefinedOrNull("logger", logger);
  }

  async execute(document: TextDocument): Promise<void> {
    // we can't check for an active provider here
    // because its already been de-activated before this event is called

    // ensure this is a file
    if (document.uri.scheme !== 'file') return;

    // attempt to match a provider file
    const provider = this.getSuggestionProvider.execute(document.fileName);
    if (!provider) return;

    // execute the listener
    await this.fire(provider, document.uri.fsPath);
  }

}