import { KeyDictionary } from '#domain/utils';

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

export type HttpClientResponse = TClientResponse<number, string>;

export type JsonClientResponse = TClientResponse<number, KeyDictionary<any>>;

export type ProcessClientResponse = TClientResponse<string, string>;