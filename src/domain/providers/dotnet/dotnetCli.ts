import type { IShellClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DotNetConfig, DotNetSource } from '#domain/providers/dotnet';
import { CrLf, getProtocolFromUrl, Lf, RegistryProtocols } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for interacting with the dotnet CLI to fetch package sources.
 */
export class DotNetCli {

  /**
   * The command to execute.
   */
  static command = "dotnet";

  /**
   * The arguments to fetch the list of NuGet sources.
   */
  static fetchSourceArgs = ['nuget', 'list', 'source', '--format', 'short'];

  /**
   * Initializes a new instance of the DotNetCli class.
   * @param config The configuration for the DotNet provider.
   * @param shellClient The shell client for executing commands.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: DotNetConfig,
    readonly shellClient: IShellClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("shellClient", shellClient);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Fetches the configured NuGet sources for a given directory.
   * @param cwd The current working directory.
   * @returns A promise resolving to an array of DotNet sources.
   */
  async fetchSources(cwd: string): Promise<Array<DotNetSource>> {
    this.logger.debug(
      "executing '{command} {args}'",
      DotNetCli.command,
      DotNetCli.fetchSourceArgs.join(' ')
    );

    try {
      const result = await this.shellClient.request(
        DotNetCli.command,
        DotNetCli.fetchSourceArgs,
        cwd
      );

      const { data } = result;

      // reject when data contains "error"
      if (data.indexOf("error") > -1) return Promise.reject(result);

      // check we have some data
      if (data.length === 0 || data.indexOf('E') === -1) {
        return [];
      }

      // extract sources
      const hasCrLf = data.indexOf(CrLf) > 0;
      const splitChar = hasCrLf ? CrLf : Lf;
      let lines = data.split(splitChar);

      // pop any blank entries
      if (lines[lines.length - 1] === '') lines.pop();

      // parse the sources
      const sources = parseSourcesArray(lines).filter(s => s.enabled);

      // combine the sources where user feed settings takes precedent
      const feedSources = convertFeedsToSources(this.config.nugetOptions.sources);
      const combinedSources = [
        ...feedSources,
        ...sources
      ];

      // log combinedSources for diagnostics
      this.logger.debug(
        "package sources found: {packageSources}",
        combinedSources.map(x => new URL(x.url))
      )

      return combinedSources;

    } catch (error) {
      this.logger.error(
        "failed to get package sources: {error}",
        error
      )

      this.logger.info(
        "using fallback source: {fallbackSource}",
        this.config.fallbackNugetSource
      )

      // return the fallback source for dotnet clients < 5.5
      return [
        <DotNetSource>{
          enabled: true,
          machineWide: false,
          protocol: RegistryProtocols.https,
          url: this.config.fallbackNugetSource,
        }
      ]
    }
  }
}

/**
 * Parses the raw output from the dotnet nuget list source command.
 * @param lines The lines of output from the command.
 * @returns An array of parsed DotNet sources.
 */
function parseSourcesArray(lines: Array<string>): Array<DotNetSource> {
  return lines.map(function (line) {
    const enabled = line.substring(0, 1) === 'E';
    const machineWide = line.substring(1, 2) === 'M';
    const offset = machineWide ? 3 : 2;
    const url = line.substring(offset);
    const protocol = getProtocolFromUrl(url);
    return {
      enabled,
      machineWide,
      url,
      protocol
    };
  });
}

/**
 * Converts a list of feed URLs into DotNetSource objects.
 * @param feeds The array of feed URLs.
 * @returns An array of DotNet sources.
 */
function convertFeedsToSources(feeds: Array<string>): Array<DotNetSource> {
  return feeds.map(function (url: string) {
    const protocol = getProtocolFromUrl(url);
    const machineWide = (protocol === RegistryProtocols.file);
    return {
      enabled: true,
      machineWide,
      url,
      protocol
    };
  });
}