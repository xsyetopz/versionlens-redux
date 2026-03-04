import type { ILogger } from '#domain/logging';
import { type GoConfig, type GoSuggestionResolver, GoSuggestionProvider } from '#domain/providers/golang';
import { test } from 'mocha-ui-esm';
import { deepEqual, equal } from 'node:assert';
import { instance, mock } from 'ts-mockito';
import Fixtures from './goSuggestionProvider.fixtures';

type TestContext = {
  resolverMock: GoSuggestionResolver
  goConfigMock: GoConfig
  loggerMock: ILogger
  put: GoSuggestionProvider
}

export const goSuggestionProviderTests = {

  [test.title]: GoSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.resolverMock = mock<GoSuggestionResolver>()
    this.goConfigMock = mock<GoConfig>()
    this.loggerMock = mock<ILogger>()
    this.put = new GoSuggestionProvider(
      instance(this.resolverMock),
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

  "parses go.mod dependencies": {
    "case $i: $1": [
      Fixtures.case1,
      function (this: TestContext, fixture: any) {
        // setup
        const { test, expected } = fixture;
        // test
        const actual = this.put.parseDependencies('test/path/go.mod', test);
        // assert
        deepEqual(actual, expected);
      }
    ]
  },

}
