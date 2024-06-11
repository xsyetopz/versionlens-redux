import assert from 'node:assert';
import { CachingOptions, ICachingOptions } from '#domain/caching';
import {
  ClientResponseSource,
  HttpClientRequestMethods,
  HttpClientResponse,
  HttpClientOptions,
  UrlUtils
} from '#domain/clients';
import { KeyStringDictionary } from '#domain/utils';
import { HttpOptions, IHttpOptions } from '#domain/http';
import { ILogger } from '#domain/logging';
import { RequestLightClient } from 'infrastructure/http';
import { test } from 'mocha-ui-esm';
import { LoggerStub } from 'test/unit/domain/logging';
import {
  anything,
  capture,
  instance,
  mock,
  when
} from 'ts-mockito';
import { RequestLightStub } from './requestLightStub';

let cachingOptsMock: ICachingOptions;
let httpOptsMock: IHttpOptions;
let loggerMock: ILogger;
let requestLightMock: RequestLightStub;

let rut: RequestLightClient;

export const RequestLightClientTests = {

  [test.title]: RequestLightClient.prototype.request.name,

  beforeEach: () => {
    cachingOptsMock = mock(CachingOptions);
    httpOptsMock = mock(HttpOptions);
    loggerMock = mock(LoggerStub);
    requestLightMock = mock(RequestLightStub);

    rut = new RequestLightClient(
      <any>instance(requestLightMock).xhr,
      <HttpClientOptions>{
        caching: instance(cachingOptsMock),
        http: instance(httpOptsMock)
      },
      instance(loggerMock)
    );

    when(cachingOptsMock.duration).thenReturn(30000);
    when(httpOptsMock.strictSSL).thenReturn(true);
  },

  "should set strictSSL to $1 and cache duration to $2 in xhr options": [
    [true, 3000],
    [false, 0],
    async (testStrictSSL: boolean, testDuration: number) => {
      when(requestLightMock.xhr(anything()))
        .thenResolve(<any>{
          responseText: '{}',
          status: 200
        })

      when(cachingOptsMock.duration).thenReturn(testDuration);
      when(httpOptsMock.strictSSL).thenReturn(testStrictSSL);

      const rut = new RequestLightClient(
        <any>instance(requestLightMock).xhr,
        <HttpClientOptions>{
          caching: instance(cachingOptsMock),
          http: instance(httpOptsMock)
        },
        instance(loggerMock)
      );

      await rut.request(HttpClientRequestMethods.get, 'anywhere')

      const [actualOpts] = capture(requestLightMock.xhr).first();
      assert.equal(actualOpts.strictSSL, testStrictSSL);
    }
  ],

  "generates the expected url $1": [
    ["with no query params", {}],
    ["with query params", { param1: 1, param2: 2 }],
    async (testTitlePart: string, testQuery: KeyStringDictionary) => {
      const testUrl = 'https://test.url.example/path';

      when(requestLightMock.xhr(anything()))
        .thenResolve(<any>{
          status: 200,
          responseText: null
        })

      const expectedUrl = UrlUtils.createUrl(testUrl, testQuery);

      await rut.request(
        HttpClientRequestMethods.get,
        testUrl,
        testQuery
      )

      const [actualOpts] = capture(requestLightMock.xhr).first();
      assert.equal(actualOpts.url, expectedUrl);
      assert.equal(actualOpts.type, HttpClientRequestMethods.get);
    }
  ],

  "returns successful responses": async () => {
    const testUrl = 'https://test.url.example/path';
    const testQueryParams = {}
    const testXhrResponse = {
      source: ClientResponseSource.remote,
      status: 200,
      responseText: "success test",
    };

    const expectedResponse = <HttpClientResponse>{
      source: ClientResponseSource.remote,
      status: testXhrResponse.status,
      data: testXhrResponse.responseText,
      rejected: false
    }

    when(requestLightMock.xhr(anything())).thenResolve(<any>testXhrResponse)

    const actual = await rut.request(
      HttpClientRequestMethods.get,
      testUrl,
      testQueryParams
    );

    assert.deepEqual(actual, expectedResponse);
  },

  "returns rejected responses": async () => {
    const testUrl = 'https://test.url.example/path';
    const testQueryParams = {}
    const testResponse = {
      status: 404,
      responseText: "not found",
      source: ClientResponseSource.remote
    };

    const expectedResponse = <HttpClientResponse>{
      status: testResponse.status,
      data: testResponse.responseText,
      source: ClientResponseSource.remote,
      rejected: true,
    }

    when(requestLightMock.xhr(anything())).thenReject(<any>testResponse)

    // test
    try {
      await rut.request(
        HttpClientRequestMethods.get,
        testUrl,
        testQueryParams
      );
      assert.ok(false);
    } catch (actual) {
      assert.deepEqual(actual, expectedResponse);
    }
  }

};