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
  PackageDescriptorType,
  PackagePathDescriptor
} from '#domain/parsers';
import {
  type GoConfig,
  type GoHttpClient,
  GoSuggestionResolver
} from '#domain/providers/golang';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './goHttpClient.fixtures';

type TestContext = {
  configMock: GoConfig
  goHttpClientMock: GoHttpClient
  loggerMock: ILogger
  cut: GoSuggestionResolver
}

export const GoSuggestionResolverTests = {

  title: GoSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<GoConfig>();
    this.goHttpClientMock = mock<GoHttpClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new GoSuggestionResolver(
      instance(this.configMock),
      instance(this.goHttpClientMock),
      instance(this.loggerMock)
    );
  },

  'fromPath: returns a directory suggestion': async function (this: TestContext) {
    const testPackageName = 'github.com/test/pkg';
    const testPath = './local/pkg';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testPath,
      'project/path'
    );
    
    const testDependency = new PackageDependency(
        testPackageMan,
        new PackageDescriptor([])
    );

    const testPathDesc: PackagePathDescriptor = {
        type: PackageDescriptorType.path,
        path: testPath,
        pathRange: createTextRange(1, 10)
    };

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

  'fromGoApi: returns suggestions from go proxy': async function (this: TestContext) {
    const testPackageName = 'github.com/pkg/errors';
    const testVersion = 'v0.8.0';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'packagepath',
    );
    const testSpec = VersionUtils.parseSemver(testPackageMan.version)!;
    const testRequest: PackageClientRequest<null> = {
      providerName: 'golang',
      clientData: null,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageMan.name, createTextRange(0)),
          createPackageVersionDesc(testPackageMan.version, createTextRange(1)),
        ]),
      )
    };

    when(this.goHttpClientMock.get(testPackageName))
      .thenResolve({
        data: Fixtures.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromGoApi(testRequest, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  }
}