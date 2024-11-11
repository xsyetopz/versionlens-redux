import { ICachingOptions, IExpiryCache } from '#domain/caching';
import { IProcessClient } from '#domain/clients';
import { ILogger } from '#domain/logging';
import { PromiseSpawnClient } from '#domain/process/promiseSpawn';
import PromiseSpawn from '@npmcli/promise-spawn';

export function createProcessClient(
  processCache: IExpiryCache,
  cachingOpts: ICachingOptions,
  logger: ILogger
): IProcessClient {
  return new PromiseSpawnClient(PromiseSpawn, processCache, cachingOpts, logger);
}