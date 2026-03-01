import type { CachingOptions, IExpiryCache } from '#domain/caching';
import {
  type IShellClient,
  type ShellClientResponse,
  ClientResponseSource,
  ShellClientRequestError
} from '#domain/clients';
import type { PromiseSpawnFn } from '#domain/clients/promiseSpawn';
import type { ILogger } from '#domain/logging';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for executing shell commands and caching their output.
 */
export class PromiseSpawnClient implements IShellClient {

  /**
   * Initializes a new instance of the PromiseSpawnClient class.
   * @param promiseSpawnFn The function used to spawn processes.
   * @param shellCache The cache for command results.
   * @param cachingOptions Caching options.
   * @param logger The logger for this client.
   */
  constructor(
    readonly promiseSpawnFn: PromiseSpawnFn,
    readonly shellCache: IExpiryCache,
    readonly cachingOptions: CachingOptions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("promiseSpawnFn", promiseSpawnFn);
    throwUndefinedOrNull("shellCache", shellCache);
    throwUndefinedOrNull("cachingOptions", cachingOptions);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Executes a shell command and returns the output, with caching.
   * @param cmd The command to execute.
   * @param args The arguments for the command.
   * @param cwd The working directory for the command.
   * @returns A promise resolving to the shell client response.
   */
  async request(cmd: string, args: Array<string>, cwd: string): Promise<ShellClientResponse> {
    const cacheKey = `${cmd} ${args.join(' ')}`;

    this.logger.trace("executing '{cacheKey}'", cacheKey);

    try {
      let source = ClientResponseSource.cache;
      const result = await this.shellCache.getOrCreate(
        cacheKey,
        async () => {
          source = ClientResponseSource.cli;
          return await this.promiseSpawnFn(cmd, args, { cwd, stdioString: true })
        },
        this.cachingOptions.duration
      )

      this.logger.debug(
        "command result from {source} - '{cacheKey}'",
        source,
        cacheKey
      );

      return <ShellClientResponse>{
        data: result.stdout,
        source,
        status: result.code,
        rejected: false
      };
    } catch (error) {
      throw new ShellClientRequestError(
        `\tcmd: ${cmd}\n`
        + `\targs: ${args}\n`
        + `\tcwd: ${cwd}\n`,
        error as Error
      );
    }

  }

}