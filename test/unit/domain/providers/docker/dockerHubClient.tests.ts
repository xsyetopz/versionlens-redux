import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { type JsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type DockerConfig, DockerHubClient } from '#domain/providers/docker';
import { deepEqual } from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';
import fixtures from './dockerHubClient.fixtures';

type TestContext = {
  configMock: DockerConfig;
  jsonClientMock: JsonHttpClient;
  loggerMock: ILogger;
}

export const dockerHubClientTests = {

  title: DockerHubClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DockerConfig>();
    this.jsonClientMock = mock<JsonHttpClient>();
    this.loggerMock = mock<ILogger>();

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000)
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
  },

  get: async function (this: TestContext) {
    // setup
    const testNs = 'library'
    const testRepo = 'node'
    const testUrl = `https://hub.docker.com/v2/namespaces/${testNs}/repositories/${testRepo}/tags`
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
    const cut = new DockerHubClient(
      instance(this.configMock),
      instance(this.jsonClientMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    );

    when(this.jsonClientMock.get(testUrl, anything())).thenResolve(testResp)

    // test
    const actual = await cut.get(testRepo, testNs)
    const actualCached = await cut.get(testRepo, testNs)

    // assert
    deepEqual(actual, expectedResp)
    deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache })
  }

}