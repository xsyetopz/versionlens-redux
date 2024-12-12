import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { OnPreSaveChanges } from '#extension/events';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockFileWatcherDependencyCache: DependencyCache
  mockEditorDependencyCache: DependencyCache
  mockLogger: ILogger
}

export const onPreSaveChangesTests = {

  [test.title]: OnPreSaveChanges.name,

  beforeEach: function (this: TestContext) {
    this.mockFileWatcherDependencyCache = mock<DependencyCache>();
    this.mockEditorDependencyCache = mock<DependencyCache>();
    this.mockLogger = mock<ILogger>();
  },

  "updates file watcher dependency cache": async function (this: TestContext) {
    const testProviderName = 'testProvider';
    const testPackageFilePath = 'test/path/dir';
    const testProvider: ISuggestionProvider = { name: testProviderName } as any;
    const testPackageDeps = [];
    const testEvent = new OnPreSaveChanges(
      instance(this.mockFileWatcherDependencyCache),
      instance(this.mockEditorDependencyCache),
      instance(this.mockLogger)
    );

    when(this.mockEditorDependencyCache.get(testProviderName, testPackageFilePath))
      .thenReturn(testPackageDeps);

    // test
    await testEvent.execute(testProvider, testPackageFilePath);

    // verify
    verify(
      this.mockEditorDependencyCache.get(
        testProviderName,
        testPackageFilePath
      )
    ).once();
    verify(
      this.mockFileWatcherDependencyCache.set(
        testProviderName,
        testPackageFilePath,
        testPackageDeps
      )
    ).once();
    verify(
      this.mockEditorDependencyCache.remove(
        testProviderName,
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