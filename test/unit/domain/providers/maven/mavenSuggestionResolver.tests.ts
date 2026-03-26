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
  type MavenClientData,
  type MavenConfig,
  type MavenHttpClient,
  MavenSuggestionResolver
} from '#domain/providers/maven';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './mavenHttpClient.fixtures';

type TestContext = {
  configMock: MavenConfig
  mavenHttpClientMock: MavenHttpClient
  loggerMock: ILogger
  cut: MavenSuggestionResolver
}

export const MavenSuggestionResolverTests = {

  title: MavenSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<MavenConfig>();
    this.mavenHttpClientMock = mock<MavenHttpClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new MavenSuggestionResolver(
      instance(this.configMock),
      instance(this.mavenHttpClientMock),
      instance(this.loggerMock)
    );
  },

  'fromMavenApi: returns suggestions from maven repository': async function (this: TestContext) {
    const testPackageName = 'junit:junit';
    const testVersion = '4.12';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'project/path'
    );
    const testSpec = VersionUtils.parseSemver(testVersion)!;
    const testRequest: PackageClientRequest<MavenClientData> = {
      providerName: 'maven',
      clientData: { repositories: [{ url: 'https://repo.maven.apache.org/maven2', protocol: 'https:' as any }] },
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.mavenHttpClientMock.get(testPackageName, testRequest.clientData.repositories.map(x => x.url)))
      .thenResolve({
        data: Fixtures.get.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromMavenApi(
      testRequest.clientData.repositories.map(x => x.url),
      testRequest,
      testSpec
    );

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  }
}