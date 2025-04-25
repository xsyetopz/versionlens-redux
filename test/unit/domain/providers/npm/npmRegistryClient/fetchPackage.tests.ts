import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import type { ILogger } from '#domain/logging';
import {
  type PackageClientRequest,
  createPackageResource,
  PackageDependency,
  PackageStatusFactory
} from '#domain/packages';
import {
  createPackageNameDesc,
  createPackageVersionDesc,
  createTextRange,
  PackageDescriptor
} from '#domain/parsers';
import {
  type INpmRegistry,
  type NpaSpec,
  type NpmClientData,
  type NpmConfig,
  NpmRegistryClient
} from '#domain/providers/npm';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import npa from 'npm-package-arg';
import { anything, instance, mock, when } from 'ts-mockito';
import Fixtures from './npmRegistryClient.fixtures';

type TestContext = {
  npmRegistryMock: INpmRegistry
  configMock: NpmConfig
  requestCache: MemoryExpiryCache
  loggerMock: ILogger
  cachingOptsMock: CachingOptions
  cut: NpmRegistryClient
}

export const fetchPackageTests = {

  [test.title]: NpmRegistryClient.prototype.fetchPackage.name,

  beforeEach: function (this: TestContext) {
    this.cachingOptsMock = mock<CachingOptions>()
    this.configMock = mock<NpmConfig>()
    this.requestCache = new MemoryExpiryCache('test-cache')
    this.loggerMock = mock<ILogger>()
    this.npmRegistryMock = mock<INpmRegistry>()
    this.cut = new NpmRegistryClient(
      instance(this.npmRegistryMock),
      instance(this.configMock),
      this.requestCache,
      instance(this.loggerMock)
    )

    when(this.configMock.caching).thenReturn(instance(this.cachingOptsMock))
    when(this.configMock.prereleaseTagFilter).thenReturn([])
    when(this.npmRegistryMock.pickRegistry(anything(), anything()))
      .thenReturn("https://registry.npmjs.org/")
  },

  'returns a registry range package': async function (this: TestContext) {
    const testPackageRes = createPackageResource(
      // package name
      'pacote',
      // package version
      '10.1.*',
      // package path
      'packagepath',
    );

    const testClientData: NpmClientData = {
      registry: 'https://registry.npmjs.org/',
      strictSSL: true
    };

    const testRequest: PackageClientRequest<NpmClientData> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        new PackageDescriptor([
          createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
          createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
        ]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(this.npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryRange)

    // test
    const actual = await this.cut.fetchPackage(testRequest, npaSpec)

    // assert
    assert.equal(actual.source, 'registry')
    assert.equal(actual.type, 'range')
    assert.equal(actual.resolved?.name, testPackageRes.name)
    assert.deepEqual(actual.resolved?.version, testPackageRes.version)
  },

  'returns a registry version package': async function (this: TestContext) {
    const testPackageRes = createPackageResource(
      // package name
      'npm-package-arg',
      // package version
      '8.0.1',
      // package path
      'packagepath',
    );

    const testClientData: NpmClientData = {
      registry: 'https://registry.npmjs.org/',
      strictSSL: true
    };

    const testRequest: PackageClientRequest<NpmClientData> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        new PackageDescriptor([
          createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
          createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
        ]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(this.npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryVersion)

    // test
    const actual = await this.cut.fetchPackage(testRequest, npaSpec)

    // assert
    assert.equal(actual.source, 'registry')
    assert.equal(actual.type, 'version')
    assert.equal(actual.resolved?.name, testPackageRes.name)
  },

  'returns capped latest versions': async function (this: TestContext) {
    const testPackageRes = createPackageResource(
      // package name
      'npm-package-arg',
      // package version
      '7.0.0',
      // package path
      'packagepath',
    );

    const testClientData: NpmClientData = {
      registry: 'https://registry.npmjs.org/',
      strictSSL: true
    };

    const testRequest: PackageClientRequest<NpmClientData> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        new PackageDescriptor([
          createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
          createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
        ]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(this.npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentCappedToLatestTaggedVersion)

    // test
    const actual = await this.cut.fetchPackage(testRequest, npaSpec)

    // assert
    assert.deepEqual(
      actual.suggestions,
      [
        PackageStatusFactory.createMatchesLatestStatus(testPackageRes.version)
      ]
    )
  },

  'returns a registry alias package': async function (this: TestContext) {
    const testPackageRes = createPackageResource(
      // package name
      'aliased',
      // package version
      'npm:pacote@11.1.9',
      // package path
      'packagepath',
    );

    const testClientData: NpmClientData = {
      registry: 'https://registry.npmjs.org/',
      strictSSL: true
    };

    const testRequest: PackageClientRequest<NpmClientData> = {
      providerName: 'testnpmprovider',
      clientData: testClientData,
      parsedDependency: new PackageDependency(
        testPackageRes,
        new PackageDescriptor([
          createPackageNameDesc(testPackageRes.name, createTextRange(0, 0)),
          createPackageVersionDesc(testPackageRes.version, createTextRange(1, 1)),
        ]),
      ),
      attempt: 1
    }

    const npaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(this.npmRegistryMock.json(anything(), anything()))
      .thenResolve(Fixtures.packumentRegistryAlias)

    // test
    const actual = await this.cut.fetchPackage(testRequest, npaSpec)

    // assert
    assert.equal(actual.source, 'registry')
    assert.equal(actual.type, 'alias')
    assert.equal(actual.resolved?.name, 'pacote')
    assert.equal(actual.resolved?.version, '11.1.9')
  }

}