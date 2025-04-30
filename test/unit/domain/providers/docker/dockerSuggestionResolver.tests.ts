import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageSuggestion,
  createPackageResource,
  PackageDependency
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type DockerConfig,
  type DockerHubClient,
  type DockerRepository,
  type MicrosoftDockerClient,
  DockerSuggestionResolver
} from '#domain/providers/docker';
import { deepEqual, equal } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './dockerSuggestionResolver.fixtures';

type TestContext = {
  configMock: DockerConfig;
  dockerHubClientMock: DockerHubClient;
  microsoftDockerClientMock: MicrosoftDockerClient;
  loggerMock: ILogger;
  cut: DockerSuggestionResolver
}

export const DockerSuggestionResolverTests = {

  title: DockerSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DockerConfig>();
    this.dockerHubClientMock = mock<DockerHubClient>();
    this.microsoftDockerClientMock = mock<MicrosoftDockerClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new DockerSuggestionResolver(
      instance(this.configMock),
      instance(this.dockerHubClientMock),
      instance(this.microsoftDockerClientMock),
      instance(this.loggerMock)
    );
  },

  fromRegistry: {
    "case $i: returns suggestion statuses from DockerRepository arrays": [
      ['23', fixtures.node.test, fixtures.node.expected1],
      ['22-bookworm', fixtures.node.test, fixtures.node.expected2],
      ['21', fixtures.node.test, fixtures.node.expected3],
      async function (this: TestContext, testTag: string, testData: DockerRepository[], expected: PackageSuggestion[]) {
        const testNs = 'library'
        const testRepo = 'node'
        const testRequest = {
          providerName: 'docker',
          attempt: 1,
          clientData: null,
          parsedDependency: new PackageDependency(
            createPackageResource(testRepo, testTag, 'test/path'),
            new PackageDescriptor([
              createPackageNameDesc(testRepo, createTextRange(1, 20)),
              createPackageVersionDesc(testTag, createTextRange(25, 30)),
            ])
          )
        } as PackageClientRequest<null>

        when(this.dockerHubClientMock.get(testRepo, testNs))
          .thenResolve({
            data: testData,
            source: ClientResponseSource.remote,
            status: 200
          })

        const actual = await this.cut.fromRegistry(testRequest.parsedDependency)
        equal(actual.suggestions.length, expected.length)
        deepEqual(actual.suggestions, expected)
      }
    ]
  }

}