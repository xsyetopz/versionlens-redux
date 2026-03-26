import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageManifest,
  PackageDependency,
  PackageSourceType
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type DotNetConfig,
  type NuGetClient,
  type NuGetClientData,
  DotnetSuggestionResolver
} from '#domain/providers/dotnet';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './nugetClient.fixtures';

type TestContext = {
  configMock: DotNetConfig
  nugetClientMock: NuGetClient
  loggerMock: ILogger
  cut: DotnetSuggestionResolver
}

export const DotnetSuggestionResolverTests = {

  title: DotnetSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DotNetConfig>();
    this.nugetClientMock = mock<NuGetClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new DotnetSuggestionResolver(
      instance(this.configMock),
      instance(this.nugetClientMock),
      instance(this.loggerMock)
    );
  },

  'fromNuGet: returns suggestions from nuget registry': async function (this: TestContext) {
    const testPackageName = 'Microsoft.Extensions.Logging';
    const testVersion = '5.0.0';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'project/path'
    );
    
    const testRequest: PackageClientRequest<NuGetClientData> = {
      providerName: 'dotnet',
      clientData: { serviceUrls: [] },
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.nugetClientMock.get(testPackageName, []))
      .thenResolve({
        data: Fixtures.get.test,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromNuGet(testRequest);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  },

  'fromNuGet: returns empty suggestions for unsupported four segment versions': async function (this: TestContext) {
    const testPackageName = 'Test.Package';
    const testVersion = '1.2.3.4';
    const testPackageMan = createPackageManifest(testPackageName, testVersion, 'project/path');
    
    const testRequest: PackageClientRequest<NuGetClientData> = {
      providerName: 'dotnet',
      clientData: { serviceUrls: [] },
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.nugetClientMock.get(testPackageName, []))
      .thenResolve({
        data: Fixtures.get.test,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromNuGet(testRequest);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.suggestions.length, 0);
  },

  'fromNuGet: returns no match status for invalid versions': async function (this: TestContext) {
    const testPackageName = 'Test.Package';
    const testVersion = 'invalid';
    const testPackageMan = createPackageManifest(testPackageName, testVersion, 'project/path');
    
    const testRequest: PackageClientRequest<NuGetClientData> = {
      providerName: 'dotnet',
      clientData: { serviceUrls: [] },
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    when(this.nugetClientMock.get(testPackageName, []))
      .thenResolve({
        data: Fixtures.get.test,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromNuGet(testRequest);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.suggestions[0].name, 'No Match');
  }
}