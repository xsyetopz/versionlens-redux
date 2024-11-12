import { ICachingOptions } from '#domain/caching';
import { IFrozenOptions } from '#domain/configuration';
import { KeyDictionary, KeyStringDictionary } from '#domain/utils';

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
  caching: ICachingOptions,
  http: IHttpOptions,
}

export type HttpClientResponse = TClientResponse<number, string>;

export enum HttpClientRequestMethods {
  get = 'GET',
  head = 'HEAD'
}

export interface THttpClientRequestFn {
  (
    method: HttpClientRequestMethods,
    url: string,
    query: KeyStringDictionary,
    headers: KeyStringDictionary,
  ): Promise<HttpClientResponse>;
}

export interface IHttpClient {
  request: THttpClientRequestFn;
}

export type JsonClientResponse = TClientResponse<number, KeyDictionary<any>>;

export interface IJsonHttpClient {
  httpClient: IHttpClient;
  request: (
    method: HttpClientRequestMethods,
    url: string,
    query: KeyStringDictionary,
    headers: KeyStringDictionary,
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