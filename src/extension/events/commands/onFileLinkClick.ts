import type { ILogger } from '#domain/logging';
import { PackageSourceType } from '#domain/packages';
import { Disposable } from '#domain/utils';
import type { SuggestionCodeLens } from '#extension/suggestions';
import type { IVsCodeConstructFactory, IVsCodeEnv, IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Event handler for clicking a file link in a suggestion.
 */
export class OnFileLinkClick extends Disposable {

  /**
   * Initializes a new instance of the OnFileLinkClick class.
   * @param construct Factory for VS Code constructs.
   * @param window VS Code window interface.
   * @param env VS Code environment interface.
   * @param logger Logger instance.
   */
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
   * Executes when a codelens file link suggestion is clicked.
   * Opens the file or folder associated with the dependency.
   * @param codeLens The clicked code lens.
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