import type { ILogger } from '#domain/logging';
import { PackageSourceType } from '#domain/packages';
import { OnFileLinkClick } from '#extension/events';
import type { IVsCodeConstructFactory, IVsCodeEnv, IVsCodeWindow } from '#extension/vscode';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify, when } from 'ts-mockito';
import type { Uri } from 'vscode';

type TestContext = {
  mockVsCodeConstruct: IVsCodeConstructFactory
  mockVsCodeWindow: IVsCodeWindow
  mockVsCodeEnv: IVsCodeEnv
  mockLogger: ILogger
  testEvent: OnFileLinkClick
}

export const onFileLinkClickTests = {

  [test.title]: OnFileLinkClick.name,

  beforeEach: function (this: TestContext) {
    this.mockVsCodeConstruct = mock<IVsCodeConstructFactory>();
    this.mockVsCodeWindow = mock<IVsCodeWindow>();
    this.mockVsCodeEnv = mock<IVsCodeEnv>();
    this.mockLogger = mock<ILogger>();
    this.testEvent = new OnFileLinkClick(
      instance(this.mockVsCodeConstruct),
      instance(this.mockVsCodeWindow),
      instance(this.mockVsCodeEnv),
      instance(this.mockLogger)
    );
  },

  "opens folders using env.openExternal": async function (this: TestContext) {
    const testFilePath = 'some/path/dir';
    // test
    await this.testEvent.execute({
      packageResponse: {
        packageSource: PackageSourceType.Directory,
        fetchedPackage: {
          version: testFilePath
        }
      }
    } as any);
    // verify
    verify(this.mockVsCodeEnv.openExternal(<any>`file:///${testFilePath}`)).once();
  },

  "opens files using window.showTextDocument": async function (this: TestContext) {
    // setup
    const testFilePath = 'some/path/dir/file.txt';
    const expected = instance(mock<Uri>())
    when(this.mockVsCodeConstruct.createFileUri(testFilePath)).thenReturn(expected);
    // test
    await this.testEvent.execute({
      packageResponse: {
        packageSource: PackageSourceType.File,
        fetchedPackage: {
          version: testFilePath
        }
      }
    } as any);
    // verify
    verify(this.mockVsCodeWindow.showTextDocument(expected)).once();
  },

};