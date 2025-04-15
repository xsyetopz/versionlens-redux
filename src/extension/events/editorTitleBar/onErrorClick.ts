import type { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import type { VersionLensState } from '#extension/state';
import type { IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { OutputChannel } from 'vscode';

export class OnErrorClick extends Disposable {

  constructor(
    readonly window: IVsCodeWindow,
    readonly state: VersionLensState,
    readonly outputChannel: OutputChannel,
    readonly logger: ILogger
  ) {
    super();
    throwUndefinedOrNull('window', window);
    throwUndefinedOrNull('state', state);
    throwUndefinedOrNull('outputChannel', outputChannel);
    throwUndefinedOrNull('logger', logger);
  }

  async execute(): Promise<void> {
    // show the version lens log window
    this.outputChannel.show();

    // clear the error and busy states
    await this.state.clearErrorState();
    await this.state.clearBusyState();

    // focus on the document unhide icons
    this.window.showTextDocument(this.window.activeTextEditor!.document);
  }

}