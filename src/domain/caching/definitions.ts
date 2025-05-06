import type { AsyncFunction } from '#domain/utils';

export enum CachingFeatures {
  CacheDuration = 'duration',
}

export interface ICache<T = any> {

  cacheName: string;

  getOrCreate(key: string, methodToCache: AsyncFunction<T>): Promise<T | undefined>;

  get(key: string): T | undefined;

  set(key: string, value: T): T;

  remove(key: string): void;

  clear(): void;

  keys(): MapIterator<string>

};

export type ExpiryCacheEntry<T> = {
  createdTime: number,
  data: T
};

export interface IExpiryCache<T = any> {

  getOrCreate(key: string, methodToCache: AsyncFunction<T>, duration: number): Promise<T>;

  get(key: string, duration: number): T | undefined;

  set(key: string, data: T): T;

  clear(): void;

}