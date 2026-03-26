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
  PackageDescriptor
} from '#domain/parsers';
import {
  type DubConfig,
  type DubJsonClient,
  DubSuggestionResolver
} from '#domain/providers/dub';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './dubJsonClient.fixtures';

type TestContext = {
  configMock: DubConfig
  dubJsonClientMock: DubJsonClient
  loggerMock: ILogger
  cut: DubSuggestionResolver
}

export const DubSuggestionResolverTests = {

  title: DubSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DubConfig>();
    this.dubJsonClientMock = mock<DubJsonClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new DubSuggestionResolver(
      instance(this.configMock),
      instance(this.dubJsonClientMock),
      instance(this.loggerMock)
    );
  },

  'fromDubApi: returns suggestions from dub registry': async function (this: TestContext) {
    const testPackageName = 'vibe-d';
    const testVersion = '0.9.0';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'project/path'
    );
    const testSpec = VersionUtils.parseSemver(testVersion)!;
    const testRequest: PackageClientRequest<null> = {
      providerName: 'dub',
      clientData: null,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.dubJsonClientMock.get(testPackageName))
      .thenResolve({
        data: Fixtures.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromDubApi(testRequest, testSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  },

  'fromDubApi: returns matches latest for repo versions': async function (this: TestContext) {
    const testPackageName = 'vibe-d';
    const testVersion = '~master';
    const testPackageMan = createPackageManifest(testPackageName, testVersion, 'project/path');
    const testSpec = VersionUtils.parseSemver(testVersion)!;
    const testRequest: PackageClientRequest<null> = {
      providerName: 'dub',
      clientData: null,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.dubJsonClientMock.get(testPackageName))
      .thenResolve({
        data: ['~master', '0.9.0', '0.8.0'],
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromDubApi(testRequest, testSpec);

    // assert
    assert.equal(actual.suggestions[0].name, 'Matches latest');
    assert.equal(actual.suggestions[0].version, '~master');
  }
}