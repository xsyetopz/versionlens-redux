import type { ILogger } from '#domain/logging';
import type { GetSuggestionsStats } from '#domain/useCases';
import type { IContextState, IVersionLensState } from '#extension';
import { OnRefreshSuggestionsStats } from '#extension/events';
import type { SuggestionsOptions } from '#extension/suggestions';
import { equal } from 'assert';
import { test } from 'mocha-ui-esm';
import { anyNumber, instance, mock, verify, when } from 'ts-mockito';
import type { StatusBarItem } from 'vscode';

type TestContext = {
  mockStatusBarItem: StatusBarItem
  mockGetSuggestionsStats: GetSuggestionsStats
  mockState: IVersionLensState
  mockOptions: SuggestionsOptions
  mockLogger: ILogger
  mockShowSuggestionsStats: IContextState<boolean>
  testStatusBarItem: StatusBarItem
  testEvent: OnRefreshSuggestionsStats
}

export const OnRefreshSuggestionsStatsTests = {

  [test.title]: OnRefreshSuggestionsStats.name,

  beforeEach: function (this: TestContext) {
    this.mockStatusBarItem = mock<StatusBarItem>();
    this.mockGetSuggestionsStats = mock<GetSuggestionsStats>();
    this.mockState = mock<IVersionLensState>();
    this.mockOptions = mock<SuggestionsOptions>();
    this.mockLogger = mock<ILogger>();
    this.mockShowSuggestionsStats = mock<IContextState<boolean>>();

    when(this.mockState.showSuggestionsStats)
      .thenReturn(instance(this.mockShowSuggestionsStats));
    when(this.mockOptions.indicators).thenReturn({
      Error: '🔴',
      NoMatch: '⚪',
      Match: '🟡'
    } as any)

    this.testStatusBarItem = instance(this.mockStatusBarItem)
    this.testEvent = new OnRefreshSuggestionsStats(
      this.testStatusBarItem,
      instance(this.mockGetSuggestionsStats),
      instance(this.mockState),
      instance(this.mockOptions),
      instance(this.mockLogger)
    );
  },

  "hides status bar item when showSuggestionsStats is false": async function (this: TestContext) {
    when(this.mockShowSuggestionsStats.value).thenReturn(false);
    // test
    await this.testEvent.execute(false);
    // verify
    verify(this.mockStatusBarItem.hide()).once();
  },

  "writes status text in status bar item": async function (this: TestContext) {
    const testStats = {
      errors: 1,
      noMatches: 0,
      updates: 10
    }
    when(this.mockShowSuggestionsStats.value).thenReturn(true);
    when(this.mockGetSuggestionsStats.execute(false)).thenResolve([{
      filePath: 'test/path/1',
      providerName: 'dotnet',
      errors: 1,
      noMatches: 0,
      updates: 10
    }])
    // test
    await this.testEvent.execute(false);
    // verify
    verify(this.mockLogger.info("fetching all suggestion stats")).once();
    verify(this.mockStatusBarItem.show()).once();
    verify(
      this.mockLogger.info(
        "completed fetching all suggestion stats ({duration} ms)",
        anyNumber()
      )
    ).once();
    // assert
    equal(this.testStatusBarItem.text, `🟡${testStats.updates} 🔴${testStats.errors} ⚪${testStats.noMatches}`)
  },

};