import type { CachingOptions } from '#domain/caching';
import { ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { createPackageResource } from '#domain/packages';
import {
  type INpmRegistry,
  type NpaSpec,
  type NpmConfig,
  NpmRegistryClient
} from '#domain/providers/npm';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import npa from 'npm-package-arg';
import { anyOfClass, anything, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  cachingOptsMock: CachingOptions
  loggerMock: ILogger
  configMock: NpmConfig
  npmRegistryMock: INpmRegistry
}

export const RequestsTests = {

  [test.title]: NpmRegistryClient.prototype.request.name,

  beforeEach: function (this: TestContext) {
    this.cachingOptsMock = mock<CachingOptions>();
    this.configMock = mock<NpmConfig>()
    this.loggerMock = mock<ILogger>()
    this.npmRegistryMock = mock<INpmRegistry>()

    when(this.cachingOptsMock.duration).thenReturn(30000);
    when(this.configMock.caching).thenReturn(instance(this.cachingOptsMock))
    when(this.configMock.prereleaseTagFilter).thenReturn([])
    when(this.npmRegistryMock.pickRegistry(anything(), anything()))
      .thenReturn("https://registry.npmjs.org/")
  },

  "returns successful responses": async function (this: TestContext) {
    const testResponse = {
      any: "test success response from npm registry"
    };

    const expectedResponse = {
      source: ClientResponseSource.remote,
      status: 200,
      data: testResponse,
      rejected: false
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

    const testClientData = {
      strictSSL: true,
      proxy: '',
      httpsProxy: ''
    };

    const testUrl = `https://registry.npmjs.org/${testNpaSpec.escapedName}`;

    when(this.npmRegistryMock.json(testUrl, testClientData))
      .thenResolve(testResponse);

    const cut = new NpmRegistryClient(
      instance(this.npmRegistryMock),
      instance(this.configMock),
      instance(this.loggerMock)
    );

    // test
    const actual = await cut.request(testNpaSpec, testClientData);

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
    assert.deepEqual(actual, expectedResponse);
  },

  "caches url responses when rejected": async function (this: TestContext) {
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

    const cut = new NpmRegistryClient(
      instance(this.npmRegistryMock),
      instance(this.configMock),
      instance(this.loggerMock)
    );

    try {
      // test
      await cut.request(testNpaSpec, anything());
      assert.ok(false);
    } catch (actual) {
      // assert
      assert.deepEqual(actual, expectedResponse);
    }

  },

}