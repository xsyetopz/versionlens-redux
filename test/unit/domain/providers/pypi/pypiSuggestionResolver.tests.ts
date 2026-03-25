import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageManifest,
  PackageDependency,
  PackageSourceType,
  VersionUtils
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor,
  createPackagePathDescType
} from '#domain/parsers';
import {
  type PypiConfig,
  type PypiHttpClient,
  PypiSuggestionResolver
} from '#domain/providers/pypi';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './pypiHttpClient.fixtures';

type TestContext = {
  configMock: PypiConfig
  pypiHttpClientMock: PypiHttpClient
  loggerMock: ILogger
  cut: PypiSuggestionResolver
}

export const PypiSuggestionResolverTests = {

  title: PypiSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<PypiConfig>();
    this.pypiHttpClientMock = mock<PypiHttpClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new PypiSuggestionResolver(
      instance(this.configMock),
      instance(this.pypiHttpClientMock),
      instance(this.loggerMock)
    );
  },

  'fromPath: returns a directory suggestion': async function (this: TestContext) {
    const testPackageName = 'test-package';
    const testPath = './local/path';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testPath,
      'project/path'
    );
    
    const testDependency = new PackageDependency(
        testPackageMan,
        new PackageDescriptor([])
    );

    const testPathDesc = createPackagePathDescType(
      testPath,
      createTextRange(1, 10)
    );

    // test
    const actual = await this.cut.fromPath(testDependency, testPathDesc);

    // assert
    assert.equal(actual.source, PackageSourceType.Directory);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testPath);
  },

  'fromGit: returns a git suggestion': async function (this: TestContext) {
    // test
    const actual = this.cut.fromGit();

    // assert
    assert.equal(actual.source, PackageSourceType.Git);
  },

  'fromPypiApi: returns suggestions from registry': async function (this: TestContext) {
    const testPackageName = 'pip';
    const testVersion = '24.3';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'packagepath',
    );
    const testSpec = VersionUtils.parseSemver(testPackageMan.version)!;
    const testRequest: PackageClientRequest<null> = {
      providerName: 'pypi',
      clientData: null,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageMan.name, createTextRange(0)),
          createPackageVersionDesc(testPackageMan.version, createTextRange(1)),
        ]),
      )
    };

    when(this.pypiHttpClientMock.get(testPackageName))
      .thenResolve({
        data: Fixtures.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromPypiApi(testRequest, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  }
}