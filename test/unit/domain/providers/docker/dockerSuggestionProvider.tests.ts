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
  put: DockerSuggestionProvider
}

export const dockerSuggestionProviderTests = {

  title: DockerSuggestionProvider.name,

  beforeEach: function (this: TestContext) {
    this.dockerClientMock = mock<DockerClient>()
    this.dockerConfigMock = mock<DockerConfig>()
    this.loggerMock = mock<ILogger>()
    this.put = new DockerSuggestionProvider(
      instance(this.dockerClientMock),
      instance(this.dockerConfigMock),
      instance(this.loggerMock)
    )
  },

  "parses dockerfiles": function (this: TestContext) {
    const testPackagePath = 'test/path/dockerfile'
    // test
    const actual = this.put.parseDependencies(testPackagePath, fixtures.dockerfile.test)
    // assert
    deepEqual(actual, fixtures.dockerfile.expected)
  },

  "parses docker compose files": function (this: TestContext) {
    // test
    const actual = this.put.parseDependencies('test/path/compose.yaml', fixtures.compose.test)
    // assert
    deepEqual(actual, fixtures.compose.expected)
  }

}