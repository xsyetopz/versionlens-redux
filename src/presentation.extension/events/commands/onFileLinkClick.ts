import { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import { SuggestionCodeLens } from '#extension/suggestions';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { env } from 'vscode';

export class OnFileLinkClick extends Disposable {

  constructor(readonly logger: ILogger) {
    super();
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes when a codelens file link suggestion is clicked
   * @param codeLens
   */
  async execute(codeLens: SuggestionCodeLens): Promise<void> {
    const filePath = codeLens.packageResponse.fetchedPackage.version;
    await env.openExternal(<any>('file:///' + filePath));
  }

}