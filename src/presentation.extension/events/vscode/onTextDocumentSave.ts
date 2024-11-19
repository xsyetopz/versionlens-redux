import type { ILogger } from '#domain/logging';
import type { ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { AsyncEmitter } from '#domain/utils';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { TextDocument } from 'vscode';

export type ProviderTextDocumentSaveEvent = (
  provider: ISuggestionProvider,
  packageFilePath: string,
) => Promise<void>;

export class OnTextDocumentSave extends AsyncEmitter<ProviderTextDocumentSaveEvent> {

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

  async execute(document: TextDocument): Promise<void> {
    // ensure we have an active provider
    if (!this.state.providerActive.value) return;

    // get the provider
    const provider = this.getSuggestionProvider.execute(document.fileName);
    if (!provider) return;

    if (this.state.showOutdated.value) {
      await this.fire(provider as ISuggestionProvider, document.uri.fsPath);

      // reset outdated flag
      await this.state.showOutdated.change(false);
    }
  }

}