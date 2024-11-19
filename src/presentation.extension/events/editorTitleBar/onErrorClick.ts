import type { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import { VersionLensState } from '#extension/state';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { type OutputChannel, window } from 'vscode';

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