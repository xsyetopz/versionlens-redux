import { TAsyncFunction } from '#domain/utils';

export type ExpiryCacheEntry<T> = {
  createdTime: number,
  data: T
};

export interface IExpiryCache {

  getOrCreate<T>(key: string, methodToCache: TAsyncFunction<T>, duration: number): Promise<T>;

  get<T>(key: string, duration: number): T;

  set<T>(key: string, data: T): T;

  clear(): void;

}