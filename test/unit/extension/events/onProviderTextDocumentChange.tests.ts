import type { ILogger } from '#domain/logging';
import type { DependencyCache, PackageDependency } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import type { DependencyChangesResult, GetDependencyChanges } from '#domain/useCases';
import { OnProviderTextDocumentChange } from '#extension/events';
import type { ContextState, VersionLensState } from '#extension/state';
import { test } from 'mocha-ui-esm';
import { instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockState: VersionLensState
  mockShowOutdated: ContextState<boolean>
  mockGetDependencyChanges: GetDependencyChanges
  mockEditorDependencyCache: DependencyCache
  mockLogger: ILogger
}

export const onProviderTextDocumentChangeTests = {

  [test.title]: OnProviderTextDocumentChange.name,

  beforeEach: function (this: TestContext) {
    this.mockState = mock<VersionLensState>();
    this.mockShowOutdated = mock<ContextState<boolean>>();
    this.mockGetDependencyChanges = mock<GetDependencyChanges>();
    this.mockEditorDependencyCache = mock<DependencyCache>();
    this.mockLogger = mock<ILogger>();

    when(this.mockState.showOutdated).thenReturn(instance(this.mockShowOutdated));
  },

  "updates editor cache with document changes": async function (this: TestContext) {
    const testProviderName = 'testProvider';
    const testParsedDeps: PackageDependency[] = [];

    const testProvider: ISuggestionProvider = { name: testProviderName } as any;
    const testPackageFilePath = 'test/path/dir';
    const testNewContent = '{}';
    const testHasChanged = true;
    const testDepChangedResult: DependencyChangesResult = {
      hasChanged: testHasChanged,
      parsedDependencies: testParsedDeps
    };
    const testEvent = new OnProviderTextDocumentChange(
      instance(this.mockState),
      instance(this.mockGetDependencyChanges),
      instance(this.mockEditorDependencyCache),
      instance(this.mockLogger)
    );

    when(this.mockGetDependencyChanges.execute(testProvider, testPackageFilePath, testNewContent))
      .thenResolve(testDepChangedResult);

    // test
    await testEvent.execute(testProvider, testPackageFilePath, testNewContent);

    // verify
    verify(
      this.mockLogger.trace(
        "{suggestionProviderName} provider text document change",
        testProviderName
      )
    ).once();
    verify(
      this.mockEditorDependencyCache.set(
        testProviderName,
        testPackageFilePath,
        testParsedDeps
      )
    ).once();
    verify(this.mockShowOutdated.change(testHasChanged)).once();
  },

};