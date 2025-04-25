import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { type IHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type PypiConfig, PypiHttpClient } from '#domain/providers/pypi';
import { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './pypiHttpClient.fixtures';

type TestContext = {
  configMock: PypiConfig
  httpClientMock: IHttpClient
  loggerMock: ILogger
}

export const PypiHttpClientTests = {

  title: PypiHttpClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<PypiConfig>();
    this.httpClientMock = mock<IHttpClient>();
    this.loggerMock = mock<ILogger>();

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000)
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
  },

  get: async function (this: TestContext) {
    // setup
    const testPackageName = 'test-package-name'
    const testApiUrl = `https://api/{name}`;
    const testUrl = `https://api/${testPackageName}`;
    const testResp = {
      data: fixtures.test,
      source: ClientResponseSource.remote,
      status: 200
    }
    const expectedResp = {
      data: fixtures.expected,
      source: ClientResponseSource.remote,
      status: 200
    }
    const cut = new PypiHttpClient(
      instance(this.configMock),
      instance(this.httpClientMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    );

    when(this.configMock.apiUrl).thenReturn(testApiUrl)
    when(this.httpClientMock.get(testUrl)).thenResolve(testResp)

    // test
    const actual = await cut.get(testPackageName)
    const actualCached = await cut.get(testPackageName)

    // assert
    deepEqual(actual, expectedResp)
    deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache })
  }

}