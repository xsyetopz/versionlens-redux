import {
  HttpClientOptions,
  IHttpClient,
  IJsonHttpClient,
  JsonHttpClient
} from '#domain/clients';
import { RequestLightClient } from "#domain/clients/requestLight";
import { ILogger } from '#domain/logging';
import * as RequireLight from 'request-light';

export function createHttpClient(
  options: HttpClientOptions,
  logger: ILogger
): IHttpClient {
  return new RequestLightClient(RequireLight.xhr, options, logger);
}

export function createJsonClient(
  options: HttpClientOptions,
  logger: ILogger
): IJsonHttpClient {
  return new JsonHttpClient(createHttpClient(options, logger));
}