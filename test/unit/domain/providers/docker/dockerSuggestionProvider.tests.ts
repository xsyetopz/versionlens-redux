import { ILogger } from '#domain/logging';
import {
  DockerClient,
  DockerConfig,
  DockerSuggestionProvider
} from '#domain/providers/docker';
import { deepEqual } from 'node:assert';
import { instance, mock } from 'ts-mockito';
import fixtures from './dockerSuggestionProvider.fixtures';

type TestContext = {
  dockerClientMock: DockerClient
  dockerConfigMock: DockerConfig
  loggerMock: ILogger
}

export const dockerSuggestionProviderTests = {

  title: DockerSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.dockerClientMock = mock<DockerClient>()
    this.dockerConfigMock = mock<DockerConfig>()
    this.loggerMock = mock<ILogger>()
  },

  "parses lf dockerfile text": function (this: TestContext) {
    const testPackagePath = 'test/path'
    const put = new DockerSuggestionProvider(
      instance(this.dockerClientMock),
      instance(this.dockerConfigMock),
      instance(this.loggerMock)
    )
    // test
    const actual = put.parseDependencies(testPackagePath, fixtures.test)
    // assert
    deepEqual(actual, fixtures.expected)
  }

}