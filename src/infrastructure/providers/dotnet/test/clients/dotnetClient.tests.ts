import assert from 'node:assert';
import { CachingOptions, ICachingOptions } from '#domain/caching';
import { ClientResponseSource, IProcessClient, UrlUtils } from '#domain/clients';
import { HttpOptions, IHttpOptions } from '#domain/http';
import { ILogger } from 'domain/logging';
import {
  DotNetCli,
  DotNetConfig,
  INugetOptions,
  NugetOptions
} from 'infrastructure/providers/dotnet';
import { LoggerStub } from 'test/unit/domain/logging';
import { anything, instance, mock, verify, when } from 'ts-mockito';
import Fixtures from './fixtures/dotnetSources';

let cacheOptsMock: ICachingOptions;
let httpOptsMock: IHttpOptions;
let nugetOptsMock: INugetOptions;
let configMock: DotNetConfig;
let loggerMock: ILogger;
let clientMock: IProcessClient;

export const DotNetCliTests = {

  title: DotNetCli.name,

  beforeEach: () => {
    cacheOptsMock = mock(CachingOptions);
    httpOptsMock = mock(HttpOptions);
    nugetOptsMock = mock(NugetOptions);
    configMock = mock(DotNetConfig);
    loggerMock = mock(LoggerStub)
    clientMock = mock()

    when(configMock.caching).thenReturn(instance(cacheOptsMock))
    when(configMock.http).thenReturn(instance(httpOptsMock))
    when(configMock.nuget).thenReturn(instance(nugetOptsMock))
  },

  fetchSources: {

    "returns an Array<DotNetSource> of enabled sources": async () => {
      const testFeeds = [
        'https://test.feed/v3/index.json',
      ];

      const expected = [
        {
          enabled: true,
          machineWide: false,
          url: testFeeds[0],
          protocol: UrlUtils.RegistryProtocols.https
        },
        {
          enabled: true,
          machineWide: false,
          url: 'https://api.nuget.org/v3/index.json',
          protocol: UrlUtils.RegistryProtocols.https
        },
        {
          enabled: true,
          machineWide: false,
          url: 'http://non-ssl/v3/index.json',
          protocol: UrlUtils.RegistryProtocols.http
        },
        {
          enabled: true,
          machineWide: true,
          url: 'C:\\Program Files (x86)\\Microsoft SDKs\\NuGetPackages\\',
          protocol: UrlUtils.RegistryProtocols.file
        },
      ]

      when(clientMock.request(anything(), anything(), anything()))
        .thenResolve({
          source: ClientResponseSource.local,
          status: "200",
          data: Fixtures.enabledSources
        })

      when(nugetOptsMock.sources).thenReturn(testFeeds)

      const cut = new DotNetCli(
        instance(configMock),
        instance(clientMock),
        instance(loggerMock)
      );

      const actualSources = await cut.fetchSources('.')

      verify(
        loggerMock.debug(
          "package sources found: %s",
          anything()
        )
      ).once();

      assert.deepEqual(actualSources, expected);
    },

    "return 0 items when no sources are enabled": async () => {
      const testFeeds: Array<string> = [];

      when(clientMock.request(
        anything(),
        anything(),
        anything()
      )).thenResolve({
        source: ClientResponseSource.local,
        status: "200",
        data: Fixtures.disabledSource
      })

      when(nugetOptsMock.sources).thenReturn(testFeeds)

      const cut = new DotNetCli(
        instance(configMock),
        instance(clientMock),
        instance(loggerMock)
      );

      const actualSources = await cut.fetchSources('.')

      assert.equal(actualSources.length, 0);
    },

    "returns only enabled sources when some sources are disabled": async () => {
      const expected = [
        {
          enabled: true,
          machineWide: false,
          url: 'https://api.nuget.org/v3/index.json',
          protocol: UrlUtils.RegistryProtocols.https
        },
      ]

      when(clientMock.request(
        anything(),
        anything(),
        anything()
      )).thenResolve({
        source: ClientResponseSource.local,
        status: "200",
        data: Fixtures.enabledAndDisabledSources
      })

      when(nugetOptsMock.sources).thenReturn([])

      const cut = new DotNetCli(
        instance(configMock),
        instance(clientMock),
        instance(loggerMock)
      );

      const actualSources = await cut.fetchSources('.')

      assert.deepEqual(actualSources, expected);
    },

    "returns fallback url on error": async () => {
      const expectedFallbackNugetSource = 'http://fallbackurl.test.net'

      when(clientMock.request(
        anything(),
        anything(),
        anything()
      )).thenReject()

      when(configMock.fallbackNugetSource).thenReturn(expectedFallbackNugetSource)

      const cut = new DotNetCli(
        instance(configMock),
        instance(clientMock),
        instance(loggerMock)
      );

      const expectedErrorResp = {
        enabled: true,
        machineWide: false,
        protocol: 'https:',
        url: expectedFallbackNugetSource,
      }

      const actual = await cut.fetchSources('.')

      verify(
        loggerMock.error(
          "failed to get package sources: %s",
          anything()
        )
      ).once();

      verify(
        loggerMock.info(
          "using fallback source: %s",
          expectedFallbackNugetSource
        )
      ).once();

      assert.deepEqual(actual, [expectedErrorResp]);
    },

  }

}