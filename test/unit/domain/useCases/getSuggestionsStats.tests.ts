import type { ILogger } from '#domain/logging';
import type { DependencyCache } from '#domain/packages';
import type { ISuggestionProvider } from '#domain/providers';
import { type GetSuggestions, GetSuggestionsStats } from '#domain/useCases';
import { deepEqual } from 'node:assert';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import Fixtures from './getSuggestionsStats.fixtures';

type TestContext = {
  mockDependencyCache: DependencyCache
  mockGetSuggestions: GetSuggestions
  mockLogger: ILogger
  testProviders: ISuggestionProvider[]
  useCase: GetSuggestionsStats
}

export const GetSuggestionsStatsTests = {

  title: GetSuggestionsStats.name,

  beforeEach: function (this: TestContext) {
    this.mockDependencyCache = mock<DependencyCache>()
    this.mockGetSuggestions = mock<GetSuggestions>()
    this.mockLogger = mock<ILogger>()
    this.testProviders = [{ name: 'test1' }, { name: 'test2' }, { name: 'test3' }] as any
    this.useCase = new GetSuggestionsStats(
      this.testProviders,
      instance(this.mockDependencyCache),
      instance(this.mockGetSuggestions),
      instance(this.mockLogger),
    )

    when(this.mockGetSuggestions.execute(anything(), anything(), anything(), anything()))
      .thenResolve([])
  },

  "returns empty array when no dependencies found": async function (this: TestContext) {
    when(this.mockDependencyCache.getFilePaths(anything())).thenReturn([])

    // test
    const actual = await this.useCase.execute(false);

    // assert
    verify(
      this.mockGetSuggestions.execute(anything(), anything(), anything(), anything())
    ).never();

    deepEqual(actual, [])
  },

  "returns empty array when no suggestions found": async function (this: TestContext) {
    const testFilePaths = ['test/path1']

    when(this.mockDependencyCache.getFilePaths(anything())).thenReturn(testFilePaths)

    // test
    const actual = await this.useCase.execute(false);

    // assert
    verify(
      this.mockGetSuggestions.execute(anything(), anything(), anything(), anything())
    ).times(3);

    deepEqual(actual, [])
  },

  "returns counts of stats": async function (this: TestContext) {
    const testFilePaths = ['test/path1']

    when(this.mockDependencyCache.getFilePaths(anything())).thenReturn(testFilePaths)
    when(this.mockGetSuggestions.execute(this.testProviders[0], anything(), anything(), false))
      .thenResolve(Fixtures.test)

    // test
    const actual = await this.useCase.execute(false);

    // assert
    verify(
      this.mockGetSuggestions.execute(anything(), anything(), anything(), anything())
    ).times(3);

    verify(
      this.mockLogger.debug("queueing suggestion stats for {PackageFilePath}", 'test/path1')
    ).times(3)

    deepEqual(actual, Fixtures.expected)
    deepEqual(this.useCase.cache.get('stats'), Fixtures.expected)
  }

}