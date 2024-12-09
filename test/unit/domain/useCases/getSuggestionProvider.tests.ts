import type { ILogger } from '#domain/logging';
import type { IProviderConfig, ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';

type TestContext = {
  mockConfig: IProviderConfig,
  mockLogger: ILogger,
  testProviders: Array<ISuggestionProvider>
}

export const getSuggestionProviderTests = {

  [test.title]: GetSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.mockConfig = mock<IProviderConfig>();
    this.mockLogger = mock<ILogger>();

    when(this.mockConfig.fileLanguage).thenReturn('json');
    when(this.mockConfig.filePatterns).thenReturn('**/package.json');
    when(this.mockConfig.fileExcludePatterns).thenReturn(['**/node_modules/**']);

    this.testProviders = [
      <ISuggestionProvider>{
        name: "test",
        config: instance(this.mockConfig),
        logger: instance(this.mockLogger)
      }
    ]
  },

  "returns provider by file pattern": function (this: TestContext) {
    const usecase = new GetSuggestionProvider(this.testProviders);
    const actual = usecase.execute("package.json");
    assert.deepEqual(actual, this.testProviders[0]);
  },

  "returns no providers when file pattern does not match": function (this: TestContext) {
    const usecase = new GetSuggestionProvider(this.testProviders);
    const actual = usecase.execute("no-match.json");
    assert.equal(actual, undefined);
  },

  "excludes files using exclude pattern": function (this: TestContext) {
    const usecase = new GetSuggestionProvider(this.testProviders);
    const actual = usecase.execute("node_modules/package.json");
    assert.equal(actual, undefined);
  },

};