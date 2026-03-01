import { type IExpiryCache, MemoryExpiryCache } from '#domain/caching';
import type { PackageClientResponse, PackageResource } from "#domain/packages";
import type { KeyDictionary, AsyncFunction } from '#domain/utils';
import { throwUndefinedOrNull } from "@esm-test/guards";

/**
 * Caches package suggestion responses, keyed by provider and package version.
 */
export class PackageCache {

  /**
   * Internal maps of caches for each provider.
   */
  readonly providerMaps: KeyDictionary<KeyDictionary<IExpiryCache>> = {};

  /**
   * Initializes a new instance of the PackageCache class.
   * @param providerNames List of provider names to initialize caches for.
   */
  constructor(providerNames: Array<string>) {
    throwUndefinedOrNull("providerNames", providerNames);

    providerNames.forEach(
      k => this.providerMaps[k] = {}
    );
  }

  /**
   * Gets a cached suggestion response or creates it by calling the provided method.
   * @param providerName The name of the provider.
   * @param packageRes The package resource information.
   * @param methodToCache The async method to call if the response is not cached.
   * @param duration The cache duration in milliseconds.
   * @returns A promise resolving to the package client response.
   */
  getOrCreate(
    providerName: string,
    packageRes: PackageResource,
    methodToCache: AsyncFunction<PackageClientResponse>,
    duration: number
  ): Promise<PackageClientResponse> {
    // get the packages map for the provider
    const packageMaps = this.providerMaps[providerName];

    // get or create the cache for the package
    let packageCache = packageMaps[packageRes.name];
    if (!packageCache) {
      packageCache = packageMaps[packageRes.name] = new MemoryExpiryCache(`${packageRes.name}-cache`);
    }

    // get or create the cache entry
    return packageCache.getOrCreate(
      packageRes.version,
      methodToCache,
      duration
    );
  }

  /**
   * Clears all cached package suggestion responses.
   */
  clear(): void {
    // get the provider names
    const providerNames = Object.keys(this.providerMaps);

    // reset each provider map
    providerNames.forEach(
      k => this.providerMaps[k] = {}
    );
  }

}