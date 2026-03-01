import type { IAuthorizer } from '#domain/authorization';
import type { CachingOptions, IExpiryCache } from '#domain/caching';
import {
  type HttpClientOptions,
  type IHttpClient,
  type IJsonHttpClient,
  type IShellClient,
  JsonHttpClient
} from '#domain/clients';
import { PromiseSpawnClient } from '#domain/clients/promiseSpawn';
import { RequestLightClient } from "#domain/clients/requestLight";
import type { ILogger } from '#domain/logging';
import PromiseSpawn from '@npmcli/promise-spawn';
import * as RequireLight from 'request-light';

/**
 * Creates a shell client using promise-spawn.
 * @param shellCache The cache for shell command responses.
 * @param cachingOpts Caching options.
 * @param logger The logger to use.
 * @returns An implementation of IShellClient.
 */
export function createShellClient(
  shellCache: IExpiryCache,
  cachingOpts: CachingOptions,
  logger: ILogger
): IShellClient {
  return new PromiseSpawnClient(PromiseSpawn, shellCache, cachingOpts, logger);
}

/**
 * Creates an HTTP client using request-light.
 * @param authorizer The authorizer for handling credentials.
 * @param options HTTP client options.
 * @returns An implementation of IHttpClient.
 */
export function createHttpClient(
  authorizer: IAuthorizer,
  options: HttpClientOptions
): IHttpClient {
  return new RequestLightClient(RequireLight, authorizer, options);
}

/**
 * Creates a JSON HTTP client.
 * @param authorizer The authorizer for handling credentials.
 * @param options HTTP client options.
 * @returns An implementation of IJsonHttpClient.
 */
export function createJsonClient(
  authorizer: IAuthorizer,
  options: HttpClientOptions
): IJsonHttpClient {
  return new JsonHttpClient(
    createHttpClient(authorizer, options)
  );
}