import { type ExpiryCacheEntry, type IExpiryCache, MemoryCache } from "#domain/caching";
import type { AsyncFunction } from '#domain/utils';
import { throwNotStringOrEmpty } from "@esm-test/guards";

export class MemoryExpiryCache<T = any> implements IExpiryCache<T> {

  cache: MemoryCache<ExpiryCacheEntry<T>>;

  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cache = new MemoryCache(cacheName);
  }

  async getOrCreate(
    key: string,
    methodToCache: AsyncFunction<T>,
    duration: number
  ): Promise<T> {
    const cached = this.get(key, duration);
    const result = cached != undefined
      ? cached
      : this.set(key, await methodToCache());

    return result;
  }

  get(key: string, duration: number): T | undefined {
    const entry = this.cache.get(key);
    if (!entry) return undefined;

    // check if the entry has expired
    if (Date.now() >= entry.createdTime + duration) {
      this.cache.remove(key);
      return undefined;
    }

    // return the cached data
    return entry.data;
  }

  set(key: string, data: T): T | undefined {
    const createdTime = Date.now();
    const newEntry = { createdTime, data };
    this.cache.set(key, newEntry);
    return newEntry.data;
  }

  clear() {
    this.cache.clear();
  }

}