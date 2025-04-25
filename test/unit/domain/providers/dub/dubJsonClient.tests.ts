import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type DubConfig, DubJsonClient } from '#domain/providers/dub';
import { deepEqual } from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';
import fixtures from './dubJsonClient.fixtures';

type TestContext = {
  configMock: DubConfig
  jsonClientMock: IJsonHttpClient
  loggerMock: ILogger
}

export const DubJsonClientTests = {

  title: DubJsonClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DubConfig>();
    this.jsonClientMock = mock<IJsonHttpClient>();
    this.loggerMock = mock<ILogger>();

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000)
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
  },

  get: async function (this: TestContext) {
    // setup
    const testPackageName = 'test-package-name'
    const testApiUrl = `https://api/`;
    const testUrl = `${testApiUrl}${testPackageName}/info`;
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
    const cut = new DubJsonClient(
      instance(this.configMock),
      instance(this.jsonClientMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    );

    when(this.configMock.apiUrl).thenReturn(testApiUrl)
    when(this.jsonClientMock.get(testUrl, anything())).thenResolve(testResp)

    // test
    const actual = await cut.get(testPackageName)
    const actualCached = await cut.get(testPackageName)

    // assert
    deepEqual(actual, expectedResp)
    deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache })
  }

}