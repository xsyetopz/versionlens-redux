import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageManifest,
  PackageDependency,
  PackageSourceType,
  PackageVersionType,
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type DenoConfig,
  type JsrClient,
  DenoSuggestionResolver
} from '#domain/providers/deno';
import type { NpaSpec, NpmClientData, NpmSuggestionResolver } from '#domain/providers/npm';
import assert from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import Fixtures from './jsrClient.fixtures';

type TestContext = {
  configMock: DenoConfig
  jsrClientMock: JsrClient
  npmSuggestionResolverMock: NpmSuggestionResolver
  loggerMock: ILogger
  cut: DenoSuggestionResolver
}

export const DenoSuggestionResolverTests = {

  title: DenoSuggestionResolver.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DenoConfig>();
    this.jsrClientMock = mock<JsrClient>();
    this.npmSuggestionResolverMock = mock<NpmSuggestionResolver>();
    this.loggerMock = mock<ILogger>();
    this.cut = new DenoSuggestionResolver(
      instance(this.configMock),
      instance(this.jsrClientMock),
      instance(this.npmSuggestionResolverMock),
      instance(this.loggerMock)
    );
  },

  'fromNpm: delegates to npmSuggestionResolver': async function (this: TestContext) {
    const testPackageName = 'node';
    const testVersion = 'latest';
    const testPackageMan = createPackageManifest(
      testPackageName,
      testVersion,
      'packagepath'
    );

    const testRequest: PackageClientRequest<NpmClientData> = {
      providerName: 'deno',
      clientData: {} as any,
      parsedDependency: new PackageDependency(
        testPackageMan,
        new PackageDescriptor([
          createPackageNameDesc(testPackageName, createTextRange(0)),
          createPackageVersionDesc(testVersion, createTextRange(1)),
        ])
      )
    };

    const testNpaSpec = {} as NpaSpec;
    const testResponse = { source: PackageSourceType.Registry } as any;

    when(this.npmSuggestionResolverMock.fromRegistry(testRequest, testNpaSpec))
      .thenResolve(testResponse);

    // test
    const actual = await this.cut.fromNpm(testRequest, testNpaSpec);

    // assert
    assert.equal(actual, testResponse);
  },

  'fromJsr: returns suggestions from jsr registry': async function (this: TestContext) {
    const testPackageName = '@std/assert';
    const testVersion = '1.0.0';
    const testNpaSpec = {
      subSpec: {
        name: testPackageName,
        rawSpec: testVersion
      }
    } as NpaSpec;

    when(this.jsrClientMock.get(testPackageName))
      .thenResolve({
        data: Fixtures.expected,
        source: ClientResponseSource.remote,
        status: 200
      });

    // test
    const actual = await this.cut.fromJsr(testNpaSpec);

    // assert
    assert.equal(actual.source, PackageSourceType.Registry);
    assert.equal(actual.type, PackageVersionType.Alias);
    assert.equal(actual.resolved?.name, testPackageName);
    assert.equal(actual.resolved?.version, testVersion);
    assert.ok(actual.suggestions.length > 0);
  }
}