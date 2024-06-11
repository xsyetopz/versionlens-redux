import {
  HttpClientOptions,
  IHttpClient,
  IJsonHttpClient,
  JsonHttpClient
} from '#domain/clients';
import { ILogger } from "domain/logging";
import * as RequireLight from 'request-light';
import { RequestLightClient } from "./requestLightClient";

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