import { type ExpiryCacheEntry, type IExpiryCache, MemoryCache } from "#domain/caching";
import type { AsyncFunction } from '#domain/utils';
import { throwNotStringOrEmpty } from "@esm-test/guards";

/**
 * An in-memory cache that supports item expiration.
 * @template T The type of the cached items.
 */
export class MemoryExpiryCache<T = any> implements IExpiryCache<T> {

  /** The underlying memory cache used for storage. */
  cache: MemoryCache<ExpiryCacheEntry<T>>;

  /**
   * Initializes a new instance of the MemoryExpiryCache class.
   * @param cacheName The unique name of the cache.
   */
  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cache = new MemoryCache(cacheName);
  }

  /**
   * Gets a cached item or creates it by calling the provided method, considering expiration.
   * @param key The cache key.
   * @param methodToCache The async method to call if the item is not cached or has expired.
   * @param duration The expiration duration in milliseconds.
   * @returns A promise resolving to the cached or newly created item.
   */
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

  /**
   * Gets an item from the cache if it hasn't expired.
   * @param key The cache key.
   * @param duration The expiration duration in milliseconds.
   * @returns The cached item, or undefined if not found or expired.
   */
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

  /**
   * Sets an item in the cache.
   * @param key The cache key.
   * @param data The data to cache.
   * @returns The cached data.
   */
  set(key: string, data: T): T {
    const createdTime = Date.now();
    const newEntry = { createdTime, data };
    this.cache.set(key, newEntry);
    return newEntry.data;
  }

  /**
   * Clears all items from the cache.
   */
  clear() {
    this.cache.clear();
  }

}