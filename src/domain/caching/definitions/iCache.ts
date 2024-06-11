import { TAsyncFunction } from '#domain/utils';

export interface ICache {

  cacheName: string;

  getOrCreate<T>(key: string, methodToCache: TAsyncFunction<T>): Promise<T>;

  get<T>(key: string): T;

  set<T>(key: string, value: T): T;

  remove(key: string): void;

  clear(): void;

};