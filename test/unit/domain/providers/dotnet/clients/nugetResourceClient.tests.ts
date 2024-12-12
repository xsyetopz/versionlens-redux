import { type IJsonHttpClient, ClientResponseSource, JsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { NuGetResourceClient } from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import assert from 'node:assert';
import { anyOfClass, anything, capture, instance, mock, verify, when } from 'ts-mockito';
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

      // test
      const actual = await cut.fetchResource(testSource);

      // verify
      verify(
        loggerMock.debug(
          "Resolved PackageBaseAddressService endpoint: {url}",
          anyOfClass(URL)
        )
      ).once();

      // assert
      assert.equal(actual, expected);

      const [actualUrl] = capture(jsonClientMock.get).first();
      assert.equal(actualUrl, testSource.url);
      assert.equal(actual, expected);
    },

    "returns empty when the resource cannot be obtained": async () => {
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

      when(jsonClientMock.get(anything())).thenReject(<any>errorResponse);

      const cut = new NuGetResourceClient(instance(jsonClientMock), instance(loggerMock));

      // test
      const actual = await cut.fetchResource(testSource)

      // verify
      verify(
        loggerMock.error(
          "Could not resolve nuget service index {url}. {error}",
          anyOfClass(URL),
          errorResponse
        )
      ).once();

      // assert
      assert.equal(actual, expectedUrl);
    },

  }

}