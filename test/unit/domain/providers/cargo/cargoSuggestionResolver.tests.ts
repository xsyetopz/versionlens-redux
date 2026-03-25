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
  type CargoConfig,
  type CratesClient,
  CargoSuggestionResolver
} from '#domain/providers/cargo';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './cratesClient.fixtures';

type TestContext = {
  configMock: CargoConfig
  cratesClientMock: CratesClient
  loggerMock: ILogger
  cut: CargoSuggestionResolver
}

export const CargoSuggestionResolverTests = {

  title: CargoSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<CargoConfig>();
    this.cratesClientMock = mock<CratesClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new CargoSuggestionResolver(
      instance(this.configMock),
      instance(this.cratesClientMock),
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

  'fromCrates: returns suggestions from registry and filters yanked': async function (this: TestContext) {
    const testPackageName = 'serde';
    const testVersion = '1.0.16';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'packagepath',
    );
    const testSpec = VersionUtils.parseSemver(testPackageMan.version)!;
    const testRequest: PackageClientRequest<null> = {
      providerName: 'cargo',
      clientData: null,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageMan.name, createTextRange(0)),
          createPackageVersionDesc(testPackageMan.version, createTextRange(1)),
        ]),
      )
    };

    const yankedVersion = "1.0.21";
    const testData = {
        versions: [
            { num: yankedVersion, yanked: true },
            ...Fixtures.test.versions
        ]
    };

    when(this.cratesClientMock.get(testPackageName))
      .thenResolve({
        data: testData as any,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromCrates(testRequest, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    
    // ensure yanked version is not in suggestions
    const hasYanked = actual.suggestions.some(s => s.version === yankedVersion);
    assert.equal(hasYanked, false, "Yanked version should be filtered out");
  }
}