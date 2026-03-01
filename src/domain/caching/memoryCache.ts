import type { ICache } from '#domain/caching';
import type { AsyncFunction } from '#domain/utils';
import { throwNotStringOrEmpty, throwUndefinedOrNull } from '@esm-test/guards';

/**
 * A simple in-memory cache implementation.
 * @template T The type of the cached items.
 */
export class MemoryCache<T> implements ICache<T> {

  /** The internal map used for storage. */
  cacheMap: Map<string, T>;

  /**
   * Initializes a new instance of the MemoryCache class.
   * @param cacheName The unique name of the cache.
   */
  constructor(readonly cacheName: string) {
    throwNotStringOrEmpty("cacheName", cacheName);

    this.cacheName = cacheName
    this.cacheMap = new Map();
  }

  /**
   * Gets a cached item or creates it by calling the provided method.
   * @param key The cache key.
   * @param methodToCache The async method to call if the item is not cached.
   * @returns A promise resolving to the cached or newly created item.
   */
  async getOrCreate(key: string, methodToCache: AsyncFunction<T>): Promise<T | undefined> {
    const cached = this.get(key);
    const result = cached != undefined
      ? cached
      : this.set(key, await methodToCache());

    return result;
  }

  /**
   * Gets an item from the cache.
   * @param key The cache key.
   * @returns The cached item, or undefined if not found.
   */
  get(key: string): T | undefined {
    throwUndefinedOrNull("key", key);
    const value = this.cacheMap.get(key);
    return value;
  }

  /**
   * Sets an item in the cache.
   * @param key The cache key.
   * @param value The value to cache.
   * @returns The cached value.
   */
  set(key: string, value: T): T {
    throwUndefinedOrNull("key", key);
    this.cacheMap.set(key, value);
    return value;
  }

  /**
   * Removes an item from the cache.
   * @param key The cache key.
   */
  remove(key: string): void {
    this.cacheMap.delete(key);
  }

  /**
   * Clears all items from the cache.
   */
  clear(): void {
    this.cacheMap.clear();
  }

  /**
   * Gets an iterator for all keys in the cache.
   * @returns A map iterator of cache keys.
   */
  keys(): MapIterator<string> {
    return this.cacheMap.keys()
  }

}