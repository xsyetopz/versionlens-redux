import type { CachingOptions } from '#domain/caching';
import {
  type HttpOptions,
  type IHttpOptions,
  type IShellClient,
  ClientResponseSource,
} from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type DotNetConfig,
  type INugetOptions,
  type NugetOptions,
  DotNetCli,
} from '#domain/providers/dotnet';
import { RegistryProtocols } from '#domain/utils';
import assert from 'node:assert';
import { anything, deepEqual, instance, mock, verify, when } from 'ts-mockito';
import Fixtures from './fixtures/dotnetSources';

type TestContext = {
  cacheOptsMock: CachingOptions
  httpOptsMock: IHttpOptions
  nugetOptsMock: INugetOptions
  configMock: DotNetConfig
  loggerMock: ILogger
  clientMock: IShellClient
}

export const DotNetCliTests = {

  title: DotNetCli.name,

  beforeEach: function (this: TestContext) {
    this.cacheOptsMock = mock<CachingOptions>();
    this.httpOptsMock = mock<HttpOptions>();
    this.nugetOptsMock = mock<NugetOptions>();
    this.configMock = mock<DotNetConfig>();
    this.loggerMock = mock<ILogger>()
    this.clientMock = mock()

    when(this.configMock.caching).thenReturn(instance(this.cacheOptsMock))
    when(this.configMock.http).thenReturn(instance(this.httpOptsMock))
    when(this.configMock.nuget).thenReturn(instance(this.nugetOptsMock))
  },

  fetchSources: {

    "returns an Array<DotNetSource> of enabled sources": async function (this: TestContext) {
      const testCwd = '.';
      const testFeeds = [
        'https://test.feed/v3/index.json',
      ];

      const expected = [
        {
          enabled: true,
          machineWide: false,
          url: testFeeds[0],
          protocol: RegistryProtocols.https
        },
        {
          enabled: true,
          machineWide: false,
          url: 'https://api.nuget.org/v3/index.json',
          protocol: RegistryProtocols.https
        },
        {
          enabled: true,
          machineWide: false,
          url: 'http://non-ssl/v3/index.json',
          protocol: RegistryProtocols.http
        },
        {
          enabled: true,
          machineWide: true,
          url: 'C:\\Program Files (x86)\\Microsoft SDKs\\NuGetPackages\\',
          protocol: RegistryProtocols.file
        },
      ]

      when(this.clientMock.request(DotNetCli.command, DotNetCli.fetchSourceArgs, testCwd))
        .thenResolve({
          source: ClientResponseSource.local,
          status: "200",
          data: Fixtures.enabledSources
        })

      when(this.nugetOptsMock.sources).thenReturn(testFeeds)

      const cut = new DotNetCli(
        instance(this.configMock),
        instance(this.clientMock),
        instance(this.loggerMock)
      );

      const actualSources = await cut.fetchSources(testCwd)

      verify(
        this.loggerMock.debug(
          "package sources found: %s",
          deepEqual(actualSources.map(x => x.url))
        )
      ).once();

      assert.deepEqual(actualSources, expected);
    },

    "return 0 items when no sources are enabled": async function (this: TestContext) {
      const testCwd = '.';
      const testFeeds: Array<string> = [];

      when(this.clientMock.request(DotNetCli.command, DotNetCli.fetchSourceArgs, testCwd))
        .thenResolve({
          source: ClientResponseSource.local,
          status: "200",
          data: Fixtures.disabledSource
        })

      when(this.nugetOptsMock.sources).thenReturn(testFeeds)

      const cut = new DotNetCli(
        instance(this.configMock),
        instance(this.clientMock),
        instance(this.loggerMock)
      );

      const actualSources = await cut.fetchSources(testCwd)

      assert.equal(actualSources.length, 0);
    },

    "returns only enabled sources when some sources are disabled":
      async function (this: TestContext) {
        const testCwd = '.';
        const expected = [
          {
            enabled: true,
            machineWide: false,
            url: 'https://api.nuget.org/v3/index.json',
            protocol: RegistryProtocols.https
          },
        ]

        when(this.clientMock.request(DotNetCli.command, DotNetCli.fetchSourceArgs, testCwd))
          .thenResolve({
            source: ClientResponseSource.local,
            status: "200",
            data: Fixtures.enabledAndDisabledSources
          })

        when(this.nugetOptsMock.sources).thenReturn([])

        const cut = new DotNetCli(
          instance(this.configMock),
          instance(this.clientMock),
          instance(this.loggerMock)
        );

        const actualSources = await cut.fetchSources(testCwd)

        assert.deepEqual(actualSources, expected);
      },

    "returns fallback url on error": async function (this: TestContext) {
      const testCwd = '.';
      const expectedFallbackNugetSource = 'http://fallbackurl.test.net'

      when(this.clientMock.request(DotNetCli.command, DotNetCli.fetchSourceArgs, testCwd))
        .thenReject()

      when(this.configMock.fallbackNugetSource).thenReturn(expectedFallbackNugetSource)

      const cut = new DotNetCli(
        instance(this.configMock),
        instance(this.clientMock),
        instance(this.loggerMock)
      );

      const expectedErrorResp = {
        enabled: true,
        machineWide: false,
        protocol: 'https:',
        url: expectedFallbackNugetSource,
      }

      const actual = await cut.fetchSources(testCwd)

      verify(
        this.loggerMock.error(
          "failed to get package sources: %s",
          anything()
        )
      ).once();

      verify(
        this.loggerMock.info(
          "using fallback source: %s",
          expectedFallbackNugetSource
        )
      ).once();

      assert.deepEqual(actual, [expectedErrorResp]);
    },

  }

}