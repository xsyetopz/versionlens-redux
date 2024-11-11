import { ICachingOptions, IExpiryCache } from '#domain/caching';
import { IProcessClient } from '#domain/clients';
import { PromiseSpawnClient } from '#domain/clients/promiseSpawn';
import { ILogger } from '#domain/logging';
import PromiseSpawn from '@npmcli/promise-spawn';

export function createProcessClient(
  processCache: IExpiryCache,
  cachingOpts: ICachingOptions,
  logger: ILogger
): IProcessClient {
  return new PromiseSpawnClient(PromiseSpawn, processCache, cachingOpts, logger);
}