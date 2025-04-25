import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { createPackageResource } from '#domain/packages';
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
import { anyOfClass, anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  configMock: NpmConfig
  loggerMock: ILogger
  npmRegistryMock: INpmRegistry
  cut: NpmRegistryClient
}

export const RequestsTests = {

  [test.title]: NpmRegistryClient.prototype.get.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<NpmConfig>()
    this.loggerMock = mock<ILogger>()
    this.npmRegistryMock = mock<INpmRegistry>()
    this.cut = new NpmRegistryClient(
      instance(this.npmRegistryMock),
      instance(this.configMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    )

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000);
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
    when(this.configMock.prereleaseTagFilter).thenReturn([])
    when(this.npmRegistryMock.pickRegistry(anything(), anything()))
      .thenReturn("https://registry.npmjs.org/")
  },

  "returns successful responses": async function (this: TestContext) {
    const testResponse = {
      'dist-tags': {},
      versions: []
    };

    const expectedResp = {
      source: ClientResponseSource.remote,
      status: 200,
      data: testResponse
    };

    const testPackageRes = createPackageResource(
      // package name
      'pacote',
      // package version
      '10.1.*',
      // package path
      'packagepath',
    );

    const testNpaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    const testClientData: NpmClientData = {
      registry: '',
      strictSSL: true,
      proxy: '',
      httpsProxy: ''
    };

    const testUrl = `https://registry.npmjs.org/${testNpaSpec.escapedName}`;

    when(this.npmRegistryMock.json(testUrl, testClientData))
      .thenResolve(testResponse);

    // test
    const actual = await this.cut.get(testNpaSpec, testClientData);
    const actualCached = await this.cut.get(testNpaSpec, testClientData);

    // verify
    verify(
      this.loggerMock.debug(
        "url: {url}, strict-ssl: {strictSSL}, proxy: {proxy}, https-proxy: {httpsProxy}",
        anyOfClass(URL),
        testClientData.strictSSL,
        testClientData.proxy,
        testClientData.httpsProxy
      )
    ).once();

    // assert
    assert.deepEqual(actual, expectedResp);
    assert.deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache });
  },

  "throws when rejected": async function (this: TestContext) {
    const testResponse = {
      code: "E404",
      message: "404 Not Found - GET https://registry.npmjs.org/somepackage - Not found",
    };

    const expectedResponse = {
      status: testResponse.code,
      data: testResponse.message,
      source: ClientResponseSource.remote,
      rejected: true,
    };

    const testPackageRes = createPackageResource(
      // package name
      'pacote',
      // package version
      '10.1.*',
      // package path
      'packagepath',
    );

    const testNpaSpec = npa.resolve(
      testPackageRes.name,
      testPackageRes.version,
      testPackageRes.path
    ) as NpaSpec;

    when(this.npmRegistryMock.json(anything(), anything()))
      .thenReject(<any>testResponse);

    try {
      // test
      await this.cut.get(testNpaSpec, anything());
      assert.ok(false);
    } catch (actual) {
      // assert
      assert.deepEqual(actual, expectedResponse);
    }

  },

}