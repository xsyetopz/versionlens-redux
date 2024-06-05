import { throwUndefinedOrNull } from '@esm-test/guards';
import { ILogger } from 'domain/logging';
import { Disposable } from 'domain/utils';
import { IconCommandFeatures, VersionLensState } from 'presentation.extension';
import { OutputChannel, commands, window } from 'vscode';

export class OnErrorClick extends Disposable {

  constructor(
    readonly state: VersionLensState,
    readonly outputChannel: OutputChannel,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull("state", state);
    throwUndefinedOrNull("outputChannel", outputChannel);
    throwUndefinedOrNull("logger", logger);

    // register the vscode commands
    this.disposable = commands.registerCommand(
      IconCommandFeatures.ShowError,
      this.execute,
      this
    );
  }

  async execute(): Promise<void> {
    // show the version lens log window
    this.outputChannel.show();

    // clear the error and busy states
    await this.state.clearErrorState();
    await this.state.clearBusyState();

    // focus on the document unhide icons
    window.showTextDocument(window.activeTextEditor.document);
  }

}