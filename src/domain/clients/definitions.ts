import type { CachingOptions } from '#domain/caching';
import type { HttpOptions } from '#domain/clients';
import type { KeyDictionary, KeyStringDictionary } from '#domain/utils';

export enum ClientResponseSource {
  remote = 'remote',
  cache = 'cache',
  local = 'local',
  cli = 'cli'
}

export type ClientResponse<TStatus, TData> = {
  source: ClientResponseSource;
  status: TStatus;
  data: TData;
  rejected?: boolean;
}

export enum HttpFeatures {
  StrictSSL = 'strictSSL'
}

export type HttpClientOptions = {
  caching: CachingOptions,
  http: HttpOptions,
}

export type HttpClientResponse = ClientResponse<number, string>;

export enum HttpClientRequestMethods {
  get = 'GET',
  head = 'HEAD'
}

export type QueryDictionary = KeyDictionary<string | number | boolean>

export interface THttpClientRequestFn {
  (
    url: string,
    query?: QueryDictionary,
    headers?: KeyStringDictionary,
  ): Promise<HttpClientResponse>;
}

export interface IHttpClient {
  get: THttpClientRequestFn;
}

export type JsonClientResponse<TData> = ClientResponse<number, TData>;

export interface IJsonHttpClient {
  httpClient: IHttpClient;
  get<TData = KeyDictionary<any>>(
    url: string,
    query?: QueryDictionary,
    headers?: KeyStringDictionary,
  ): Promise<JsonClientResponse<TData>>;
}

export type ShellClientResponse = ClientResponse<string, string>;

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

export type GithubTagsApiResult = [{ name: string }]

export type GithubCommitsApiResult = [{ sha: string }]

export type GithubJsonClientResponse = JsonClientResponse<string[]>