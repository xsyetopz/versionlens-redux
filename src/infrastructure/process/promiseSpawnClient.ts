import { throwUndefinedOrNull } from '@esm-test/guards';
import { ICachingOptions, IExpiryCache } from '#domain/caching';
import { ClientResponseSource, IProcessClient, ProcessClientResponse } from '#domain/clients';
import { ILogger } from 'domain/logging';
import { IPromiseSpawnFn } from './iPromiseSpawn';

export class PromiseSpawnClient implements IProcessClient {

  constructor(
    readonly promiseSpawnFn: IPromiseSpawnFn,
    readonly processCache: IExpiryCache,
    readonly cachingOptions: ICachingOptions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("promiseSpawnFn", promiseSpawnFn);
    throwUndefinedOrNull("processCache", processCache);
    throwUndefinedOrNull("cachingOptions", cachingOptions);
    throwUndefinedOrNull("logger", logger);
  }

  async request(cmd: string, args: Array<string>, cwd: string): Promise<ProcessClientResponse> {
    const cacheKey = `${cmd} ${args.join(' ')}`;

    this.logger.silly('executing %s', cacheKey);

    try {
      let source = ClientResponseSource.cache;
      const result = await this.processCache.getOrCreate(
        cacheKey,
        async () => {
          source = ClientResponseSource.cli;
          return await this.promiseSpawnFn(cmd, args, { cwd, stdioString: true })
        },
        this.cachingOptions.duration
      )

      this.logger.debug("command result from %s - '%s'", source, cacheKey);

      return <ProcessClientResponse>{
        data: result.stdout,
        source,
        status: result.code,
        rejected: false
      };

    } catch (error) {

      const result = <ProcessClientResponse>{
        data: error.message,
        source: ClientResponseSource.cli,
        status: error.code,
        rejected: true
      };

      throw result;
    }

  }

}