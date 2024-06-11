import PromiseSpawn from '@npmcli/promise-spawn';
import { ICachingOptions, IExpiryCache } from '#domain/caching';
import { IProcessClient } from '#domain/clients';
import { ILogger } from 'domain/logging';
import { PromiseSpawnClient } from './promiseSpawnClient';

export function createProcessClient(
  processCache: IExpiryCache,
  cachingOpts: ICachingOptions,
  logger: ILogger
): IProcessClient {
  return new PromiseSpawnClient(PromiseSpawn, processCache, cachingOpts, logger);
}