import { type ICachingOptions, CachingOptions } from '#domain/caching';
import {
  type HttpClientResponse,
  type IHttpClient,
  type IHttpOptions,
  ClientResponseSource,
  HttpOptions,
  JsonHttpClient
} from '#domain/clients';
import assert from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';

let cachingOptsMock: ICachingOptions;
let httpOptsMock: IHttpOptions;
let httpClientMock: IHttpClient;

export const JsonClientRequestTests = {

  title: "JsonClientRequest",

  beforeEach: () => {
    cachingOptsMock = mock(CachingOptions);
    httpOptsMock = mock(HttpOptions);
    httpClientMock = mock();

    when(cachingOptsMock.duration).thenReturn(30000);
    when(httpOptsMock.strictSSL).thenReturn(true);
  },

  "requestJson": {

    "returns response as an object": async () => {
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
        httpClientMock.get(
          anything(),
          anything(),
          anything()
        )
      ).thenResolve(testResponse)

      const rut = new JsonHttpClient(instance(httpClientMock));
      await rut.get(
        testUrl,
        testQueryParams
      ).then(response => {
        assert.deepEqual(response, expectedCacheData);
      })
    },

  },

};