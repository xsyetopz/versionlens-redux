import type { AsyncFunction } from '#domain/utils';

/**
 * Feature keys used for caching configuration.
 */
export enum CachingFeatures {
  /** The duration for which items are cached. */
  CacheDuration = 'duration',
}

/**
 * Interface for a generic cache.
 * @template T The type of the cached items.
 */
export interface ICache<T = any> {

  /** The name of the cache. */
  cacheName: string;

  /**
   * Gets a cached item or creates it by calling the provided method.
   * @param key The cache key.
   * @param methodToCache The async method to call if the item is not cached.
   * @returns A promise resolving to the cached or newly created item.
   */
  getOrCreate(key: string, methodToCache: AsyncFunction<T>): Promise<T | undefined>;

  /**
   * Gets an item from the cache.
   * @param key The cache key.
   * @returns The cached item, or undefined if not found.
   */
  get(key: string): T | undefined;

  /**
   * Sets an item in the cache.
   * @param key The cache key.
   * @param value The value to cache.
   * @returns The cached value.
   */
  set(key: string, value: T): T;

  /**
   * Removes an item from the cache.
   * @param key The cache key.
   */
  remove(key: string): void;

  /**
   * Clears all items from the cache.
   */
  clear(): void;

  /**
   * Gets an iterator for all keys in the cache.
   * @returns A map iterator of cache keys.
   */
  keys(): MapIterator<string>

};

/**
 * Represents an entry in an expiry cache.
 * @template T The type of the cached data.
 */
export type ExpiryCacheEntry<T> = {
  /** The time when the entry was created. */
  createdTime: number,
  /** The cached data. */
  data: T
};

/**
 * Interface for a cache with item expiration.
 * @template T The type of the cached items.
 */
export interface IExpiryCache<T = any> {

  /**
   * Gets a cached item or creates it by calling the provided method, considering expiration.
   * @param key The cache key.
   * @param methodToCache The async method to call if the item is not cached or has expired.
   * @param duration The expiration duration in milliseconds.
   * @returns A promise resolving to the cached or newly created item.
   */
  getOrCreate(key: string, methodToCache: AsyncFunction<T>, duration: number): Promise<T>;

  /**
   * Gets an item from the cache if it hasn't expired.
   * @param key The cache key.
   * @param duration The expiration duration in milliseconds.
   * @returns The cached item, or undefined if not found or expired.
   */
  get(key: string, duration: number): T | undefined;

  /**
   * Sets an item in the cache.
   * @param key The cache key.
   * @param data The data to cache.
   * @returns The cached data.
   */
  set(key: string, data: T): T;

  /**
   * Clears all items from the cache.
   */
  clear(): void;

}