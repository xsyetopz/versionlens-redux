import { test } from 'mocha-ui-esm';
import { deepEqual } from 'node:assert';
import { anything, instance, mock, when } from 'ts-mockito';
import { type IExpiryCache, CachingOptions } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import { type ILogger } from '#domain/logging';
import { type RubyConfig, type RubyHttpClientResponse, RubyHttpClient } from '#domain/providers/ruby';
import { rubyHttpClientFixtures, rubyHttpClientResultFixtures } from './rubyHttpClient.fixtures';

type TestContext = {
  mockConfig: RubyConfig;
  mockCaching: CachingOptions;
  mockJsonClient: IJsonHttpClient;
  mockRequestCache: IExpiryCache<RubyHttpClientResponse>;
  mockLogger: ILogger;
  testClient: RubyHttpClient;
}

export const RubyHttpClientTests = {
  [test.title]: RubyHttpClient.name,

  beforeEach: function (this: TestContext) {
    this.mockConfig = mock<RubyConfig>();
    this.mockCaching = mock(CachingOptions);
    this.mockJsonClient = mock<IJsonHttpClient>();
    this.mockRequestCache = mock<IExpiryCache<RubyHttpClientResponse>>();
    this.mockLogger = mock<ILogger>();

    this.testClient = new RubyHttpClient(
      instance(this.mockConfig),
      instance(this.mockJsonClient),
      instance(this.mockRequestCache),
      instance(this.mockLogger)
    );
  },

  "get": {

    "returns versions from registry and caches them": async function (this: TestContext) {
      // setup
      const testPackageName = 'test-package';
      const testUrl = 'https://rubygems.org/api/v1/versions/test-package.json';

      when(this.mockConfig.apiUrl).thenReturn('https://rubygems.org/api/v1/versions/{name}.json');
      when(this.mockCaching.duration).thenReturn(60);
      when(this.mockConfig.caching).thenReturn(instance(this.mockCaching));
      when(this.mockRequestCache.get(testUrl, 60)).thenReturn(undefined);
      when(this.mockJsonClient.get<any[]>(testUrl)).thenResolve(rubyHttpClientFixtures.success);
      when(this.mockRequestCache.set(testUrl, anything())).thenReturn(rubyHttpClientResultFixtures.success);

      // test
      const actual = await this.testClient.get(testPackageName);

      // assert
      deepEqual(actual, rubyHttpClientResultFixtures.success);
    },

    "returns versions from cache": async function (this: TestContext) {
      // setup
      const testPackageName = 'test-package';
      const testUrl = 'https://rubygems.org/api/v1/versions/test-package.json';
      const cachedResponse: RubyHttpClientResponse = {
        ...rubyHttpClientResultFixtures.success,
        source: ClientResponseSource.cache
      };

      when(this.mockConfig.apiUrl).thenReturn('https://rubygems.org/api/v1/versions/{name}.json');
      when(this.mockCaching.duration).thenReturn(60);
      when(this.mockConfig.caching).thenReturn(instance(this.mockCaching));
      when(this.mockRequestCache.get(testUrl, 60)).thenReturn(rubyHttpClientResultFixtures.success);

      // test
      const actual = await this.testClient.get(testPackageName);

      // assert
      deepEqual(actual, cachedResponse);
    }

  }

};
