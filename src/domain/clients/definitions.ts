import type { CachingOptions } from '#domain/caching';
import type { IFrozenOptions } from '#domain/configuration';
import type { KeyDictionary, KeyStringDictionary } from '#domain/utils';

export enum ClientResponseSource {
  remote = 'remote',
  cache = 'cache',
  local = 'local',
  cli = 'cli'
}

export type TClientResponse<TStatus, TData> = {
  source: ClientResponseSource;
  status: TStatus;
  data: TData;
  rejected?: boolean;
}

export enum HttpFeatures {
  StrictSSL = 'strictSSL'
}

export interface IHttpOptions extends IFrozenOptions {
  config: IFrozenOptions;
  strictSSL: boolean;
}

export type HttpClientOptions = {
  caching: CachingOptions,
  http: IHttpOptions,
}

export type HttpClientResponse = TClientResponse<number, string>;

export enum HttpClientRequestMethods {
  get = 'GET',
  head = 'HEAD'
}

export interface THttpClientRequestFn {
  (
    url: string,
    query?: KeyStringDictionary,
    headers?: KeyStringDictionary,
  ): Promise<HttpClientResponse>;
}

export interface IHttpClient {
  get: THttpClientRequestFn;
}

export type JsonClientResponse = TClientResponse<number, KeyDictionary<any>>;

export interface IJsonHttpClient {
  httpClient: IHttpClient;
  get: (
    url: string,
    query?: KeyStringDictionary,
    headers?: KeyStringDictionary,
  ) => Promise<JsonClientResponse>;
}

export type ShellClientResponse = TClientResponse<string, string>;

export interface ShellClientRequestFn {
  (
    cmd: string,
    args: Array<string>,
    cwd: string,
  ): Promise<ShellClientResponse>
}

export interface IShellClient {
  request: ShellClientRequestFn;
}