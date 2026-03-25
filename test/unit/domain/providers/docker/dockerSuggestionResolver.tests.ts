import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  type PackageSuggestion,
  createPackageManifest,
  PackageDependency
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageRegistryDescType,
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
import assert, { deepEqual, equal } from 'node:assert';
import { existsSync, mkdirSync, rmSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './dockerSuggestionResolver.fixtures';

type TestContext = {
  configMock: DockerConfig
  dockerHubClientMock: DockerHubClient
  microsoftDockerClientMock: MicrosoftDockerClient
  loggerMock: ILogger
  cut: DockerSuggestionResolver
  testContextPath: string
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

  'fromPath: returns a directory suggestion for build context': async function (this: TestContext) {
    const testPackageName = 'node';
    const testPath = './docker/context';
    const testPackageFilePath = resolve('./project/package.json');
    const fullContextPath = resolve(dirname(testPackageFilePath), testPath);

    // ensure directory exists
    if (!existsSync(fullContextPath)) mkdirSync(fullContextPath, { recursive: true });

    const testPackageMan = createPackageManifest(
      testPackageName,
      'latest',
      testPackageFilePath
    );
    
    const testPathDesc = createPackagePathDescType(
      testPath,
      createTextRange(1, 10)
    );

    const testDependency = new PackageDependency(
        testPackageMan,
        new PackageDescriptor([testPathDesc])
    );

    // test
    const actual = await this.cut.fromPath(testDependency);

    // cleanup
    rmSync(dirname(fullContextPath), { recursive: true, force: true });

    // assert
    assert.equal(actual.source, 'file');
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, fullContextPath);
  },

  'fromRegistry: switches to microsoft client when registry is specified': async function (this: TestContext) {
    const testTag = 'latest';
    const testRegistry = 'mcr.microsoft.com';
    const testPackageMan = createPackageManifest('mssql/server', testTag, 'test/path');
    
    const testRegistryDesc = createPackageRegistryDescType(testRegistry);

    const testDependency = new PackageDependency(
        testPackageMan,
        new PackageDescriptor([testRegistryDesc])
    );

    when(this.microsoftDockerClientMock.get('server', 'mssql'))
      .thenResolve({
        data: fixtures.mssql.test,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromRegistry(testDependency);

    // assert
    assert.equal(actual.source, 'registry');
    assert.equal(actual.resolved?.name, 'mssql/server');
  },

  fromRegistry: {
    "case $i: returns suggestion statuses from DockerRepository arrays": [
      ['23', fixtures.node.test, fixtures.node.expectLatestStatusWithBuildSuggestions],
      ['22-bookworm', fixtures.node.test, fixtures.node.expectFixedWithSuggestions],
      ['21', fixtures.node.test, fixtures.node.expectNoMatchWithSuggestions],
      ['latest', fixtures.mssql.test, fixtures.mssql.expectLatestStatusWithBuildSuggestions],
      ['', fixtures.mssql.test, fixtures.mssql.expectNoMatchWithLatestSuggestion],
      async function (this: TestContext, testTag: string, testData: DockerRepository[], expected: PackageSuggestion[]) {
        const testNs = 'library'
        const testRepo = 'node'
        const testRequest: PackageClientRequest<null> = {
          providerName: 'docker',
          clientData: null,
          parsedDependency: new PackageDependency(
            createPackageManifest(testRepo, testTag, 'test/path'),
            new PackageDescriptor([
              createPackageNameDesc(testRepo, createTextRange(1, 20)),
              createPackageVersionDesc(testTag, createTextRange(25, 30)),
            ])
          )
        }

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