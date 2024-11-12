import { ICachingOptions, IExpiryCache } from '#domain/caching';
import {
  ClientResponseSource,
  IShellClient,
  ShellClientResponse
} from '#domain/clients';
import { IPromiseSpawnFn } from '#domain/clients/promiseSpawn';
import { ILogger } from '#domain/logging';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PromiseSpawnClient implements IShellClient {

  constructor(
    readonly promiseSpawnFn: IPromiseSpawnFn,
    readonly shellCache: IExpiryCache,
    readonly cachingOptions: ICachingOptions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("promiseSpawnFn", promiseSpawnFn);
    throwUndefinedOrNull("shellCache", shellCache);
    throwUndefinedOrNull("cachingOptions", cachingOptions);
    throwUndefinedOrNull("logger", logger);
  }

  async request(cmd: string, args: Array<string>, cwd: string): Promise<ShellClientResponse> {
    const cacheKey = `${cmd} ${args.join(' ')}`;

    this.logger.silly('executing %s', cacheKey);

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

      this.logger.debug("command result from %s - '%s'", source, cacheKey);

      return <ShellClientResponse>{
        data: result.stdout,
        source,
        status: result.code,
        rejected: false
      };

    } catch (error) {

      const result = <ShellClientResponse>{
        data: error.message,
        source: ClientResponseSource.cli,
        status: error.code,
        rejected: true
      };

      throw result;
    }

  }

}