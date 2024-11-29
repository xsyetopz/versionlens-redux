import type { IAuthorizer } from '#domain/authorization';
import { type ICachingOptions, CachingOptions } from '#domain/caching';
import {
  type HttpClientOptions,
  type HttpClientResponse,
  type IHttpOptions,
  ClientResponseSource,
  HttpClientRequestMethods,
  HttpOptions
} from '#domain/clients';
import {
  type IXhrRequest,
  type IXhrResponse,
  httpClientDefaultHeaders,
  RequestLightClient
} from '#domain/clients/requestLight';
import { type KeyStringDictionary, createUrl } from '#domain/utils';
import { test } from 'mocha-ui-esm';
import assert from 'node:assert';
import type { XHROptions } from 'request-light';
import { anything, capture, deepEqual, instance, mock, verify, when } from 'ts-mockito';

type TestContext = {
  mockAuthorizer: IAuthorizer
  mockCachingOpts: ICachingOptions
  mockHttpOpts: IHttpOptions
  mockRequestLight: IXhrRequest,
  testRequestOpts: XHROptions,
  rut: RequestLightClient
}

export const RequestLightClientTests = {

  [test.title]: RequestLightClient.name,

  beforeEach: function (this: TestContext) {
    this.mockAuthorizer = mock<IAuthorizer>();
    this.mockCachingOpts = mock(CachingOptions);
    this.mockHttpOpts = mock(HttpOptions);
    this.mockRequestLight = mock<IXhrRequest>();
    const testOptions: HttpClientOptions = {
      caching: instance(this.mockCachingOpts),
      http: instance(this.mockHttpOpts)
    };

    when(this.mockRequestLight.xhr(anything())).thenResolve({
      status: 0,
      headers: { ...httpClientDefaultHeaders },
      responseText: ''
    });

    // default options
    when(this.mockCachingOpts.duration).thenReturn(30000);
    when(this.mockHttpOpts.strictSSL).thenReturn(true);

    // request under test
    this.rut = new RequestLightClient(
      instance(this.mockRequestLight),
      instance(this.mockAuthorizer),
      testOptions
    );
  },

  "should set strictSSL to $1 and cache duration to $2 in xhr options": [
    [true, 3000],
    [false, 0],
    async function (this: TestContext, testStrictSSL: boolean, testDuration: number) {
      const testOptions: HttpClientOptions = {
        caching: instance(this.mockCachingOpts),
        http: instance(this.mockHttpOpts)
      };

      when(this.mockRequestLight.xhr(anything()))
        .thenResolve({
          status: 200,
          headers: {},
          responseText: 'test response'
        });

      // set test options
      when(this.mockCachingOpts.duration).thenReturn(testDuration);
      when(this.mockHttpOpts.strictSSL).thenReturn(testStrictSSL);

      const rut = new RequestLightClient(
        instance(this.mockRequestLight),
        instance(this.mockAuthorizer),
        testOptions
      );

      // test
      await rut.get('http://anywhere');

      // assert
      const [actualOpts] = capture(this.mockRequestLight.xhr).first();
      assert.equal(actualOpts.strictSSL, testStrictSSL);
    }
  ],

  "generates the expected url $1": [
    ["with no query params", {}],
    ["with query params", { param1: 1, param2: 2 }],
    async function (this: TestContext, testTitlePart: string, testQuery: KeyStringDictionary) {
      const testUrl = 'https://test.url.example/path';
      const expectedUrl = createUrl(testUrl, testQuery);

      when(this.mockRequestLight.xhr(anything()))
        .thenResolve({
          status: 200,
          headers: {},
          responseText: null
        });

      // test
      await this.rut.get(testUrl, testQuery);

      // assert
      const [actualOpts] = capture(this.mockRequestLight.xhr).first();
      assert.equal(actualOpts.url, expectedUrl);
      assert.equal(actualOpts.type, HttpClientRequestMethods.get);
    }
  ],

  "returns successful responses": async function (this: TestContext) {
    const testUrl = 'https://test.url.example/path';
    const testQueryParams = {}
    const testXhrResponse = {
      source: ClientResponseSource.remote,
      status: 200,
      responseText: "success test",
    };
    const expectedResponse: HttpClientResponse = {
      source: ClientResponseSource.remote,
      status: testXhrResponse.status,
      data: testXhrResponse.responseText,
      rejected: false
    };

    when(this.mockRequestLight.xhr(anything()))
      .thenResolve({
        status: testXhrResponse.status,
        headers: {},
        responseText: testXhrResponse.responseText
      });

    // test
    const actual = await this.rut.get(testUrl, testQueryParams);

    // assert
    assert.deepEqual(actual, expectedResponse);
  },

  "returns rejected responses": async function (this: TestContext) {
    const testUrl = 'https://test.url.example/path';
    const testQueryParams = {}
    const testResponse = {
      status: 404,
      responseText: "not found",
      source: ClientResponseSource.remote
    };
    const testFailedResponse: IXhrResponse = {
      status: testResponse.status,
      headers: {},
      responseText: testResponse.responseText
    };
    const expectedResponse: HttpClientResponse = {
      status: testResponse.status,
      data: testResponse.responseText,
      source: ClientResponseSource.remote,
      rejected: true,
    };

    when(this.mockRequestLight.xhr(anything()))
      .thenReject(<any>testFailedResponse);

    // test
    try {
      await this.rut.get(testUrl, testQueryParams);
      assert.ok(false);
    } catch (actual) {
      assert.deepEqual(actual, expectedResponse);
    }
  },

  "throws rejected errors": async function (this: TestContext) {
    const testUrl = 'https://test.url.example/path';
    when(this.mockRequestLight.xhr(anything())).thenReject(new Error("should be thrown"));

    // test
    try {
      await this.rut.get(testUrl);
      assert.ok(false);
    } catch (actual) {
      assert.ok(actual instanceof Error);
      assert.equal(actual.message, "should be thrown");
    }
  },

  "uses provided caller authorization headers": async function (this: TestContext) {
    const testHost = 'test.url.example';
    const testUrl = `https://${testHost}/path`;
    const testAuth = { Authorization: 'test provided token' };
    const expectedOptions: XHROptions = {
      url: testUrl,
      type: HttpClientRequestMethods.get,
      headers: { ...testAuth, ...httpClientDefaultHeaders },
      strictSSL: true
    };

    // test
    const actual = await this.rut.get(testUrl, {}, testAuth);

    // verify
    verify(this.mockRequestLight.xhr(deepEqual(expectedOptions))).once();
    verify(this.mockAuthorizer.urlHasAuthConsent(testHost)).never();

    // assert
    assert.ok(!!actual);
  },

  "$1 headers.Authorization when isUrlAuthorized() is '$2' and authToken is '$3'":
    [
      ['auto sets', true, 'test token'],
      ['does not set', true, undefined],
      ['does not set', false, undefined],
      async function (
        this: TestContext,
        testTitle: string,
        testIsUrlAuthorized: boolean,
        testToken: string
      ) {
        const testHost = 'https://test.url.example';
        const testUrl = `${testHost}/path`;
        const expectedOptions: XHROptions = {
          url: testUrl,
          type: HttpClientRequestMethods.get,
          headers: httpClientDefaultHeaders,
          strictSSL: true
        };

        if (testToken !== undefined) {
          expectedOptions.headers = {
            ...expectedOptions.headers,
            ...{
              Authorization: testToken,
            }
          }
        }

        // set options
        when(this.mockHttpOpts.strictSSL).thenReturn(true);

        // set auth
        when(this.mockAuthorizer.urlHasAuthConsent(testHost))
          .thenReturn(testIsUrlAuthorized);

        when(this.mockAuthorizer.getToken(testHost)).thenResolve(testToken);

        // test
        const actual = await this.rut.get(testUrl);

        // verify
        verify(this.mockRequestLight.xhr(deepEqual(expectedOptions))).once();
        verify(this.mockAuthorizer.urlHasAuthConsent(testHost)).once();
        verify(this.mockAuthorizer.retryCredentials(testHost)).never();
        verify(this.mockAuthorizer.getConsent(testHost)).never();

        // assert
        assert.ok(!!actual);
      }
    ],

  "does not retry to authorize when authorizer.getConsent() returns false":
    async function (this: TestContext) {
      const testHost = 'https://test.url.example';
      const testUrl = `${testHost}/path`;
      const testRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: httpClientDefaultHeaders,
        strictSSL: true
      };
      const testResponse: IXhrResponse = {
        status: 401,
        headers: {},
        responseText: 'not authorized'
      };

      when(this.mockRequestLight.xhr(deepEqual(testRequest))).thenReject(<any>testResponse);
      when(this.mockAuthorizer.getConsent(testHost)).thenResolve(false);

      try {
        // test
        await this.rut.get(testUrl);
        assert.fail();
      } catch (error) {
        // assert
        assert.equal(error.status, 401);
        assert.equal(error.data, testResponse.responseText);
      }

      // verify
      verify(this.mockAuthorizer.getToken(anything())).never();
      verify(this.mockRequestLight.xhr(deepEqual(testRequest))).once();
      verify(this.mockAuthorizer.getConsent(testHost)).once();
      verify(this.mockAuthorizer.retryCredentials(testHost)).never();
    },

  "retries to authorize when authorizer.getConsent() returns true":
    async function (this: TestContext) {
      let testRetries = 0;
      const testHost = 'https://test.url.example';
      const testUrl = `${testHost}/path`;
      const testToken = '12345678';
      const testFirstRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: httpClientDefaultHeaders,
        strictSSL: true
      };
      const testSecondRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: { ...httpClientDefaultHeaders, Authorization: testToken },
        strictSSL: true
      };
      const testResponse: IXhrResponse = {
        status: 401,
        headers: {},
        responseText: 'not authorized'
      };

      // set state first time is called
      when(this.mockAuthorizer.urlHasAuthConsent(testHost)).thenReturn(false);

      // first xhr time called
      when(this.mockRequestLight.xhr(deepEqual(testFirstRequest)))
        .thenCall(() => {
          testRetries++;
          when(this.mockAuthorizer.urlHasAuthConsent(testHost)).thenReturn(true);
          when(this.mockAuthorizer.getToken(testHost)).thenResolve(testToken);
          when(this.mockAuthorizer.getConsent(testHost)).thenResolve(true);
          return Promise.reject(testResponse);
        });

      // second xhr time called
      when(this.mockRequestLight.xhr(deepEqual(testSecondRequest))).thenResolve({
        status: 200,
        responseText: 'success',
        headers: {}
      });

      try {
        // test
        const actual = await this.rut.get(testUrl);
        // assert
        assert.equal(testRetries, 1);
        assert.equal(actual.status, 200);
        assert.equal(actual.data, 'success');
      } catch (error) {
        assert.fail();
      }

      // verify
      verify(this.mockRequestLight.xhr(deepEqual(testFirstRequest))).once();
      verify(this.mockRequestLight.xhr(deepEqual(testSecondRequest))).once();
      verify(this.mockAuthorizer.urlHasAuthConsent(testHost)).twice();
      verify(this.mockAuthorizer.getToken(testHost)).once();
      verify(this.mockAuthorizer.getConsent(testHost)).once();
      verify(this.mockAuthorizer.retryCredentials(testHost)).never();
    },

  "does not retry to authorize when authorizer.retryCredentials() returns false":
    async function (this: TestContext) {
      const testHost = 'https://test.url.example';
      const testUrl = `${testHost}/path`;
      const testToken = '12345678';
      const testRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: { ...httpClientDefaultHeaders, Authorization: testToken },
        strictSSL: true
      };
      const testResponse: IXhrResponse = {
        status: 401,
        headers: {},
        responseText: 'not authorized'
      };

      when(this.mockAuthorizer.urlHasAuthConsent(testHost)).thenReturn(true);
      when(this.mockAuthorizer.getToken(testHost)).thenResolve(testToken);
      when(this.mockRequestLight.xhr(deepEqual(testRequest))).thenReject(<any>testResponse);
      when(this.mockAuthorizer.retryCredentials(testHost)).thenResolve(false);

      try {
        // test
        await this.rut.get(testUrl);
        assert.fail();
      } catch (error) {
        // assert
        assert.equal(error.status, 401);
        assert.equal(error.data, testResponse.responseText);
      }

      // verify
      verify(this.mockAuthorizer.getToken(anything())).once();
      verify(this.mockRequestLight.xhr(deepEqual(testRequest))).once();
      verify(this.mockAuthorizer.retryCredentials(testHost)).once();
      verify(this.mockAuthorizer.getConsent(testHost)).never();
    },

  "retries to authorize when authorizer.retryCredentials() returns true":
    async function (this: TestContext) {
      let testRetries = 0;
      const testHost = 'https://test.url.example';
      const testUrl = `${testHost}/path`;
      const testFirstToken = '12345678';
      const testSecondToken = 'ABCDEFGH';
      const testFirstRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: { ...httpClientDefaultHeaders, Authorization: testFirstToken },
        strictSSL: true
      };
      const testSecondRequest: XHROptions = {
        url: testUrl,
        type: HttpClientRequestMethods.get,
        headers: { ...httpClientDefaultHeaders, Authorization: testSecondToken },
        strictSSL: true
      };
      const testResponse: IXhrResponse = {
        status: 401,
        headers: {},
        responseText: 'not authorized'
      };

      when(this.mockAuthorizer.urlHasAuthConsent(testHost)).thenReturn(true);
      when(this.mockAuthorizer.getToken(testHost)).thenResolve(testFirstToken);

      // first xhr time called
      when(this.mockRequestLight.xhr(deepEqual(testFirstRequest)))
        .thenCall(() => {
          testRetries++;
          when(this.mockAuthorizer.getToken(testHost)).thenResolve(testSecondToken);
          when(this.mockAuthorizer.retryCredentials(testHost)).thenResolve(true);
          return Promise.reject(testResponse);
        });

      // second xhr time called
      when(this.mockRequestLight.xhr(deepEqual(testSecondRequest))).thenResolve({
        status: 200,
        responseText: 'success',
        headers: {}
      });

      try {
        // test
        const actual = await this.rut.get(testUrl);
        // assert
        assert.equal(testRetries, 1);
        assert.equal(actual.status, 200);
        assert.equal(actual.data, 'success');
      } catch (error) {
        assert.fail();
      }

      // verify
      verify(this.mockRequestLight.xhr(deepEqual(testFirstRequest))).once();
      verify(this.mockRequestLight.xhr(deepEqual(testSecondRequest))).once();
      verify(this.mockAuthorizer.urlHasAuthConsent(testHost)).twice();
      verify(this.mockAuthorizer.getToken(testHost)).twice();
      verify(this.mockAuthorizer.retryCredentials(testHost)).once();
      verify(this.mockAuthorizer.getConsent(testHost)).never();
    },

};