import type { ICache } from '#domain/caching';
import type { AsyncFunction } from '#domain/utils';
import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';

export class MemoryCache<T> implements ICache<T> {

  cacheMap: Map<string, T>;

  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cacheName = cacheName
    this.cacheMap = new Map();
  }

  async getOrCreate(key: string, methodToCache: AsyncFunction<T>): Promise<T | undefined> {
    const cached = this.get(key);
    const result = cached != undefined
      ? cached
      : this.set(key, await methodToCache());

    return result;
  }

  get(key: string): T | undefined {
    throwUndefinedOrNull("key", key);
    const value = this.cacheMap.get(key);
    return value;
  }

  set(key: string, value: T): T {
    throwUndefinedOrNull("key", key);
    this.cacheMap.set(key, value);
    return value;
  }

  remove(key: string): void {
    this.cacheMap.delete(key);
  }

  clear(): void {
    this.cacheMap.clear();
  }

  [Symbol.iterator]() {
    return this.cacheMap.entries()
  }

}