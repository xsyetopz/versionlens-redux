import { type CachingOptions, MemoryExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, type JsonClientResponse, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { type DotNetConfig, NuGetClient } from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import { deepEqual, equal } from 'node:assert';
import {
  anyOfClass,
  anything,
  capture,
  instance,
  mock,
  verify,
  when
} from 'ts-mockito';
import Fixtures from './nugetClient.fixtures';

type TestContext = {
  configMock: DotNetConfig
  jsonClientMock: IJsonHttpClient
  loggerMock: ILogger
  cut: NuGetClient
}

export const NuGetClientTests = {

  title: NuGetClient.name,

  beforeEach: function (this: TestContext) {
    this.configMock = mock<DotNetConfig>();
    this.jsonClientMock = mock<IJsonHttpClient>();
    this.loggerMock = mock<ILogger>();
    this.cut = new NuGetClient(
      instance(this.configMock),
      instance(this.jsonClientMock),
      new MemoryExpiryCache('test-cache'),
      instance(this.loggerMock)
    );

    const cachingOptsMock = mock<CachingOptions>()
    when(cachingOptsMock.duration).thenReturn(3000)
    when(this.configMock.caching).thenReturn(instance(cachingOptsMock))
  },

  get: {
    "fetches from single url": async function (this: TestContext) {
      // setup
      const testPackageName = 'test-package-name'
      const testApiUrl = 'https://api';
      const testUrl = `${testApiUrl}/${testPackageName}/index.json`;
      const testResp = {
        data: Fixtures.get.test,
        source: ClientResponseSource.remote,
        status: 200
      }
      const expectedResp = {
        data: Fixtures.get.expected,
        source: ClientResponseSource.remote,
        status: 200
      }

      when(this.jsonClientMock.get(testUrl)).thenResolve(testResp)

      // test
      const actual = await this.cut.get(testPackageName, [testApiUrl])
      const actualCached = await this.cut.get(testPackageName, [testApiUrl])

      // assert
      deepEqual(actual, expectedResp)
      deepEqual(actualCached, { ...expectedResp, source: ClientResponseSource.cache })
    },
    "attempts fallback url when 404": async function (this: TestContext) {
      const testPackageName = 'test-package-name'
      const failUrl = `http://failed`
      const successUrl = `http://success`
      const testFailResp: JsonClientResponse<any> = {
        data: [],
        source: ClientResponseSource.remote,
        status: 404,
        rejected: true
      }
      const successResp: JsonClientResponse<any> = {
        data: [],
        source: ClientResponseSource.remote,
        status: 200,
        rejected: false
      }

      when(this.jsonClientMock.get(`${failUrl}/${testPackageName}/index.json`))
        .thenReject(testFailResp as any)

      when(this.jsonClientMock.get(`${successUrl}/${testPackageName}/index.json`))
        .thenResolve(successResp as any)

      // test
      const actual = await this.cut.get(testPackageName, [failUrl, successUrl])
      const actualCached = await this.cut.get(testPackageName, [failUrl, successUrl])

      // assert
      deepEqual(actual, successResp)
      deepEqual(actualCached, { ...successResp, source: ClientResponseSource.cache })
    }
  },

  fetchResource: {
    "returns the package resource from a list of resources": async function (this: TestContext) {
      const testSource = {
        enabled: true,
        machineWide: false,
        url: 'https://test',
        protocol: RegistryProtocols.https
      };

      const mockResponse = {
        source: ClientResponseSource.remote,
        status: 200,
        data: Fixtures.resource,
      };

      const expected = 'https://api.nuget.org/v3-flatcontainer1/';
      when(this.jsonClientMock.get(anything())).thenResolve(mockResponse)

      // test
      const actual = await this.cut.fetchResource(testSource);

      // verify
      verify(
        this.loggerMock.debug(
          "Resolved PackageBaseAddressService endpoint: {url}",
          anyOfClass(URL)
        )
      ).once();

      // assert
      equal(actual, expected);

      const [actualUrl] = capture(this.jsonClientMock.get).first();
      equal(actualUrl, testSource.url);
      equal(actual, expected);
    },

    "returns empty when the resource cannot be obtained": async function (this: TestContext) {
      const testResourceUrl = 'https://test'
      const testSource = {
        enabled: true,
        machineWide: false,
        url: testResourceUrl,
        protocol: RegistryProtocols.https
      };

      const errorResponse = {
        source: 'remote',
        status: 404,
        data: 'an error occurred',
        rejected: true
      };

      const expectedUrl = "";

      when(this.jsonClientMock.get(anything())).thenReject(<any>errorResponse);

      // test
      const actual = await this.cut.fetchResource(testSource)

      // verify
      verify(
        this.loggerMock.error(
          "Could not resolve nuget service index {url}. {error}",
          anyOfClass(URL),
          errorResponse
        )
      ).once();

      // assert
      equal(actual, expectedUrl);
    },

  }

}