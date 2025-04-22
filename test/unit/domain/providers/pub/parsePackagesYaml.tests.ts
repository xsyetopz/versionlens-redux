import { ILogger } from '#domain/logging';
import { PubClient, PubConfig, PubSuggestionProvider } from '#domain/providers/pub';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './parsePackagesYaml.fixtures';

type TestContext = {
  pubClientMock: PubClient
  pubConfigMock: PubConfig
  loggerMock: ILogger
}

export const pubSuggestionProviderTests = {

  [test.title]: PubSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.pubClientMock = mock<PubClient>()
    this.pubConfigMock = mock<PubConfig>()
    this.loggerMock = mock<ILogger>()
  },

  "returns empty when no matches found": function (this: TestContext) {
    const put = new PubSuggestionProvider(
      instance(this.pubClientMock),
      instance(this.pubConfigMock),
      instance(this.loggerMock)
    );
    // test
    const actual = put.parseDependencies('test/path', '')
    // assert
    assert.equal(actual.length, 0);
  },

  "returns empty when no dependency entry names match": function (this: TestContext) {
    const includePropNames = ["non-dependencies"];
    const put = new PubSuggestionProvider(
      instance(this.pubClientMock),
      instance(this.pubConfigMock),
      instance(this.loggerMock)
    );

    when(this.pubConfigMock.dependencyProperties).thenReturn(includePropNames);

    const results = put.parseDependencies('test/path', Fixtures.parsesDependencyEntries.test);

    assert.equal(results.length, 0);
  },

  "case $i: parses yaml dependencies": [
    Fixtures.parsesDependencyEntries,
    Fixtures.parsesPathDependencies,
    Fixtures.parsesGitDepencdencies,
    Fixtures.parsesHostedDependencies,
    Fixtures.parsesProjectVersionNoQuotes,
    Fixtures.parsesProjectVersionWithQuotes,
    Fixtures.parsesProjectVersionWithComment,
    Fixtures.parsesEmptyProjectVersionWithComment,
    function (this: TestContext, fixture: any) {
      const includePropNames = ["version", "dependencies"];
      const put = new PubSuggestionProvider(
        instance(this.pubClientMock),
        instance(this.pubConfigMock),
        instance(this.loggerMock)
      );

      when(this.pubConfigMock.dependencyProperties).thenReturn(includePropNames);

      const results = put.parseDependencies('test/path', fixture.test);

      assert.deepEqual(results, fixture.expected);
    }
  ]
}