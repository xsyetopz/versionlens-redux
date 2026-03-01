import type { ILogger } from '#domain/logging';
import { Disposable } from '#domain/utils';
import type { VersionLensState } from '#extension/state';
import type { IVsCodeWindow } from '#extension/vscode';
import { throwUndefinedOrNull } from '@esm-test/guards';
import type { OutputChannel } from 'vscode';

/**
 * Event handler for clicking the error icon in the editor title bar.
 */
export class OnErrorClick extends Disposable {

  /**
   * Initializes a new instance of the OnErrorClick class.
   * @param window VS Code window interface.
   * @param state Extension state.
   * @param outputChannel The log output channel to show.
   * @param logger Logger instance.
   */
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

  /**
   * Executes when the error icon is clicked.
   * Shows the log channel and clears error/busy states.
   */
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