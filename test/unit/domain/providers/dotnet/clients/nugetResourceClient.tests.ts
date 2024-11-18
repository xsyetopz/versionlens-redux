import { type IJsonHttpClient, ClientResponseSource, JsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { NuGetResourceClient } from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import assert from 'node:assert';
import { anything, capture, instance, mock, when } from 'ts-mockito';
import Fixtures from './fixtures/nugetResources';

let jsonClientMock: IJsonHttpClient;
let loggerMock: ILogger;

export const NuGetResourceClientTests = {

  title: NuGetResourceClient.name,

  beforeEach: () => {
    jsonClientMock = mock(JsonHttpClient);
    loggerMock = mock<ILogger>();
  },

  "fetchResource": {

    "returns the package resource from a list of resources": async () => {
      const testSource = {
        enabled: true,
        machineWide: false,
        url: 'https://test',
        protocol: RegistryProtocols.https
      };

      const mockResponse = {
        source: ClientResponseSource.remote,
        status: 200,
        data: Fixtures.success,
      };

      const expected = 'https://api.nuget.org/v3-flatcontainer1/';

      when(jsonClientMock.get(anything())).thenResolve(mockResponse)

      const cut = new NuGetResourceClient(instance(jsonClientMock), instance(loggerMock))

      return cut.fetchResource(testSource)
        .then(actualSources => {
          assert.equal(actualSources, expected)

          const [actualUrl] = capture(jsonClientMock.get).first();
          assert.equal(actualUrl, testSource.url);
        });
    },

    "returns empty when the resource cannot be obtained": async () => {

      const testSource = {
        enabled: true,
        machineWide: false,
        url: 'https://test',
        protocol: RegistryProtocols.https
      };

      const errorResponse = {
        source: 'remote',
        status: 404,
        data: 'an error occurred',
        rejected: true
      };

      const expectedUrl = "";

      when(jsonClientMock.get(anything(), anything(), anything()))
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