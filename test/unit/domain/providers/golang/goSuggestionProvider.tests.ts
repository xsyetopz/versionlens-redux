import type { ILogger } from '#domain/logging';
import { type GoClient, type GoConfig, GoSuggestionProvider } from '#domain/providers/golang';
import { deepEqual, equal } from 'node:assert';
import { instance, mock } from 'ts-mockito';
import fixtures from './goSuggestionProvider.fixtures';

type TestContext = {
  goClientMock: GoClient
  goConfigMock: GoConfig
  loggerMock: ILogger
  put: GoSuggestionProvider
}

export const goSuggestionProviderTests = {

  title: GoSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.goClientMock = mock<GoClient>()
    this.goConfigMock = mock<GoConfig>()
    this.loggerMock = mock<ILogger>()
    this.put = new GoSuggestionProvider(
      instance(this.goClientMock),
      instance(this.goConfigMock),
      instance(this.loggerMock)
    )
  },

  "returns empty when file is empty": function (this: TestContext) {
    // test
    const actual = this.put.parseDependencies('test/path/go.mod', '')
    // assert
    equal(actual.length, 0);
  },

  "returns empty when no dependencies found": function (this: TestContext) {
    const testContent = `
      module github.com/xxx/yyy
      go 1.20
    `
    // test
    const actual = this.put.parseDependencies('test/path/go.mod', testContent)
    // assert
    equal(actual.length, 0);
  },

  "parses go.mod dependencies": function (this: TestContext) {
    // test
    const actual = this.put.parseDependencies('test/path/go.mod', fixtures.test)
    // assert
    deepEqual(actual, fixtures.expected)
  },

}