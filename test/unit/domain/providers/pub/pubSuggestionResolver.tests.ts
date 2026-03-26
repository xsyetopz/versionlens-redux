import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  createPackageManifest,
  PackageDependency,
  PackageSourceType,
  VersionUtils
} from '#domain/packages';
import {
  createTextRange,
  PackageDescriptor,
  PackagePathDescriptor,
  PackageDescriptorType
} from '#domain/parsers';
import {
  type PubConfig,
  type PubJsonClient,
  PubSuggestionResolver
} from '#domain/providers/pub';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './pubJsonClient.fixtures';

type TestContext = {
  configMock: PubConfig
  pubJsonClientMock: PubJsonClient
  loggerMock: ILogger
  cut: PubSuggestionResolver
}

export const PubSuggestionResolverTests = {

  title: PubSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<PubConfig>();
    this.pubJsonClientMock = mock<PubJsonClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new PubSuggestionResolver(
      instance(this.configMock),
      instance(this.pubJsonClientMock),
      instance(this.loggerMock)
    );
  },

  'fromPath: returns a directory suggestion': async function (this: TestContext) {
    const testPackageName = 'path';
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

  'fromPubApi: returns suggestions from pub registry': async function (this: TestContext) {
    const testPackageName = 'path';
    const testVersion = '1.8.0';
    const testUrl = `https://pub.dev/api/packages/${testPackageName}`;
    const testSpec = VersionUtils.parseSemver(testVersion)!;

    when(this.pubJsonClientMock.get(testUrl))
      .thenResolve({
        data: Fixtures.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromPubApi(testUrl, testPackageName, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  }
}