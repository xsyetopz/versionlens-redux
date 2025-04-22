import { ClientResponseSource } from '#domain/clients';
import { ILogger } from '#domain/logging';
import { ClientResponseFactory, createPackageResource, PackageDependency, TPackageClientRequest } from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import { DockerClient, DockerConfig, DockerHubClient } from '#domain/providers/docker';
import { deepEqual, equal } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './dockerClient.fixtures.js';

type TestContext = {
  configMock: DockerConfig;
  dockerHubClientMock: DockerHubClient;
  loggerMock: ILogger;
  cut: DockerClient
}

export const dockerClientTests = {

  title: DockerClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock(DockerConfig);
    this.dockerHubClientMock = mock(DockerHubClient);
    this.loggerMock = mock<ILogger>();
    this.cut = new DockerClient(
      instance(this.configMock),
      instance(this.dockerHubClientMock),
      instance(this.loggerMock)
    );
  },

  fetchPackage: {
    "returns not supported suggestion": async function (this: TestContext) {
      const testNs = 'library'
      const testRepo = '${ARG1}'
      const testRequest = {
        providerName: 'docker',
        attempt: 1,
        clientData: {},
        parsedDependency: new PackageDependency(
          createPackageResource(testRepo, '23', 'test/path'),
          new PackageDescriptor([
            createPackageNameDesc(testRepo, createTextRange(1, 20)),
            createPackageVersionDesc('23', createTextRange(25, 30)),
          ])
        )
      } as TPackageClientRequest<null>

      when(this.dockerHubClientMock.get(testRepo, testNs))
        .thenResolve({
          data: fixtures.test,
          source: ClientResponseSource.remote,
          status: 200
        })

      const actual = await this.cut.fetchPackage(testRequest)
      deepEqual(actual, ClientResponseFactory.createNotSupported())
    },
    "creates suggestions": {
      async function(this: TestContext) {
        const testNs = 'library'
        const testRepo = 'node'
        const testRequest = {
          providerName: 'docker',
          attempt: 1,
          clientData: {},
          parsedDependency: new PackageDependency(
            createPackageResource(testRepo, '23', 'test/path'),
            new PackageDescriptor([
              createPackageNameDesc(testRepo, createTextRange(1, 20)),
              createPackageVersionDesc('23', createTextRange(25, 30)),
            ])
          )
        } as TPackageClientRequest<null>

        when(this.dockerHubClientMock.get(testRepo, testNs))
          .thenResolve({
            data: fixtures.test,
            source: ClientResponseSource.remote,
            status: 200
          })

        const actual = await this.cut.fetchPackage(testRequest)
        equal(actual.suggestions.length, 2)
        deepEqual(actual.suggestions, fixtures.expected)
      }
    }
  }

}