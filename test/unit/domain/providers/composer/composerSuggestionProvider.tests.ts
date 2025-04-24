import type { ILogger } from '#domain/logging';
import {
  type ComposerClient,
  type ComposerConfig,
  ComposerSuggestionProvider
} from '#domain/providers/composer';
import { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './composerSuggestionProvider.fixtures';

type TestContext = {
  ComposerClientMock: ComposerClient
  ComposerConfigMock: ComposerConfig
  loggerMock: ILogger
  put: ComposerSuggestionProvider
}

export const composerSuggestionProviderTests = {

  title: ComposerSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.ComposerClientMock = mock<ComposerClient>()
    this.ComposerConfigMock = mock<ComposerConfig>()
    this.loggerMock = mock<ILogger>()
    this.put = new ComposerSuggestionProvider(
      instance(this.ComposerClientMock),
      instance(this.ComposerConfigMock),
      instance(this.loggerMock)
    )
  },

  "parses dependencies": function (this: TestContext) {
    const testPackagePath = 'test/path/composer.json'
    const testProps = [
      'version',
      'require',
      'require-dev'
    ];
    when(this.ComposerConfigMock.dependencyProperties).thenReturn(testProps);
    // test
    const actual = this.put.parseDependencies(testPackagePath, fixtures.test)
    // assert
    deepEqual(actual, fixtures.expected)
  },

}