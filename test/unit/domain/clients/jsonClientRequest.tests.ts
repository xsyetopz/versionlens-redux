import type { CachingOptions } from '#domain/caching';
import {
  type HttpClientResponse,
  type HttpOptions,
  type IHttpClient,
  type IHttpOptions,
  ClientResponseSource,
  JsonHttpClient
} from '#domain/clients';
import assert from 'node:assert';
import { anything, deepEqual, instance, mock, when } from 'ts-mockito';

type TestContext = {
  cachingOptsMock: CachingOptions;
  httpOptsMock: IHttpOptions;
  httpClientMock: IHttpClient;
}

export const JsonClientRequestTests = {

  title: "JsonClientRequest",

  beforeEach: function (this: TestContext) {
    this.cachingOptsMock = mock<CachingOptions>();
    this.httpOptsMock = mock<HttpOptions>();
    this.httpClientMock = mock<IHttpClient>();

    when(this.cachingOptsMock.duration).thenReturn(30000);
    when(this.httpOptsMock.strictSSL).thenReturn(true);
  },

  "get": {

    "returns response as an object": async function (this: TestContext) {
      const testUrl = 'https://test.url.example/path';
      const testQueryParams = {}
      const testResponse: HttpClientResponse = {
        source: ClientResponseSource.remote,
        status: 404,
        data: '{ "item1": "not found" }',
      }

      const expectedCacheData: HttpClientResponse = {
        source: ClientResponseSource.remote,
        status: testResponse.status,
        data: JSON.parse(testResponse.data),
      }

      when(
        this.httpClientMock.get(
          testUrl,
          deepEqual(testQueryParams),
          anything()
        )
      ).thenResolve(testResponse)

      const rut = new JsonHttpClient(instance(this.httpClientMock));

      // test
      const actual = await rut.get(testUrl, testQueryParams);

      // assert
      assert.deepEqual(actual, expectedCacheData);
    },

  },

};