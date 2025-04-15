import type { ILogger } from '#domain/logging';
import { OnErrorClick } from '#extension/events';
import type { VersionLensState } from '#extension/state';
import type { IVsCodeWindow } from '#extension/vscode';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify } from 'ts-mockito';
import type { OutputChannel } from 'vscode';

type TestContext = {
  mockWindow: IVsCodeWindow
  mockState: VersionLensState
  mockOutputChannel: OutputChannel
  mockLogger: ILogger
}

export const onErrorClickTests = {

  [test.title]: OnErrorClick.name,

  beforeEach: function (this: TestContext) {
    this.mockWindow = mock<IVsCodeWindow>();
    this.mockState = mock<VersionLensState>();
    this.mockOutputChannel = mock<OutputChannel>();
    this.mockLogger = mock<ILogger>();
  },

  "focuses error output and clears states": async function (this: TestContext) {
    const testEvent = new OnErrorClick(
      instance(this.mockWindow),
      instance(this.mockState),
      instance(this.mockOutputChannel),
      instance(this.mockLogger)
    );

    // test
    await testEvent.execute();

    // verify
    verify(this.mockOutputChannel.show()).once();
    verify(this.mockState.clearErrorState()).once();
    verify(this.mockState.clearBusyState()).once();
    verify(
      this.mockWindow.showTextDocument(this.mockWindow.activeTextEditor!.document)
    ).once();
  },

};