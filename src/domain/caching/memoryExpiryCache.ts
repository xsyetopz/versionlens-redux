import { throwNotStringOrEmpty } from "@esm-test/guards";
import { ExpiryCacheEntry, IExpiryCache, MemoryCache } from "#domain/caching";
import { TAsyncFunction } from '#domain/utils';

export class MemoryExpiryCache implements IExpiryCache {

  cache: MemoryCache;

  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cache = new MemoryCache(cacheName);
  }

  async getOrCreate<T>(
    key: string,
    methodToCache: TAsyncFunction<T>,
    duration: number
  ): Promise<T> {
    const cached = this.get<T>(key, duration);
    const result = cached != undefined
      ? cached
      : this.set(key, await methodToCache());

    return result;
  }

  get<T>(key: string, duration: number): T {
    const entry = this.cache.get<ExpiryCacheEntry<T>>(key);
    if (!entry) {
      return undefined;
    }

    // check if the entry has expired
    if (Date.now() >= entry.createdTime + duration) {
      this.cache.remove(key);
      return undefined;
    }

    // return the cached data
    return entry.data;
  }

  set<T>(key: string, data: T): T {
    const createdTime = Date.now();
    const newEntry = {
      createdTime,
      data
    };
    this.cache.set<ExpiryCacheEntry<T>>(key, newEntry);
    return newEntry.data;
  }

  clear() {
    this.cache.clear();
  }

}