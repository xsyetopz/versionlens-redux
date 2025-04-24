import { type IJsonHttpClient, ClientResponseSource, JsonClientResponse } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { NuGetClient } from '#domain/providers/dotnet';
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
  jsonClientMock: IJsonHttpClient;
  loggerMock: ILogger;
}

export const NuGetClientTests = {

  title: NuGetClient.name,

  beforeEach: function (this: TestContext) {
    this.jsonClientMock = mock<IJsonHttpClient>();
    this.loggerMock = mock<ILogger>();
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
      const cut = new NuGetClient(instance(this.jsonClientMock), instance(this.loggerMock));
      when(this.jsonClientMock.get(testUrl)).thenResolve(testResp)

      // test
      const actual = await cut.get(testPackageName, [testApiUrl])
      // assert
      deepEqual(actual, expectedResp)
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

      const cut = new NuGetClient(instance(this.jsonClientMock), instance(this.loggerMock))

      // test
      const actual = await cut.get(testPackageName, [failUrl, successUrl])

      // assert
      deepEqual(actual, successResp)
    }
  },

  fetchResource: {
    "": async function (this: TestContext) {
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
      const cut = new NuGetClient(instance(this.jsonClientMock), instance(this.loggerMock))

      // test
      const actual = await cut.fetchResource(testSource);

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

      const cut = new NuGetClient(instance(this.jsonClientMock), instance(this.loggerMock));

      // test
      const actual = await cut.fetchResource(testSource)

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