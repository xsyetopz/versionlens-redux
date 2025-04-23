import type { ILogger } from '#domain/logging';
import { PackageSourceType } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { SuggestionCodeLens } from '#extension/suggestions';
import type { IVsCodeConstructFactory, IVsCodeEnv, IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class OnFileLinkClick extends Disposable {

  constructor(
    readonly construct: IVsCodeConstructFactory,
    readonly window: IVsCodeWindow,
    readonly env: IVsCodeEnv,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('construct', construct);
    throwUndefinedOrNull('window', window);
    throwUndefinedOrNull('env', env);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Executes when a codelens file link suggestion is clicked
   * @param codeLens
   */
  async execute(codeLens: SuggestionCodeLens): Promise<void> {
    const filePath = codeLens.packageResponse.fetchedPackage!.version;
    if (codeLens.packageResponse.packageSource === PackageSourceType.Directory)
      // open folder
      await this.env.openExternal(`file:///${filePath}` as any);
    else
      // open file
      await this.window.showTextDocument(this.construct.createFileUri(filePath));
  }

}