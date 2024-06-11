import assert from 'node:assert';
import { ILogger } from '#domain/logging';
import { IProviderConfig, ISuggestionProvider } from '#domain/providers';
import { GetSuggestionProvider } from '#domain/useCases';
import { test } from 'mocha-ui-esm';
import { instance, mock, when } from 'ts-mockito';

type TestContext = {
  mockConfig: IProviderConfig,
  testProviders: Array<ISuggestionProvider>
}

export const getSuggestionProviderTests = {

  [test.title]: GetSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    const mockLogger = mock<ILogger>();
    const mockConfig = mock<IProviderConfig>();
    when(mockConfig.fileMatcher).thenReturn({
      language: "json",
      scheme: "file",
      pattern: "**/package.json",
      exclude: "**/node_modules/**"
    });

    this.testProviders = [
      <ISuggestionProvider>{
        name: "test",
        config: instance(mockConfig),
        logger: instance(mockLogger)
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