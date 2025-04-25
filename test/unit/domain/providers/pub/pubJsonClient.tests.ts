import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type PubConfig, PubJsonClient } from '#domain/providers/pub';
import { deepEqual } from 'node:assert';
import { instance, mock, when } from 'ts-mockito';
import fixtures from './pubJsonClient.fixtures';

type TestContext = {
  configMock: PubConfig;
  jsonClientMock: IJsonHttpClient;
  loggerMock: ILogger;
}

export const PubJsonClientTests = {

  title: PubJsonClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<PubConfig>();
    this.jsonClientMock = mock<IJsonHttpClient>();
    this.loggerMock = mock<ILogger>();
  },

  get: async function (this: TestContext) {
    // setup
    const testPackageName = 'test-package-name'
    const testApiUrl = `https://api/`;
    const testUrl = `${testApiUrl}${testPackageName}`;
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
    const cut = new PubJsonClient(
      instance(this.configMock),
      instance(this.jsonClientMock),
      instance(this.loggerMock)
    );

    // when(this.configMock.apiUrl).thenReturn(testApiUrl)
    when(this.jsonClientMock.get(testUrl)).thenResolve(testResp)

    // test
    const actual = await cut.get(testUrl)
    // assert
    deepEqual(actual, expectedResp)
  }

}