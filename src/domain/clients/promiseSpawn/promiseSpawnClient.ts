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

export class PromiseSpawnClient implements IShellClient {

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
        error
      );
    }

  }

}