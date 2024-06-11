import assert from 'node:assert';
import {
  ClientResponseSource,
  HttpClientRequestMethods,
  IJsonHttpClient,
  JsonHttpClient,
  UrlUtils
} from '#domain/clients';
import { ILogger } from 'domain/logging';
import { NuGetResourceClient } from 'infrastructure/providers/dotnet';
import { LoggerStub } from 'test/unit/domain/logging';
import { anything, capture, instance, mock, when } from 'ts-mockito';
import Fixtures from './fixtures/nugetResources';

let jsonClientMock: IJsonHttpClient;
let loggerMock: ILogger;

export const NuGetResourceClientTests = {

  title: NuGetResourceClient.name,

  beforeEach: () => {
    jsonClientMock = mock(JsonHttpClient);
    loggerMock = mock(LoggerStub);
  },

  "fetchResource": {

    "returns the package resource from a list of resources": async () => {
      const testSource = {
        enabled: true,
        machineWide: false,
        url: 'https://test',
        protocol: UrlUtils.RegistryProtocols.https
      };

      const mockResponse = {
        source: ClientResponseSource.remote,
        status: 200,
        data: Fixtures.success,
      };

      const expected = 'https://api.nuget.org/v3-flatcontainer1/';

      when(jsonClientMock.request(anything(), anything(), anything(), anything()))
        .thenResolve(mockResponse)

      const cut = new NuGetResourceClient(
        instance(jsonClientMock),
        instance(loggerMock)
      )

      return cut.fetchResource(testSource)
        .then(actualSources => {
          assert.equal(actualSources, expected)

          const [actualMethod, actualUrl] = capture(jsonClientMock.request).first();
          assert.equal(actualMethod, HttpClientRequestMethods.get);
          assert.equal(actualUrl, testSource.url);
        });
    },

    "returns empty when the resource cannot be obtained": async () => {

      const testSource = {
        enabled: true,
        machineWide: false,
        url: 'https://test',
        protocol: UrlUtils.RegistryProtocols.https
      };

      const errorResponse = {
        source: 'remote',
        status: 404,
        data: 'an error occurred',
        rejected: true
      };

      const expectedUrl = "";

      when(jsonClientMock.request(anything(), anything(), anything(), anything()))
        .thenReject(<any>errorResponse)

      const cut = new NuGetResourceClient(
        instance(jsonClientMock),
        instance(loggerMock)
      )

      await cut.fetchResource(testSource)
        .then(actualUrl => {
          assert.equal(actualUrl, expectedUrl)
        })
        .catch(err => {
          assert.fail();
        });

    },

  }

}