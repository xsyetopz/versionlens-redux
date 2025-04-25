import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { type JsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type ComposerConfig, PackagistClient } from '#domain/providers/composer';
import { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './packagistClient.fixtures';

type TestContext = {
  configMock: ComposerConfig;
  jsonClientMock: JsonHttpClient;
  loggerMock: ILogger;
}

export const PackagistClientTests = {

  title: PackagistClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<ComposerConfig>();
    this.jsonClientMock = mock<JsonHttpClient>();
    this.loggerMock = mock<ILogger>();

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000)
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
  },

  get: async function (this: TestContext) {
    // setup
    const testPackageName = 'test-package-name'
    const testApiUrl = `https://api/p2/`;
    const testUrl = `${testApiUrl}${testPackageName}.json`;
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
    const cut = new PackagistClient(
      instance(this.configMock),
      instance(this.jsonClientMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    );

    when(this.configMock.apiUrl).thenReturn(testApiUrl)
    when(this.jsonClientMock.get(testUrl)).thenResolve(testResp)

    // test
    const actual = await cut.get(testPackageName)
    const actualCached = await cut.get(testPackageName)

    // assert
    deepEqual(actual, expectedResp)
    deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache })
  }

}