import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import { SuggestionCodeLens, SuggestionCommandContributions } from 'presentation.extension';
import { Disposable, commands, env } from 'vscode';

export class OnFileLinkClick {

  constructor(readonly logger: ILogger) {
    throwUndefinedOrNull("logger", logger);

    // register the vscode command
    this.disposable = commands.registerCommand(
      SuggestionCommandContributions.OnFileLinkClick,
      this.execute,
      this
    );
  }

  disposable: Disposable;

  /**
   * Executes when a codelens file link suggestion is clicked
   * @param codeLens
   */
  async execute(codeLens: SuggestionCodeLens): Promise<void> {
    const filePath = codeLens.package.fetchedPackage.version;
    await env.openExternal(<any>('file:///' + filePath));
  }

  async dispose() {
    this.disposable.dispose();
    this.logger.debug("disposed");
  }

}