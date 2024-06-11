import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';
import { ICache } from '#domain/caching';
import { TAsyncFunction } from '#domain/utils';

type CacheMap = {
  [key: string]: any;
};

export class MemoryCache implements ICache {

  cacheMap: CacheMap;

  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cacheName = cacheName
    this.cacheMap = {};
  }

  async getOrCreate<T>(key: string, methodToCache: TAsyncFunction<T>): Promise<T> {
    const cached = this.get<T>(key);
    const result = cached != undefined
      ? cached
      : this.set(key, await methodToCache());

    return result;
  }

  get<T>(key: string): T {
    throwUndefinedOrNull("key", key);
    const value = this.cacheMap[key];
    return value;
  }

  set<T>(key: string, value: T): T {
    throwUndefinedOrNull("key", key);
    this.cacheMap[key] = value;
    return value;
  }

  remove(key: string): void {
    delete this.cacheMap[key];
  }

  clear(): void {
    this.cacheMap = {};
  }

  static createKey(...keyParts: string[]) {
    return keyParts.join("->");
  }

}