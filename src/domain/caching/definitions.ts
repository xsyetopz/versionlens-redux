import type { TAsyncFunction } from '#domain/utils';

export enum CachingFeatures {
  CacheDuration = 'duration',
}

export interface ICache {

  cacheName: string;

  getOrCreate<T>(key: string, methodToCache: TAsyncFunction<T>): Promise<T>;

  get<T>(key: string): T;

  set<T>(key: string, value: T): T;

  remove(key: string): void;

  clear(): void;

};

export type ExpiryCacheEntry<T> = {
  createdTime: number,
  data: T
};

export interface IExpiryCache {

  getOrCreate<T>(key: string, methodToCache: TAsyncFunction<T>, duration: number): Promise<T>;

  get<T>(key: string, duration: number): T | undefined;

  set<T>(key: string, data: T): T | undefined;

  clear(): void;

}