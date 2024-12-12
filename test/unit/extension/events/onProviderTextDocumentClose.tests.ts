import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { OnProviderTextDocumentClose } from '#extension/events';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify } from 'ts-mockito';

type TestContext = {
  mockEditorDependencyCache: DependencyCache
  mockLogger: ILogger
}

export const onProviderTextDocumentCloseTests = {

  [test.title]: OnProviderTextDocumentClose.name,

  beforeEach: function (this: TestContext) {
    this.mockEditorDependencyCache = mock<DependencyCache>();
    this.mockLogger = mock<ILogger>();
  },

  "removes packageFile from editor cache": async function (this: TestContext) {
    const testProviderName = 'testProvider';
    const testPackageFilePath = 'test/path/dir';
    const testProvider: ISuggestionProvider = { name: testProviderName } as any;
    const testEvent = new OnProviderTextDocumentClose(
      instance(this.mockEditorDependencyCache),
      instance(this.mockLogger)
    );

    // test
    await testEvent.execute(testProvider, testPackageFilePath);

    // verify
    verify(
      this.mockEditorDependencyCache.remove(
        testProvider.name,
        testPackageFilePath
      )
    ).once();
    verify(
      this.mockLogger.debug(
        'cleared editor dependency cache for {packageFilePath}',
        testPackageFilePath
      )
    ).once();
  },

};