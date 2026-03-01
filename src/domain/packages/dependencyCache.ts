import { type ICache, MemoryCache } from '#domain/caching';
import type { PackageDependency } from "#domain/packages";
import type { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from "@esm-test/guards";

/**
 * Caches parsed dependencies for package files, keyed by provider.
 */
export class DependencyCache {

  /**
   * Internal maps of caches for each provider.
   */
  readonly providerMaps: KeyDictionary<ICache<PackageDependency[]>> = {};

  /**
   * Initializes a new instance of the DependencyCache class.
   * @param providerNames List of provider names to initialize caches for.
   */
  constructor(providerNames: Array<string>) {
    throwUndefinedOrNull("providerNames", providerNames);

    providerNames.forEach(
      k => this.providerMaps[k] = new MemoryCache<PackageDependency[]>(`${k}-dependency-cache`)
    );
  }

  /**
   * Gets cached dependencies for a specific provider and package file.
   * @param providerName The name of the provider.
   * @param packageFilePath The path to the package file.
   * @returns The list of cached dependencies, or undefined if not found.
   */
  get(providerName: string, packageFilePath: string): PackageDependency[] | undefined {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // get the cache entry
    return packageFilesCache.get(packageFilePath);
  }

  /**
   * Gets all cached package file paths for a specific provider.
   * @param providerName The name of the provider.
   * @returns An array of file paths.
   */
  getFilePaths(providerName: string): string[] {
    const packageFilesCache = this.providerMaps[providerName];
    return [...packageFilesCache.keys()];
  }

  /**
   * Caches dependencies for a specific provider and package file.
   * @param providerName The name of the provider.
   * @param packageFilePath The path to the package file.
   * @param dependencies The list of dependencies to cache.
   */
  set(providerName: string, packageFilePath: string, dependencies: PackageDependency[]): void {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // set the cache entry
    packageFilesCache.set(packageFilePath, dependencies);
  }

  /**
   * Removes a cache entry for a specific provider and package file.
   * @param providerName The name of the provider.
   * @param packageFilePath The path to the package file.
   */
  remove(providerName: string, packageFilePath: string) {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // remove the cache entry
    packageFilesCache.remove(packageFilePath);
  }

  /**
   * Clears all cached dependencies for all providers.
   */
  clear(): void {
    // get the provider names
    const providerNames = Object.keys(this.providerMaps);

    // clear each provider cache
    providerNames.forEach(
      k => this.providerMaps[k].clear()
    );
  }

  /**
   * Helper method to search multiple dependency caches for a specific package file.
   * @param providerName The name of the provider.
   * @param packageFilePath The path to the package file.
   * @param dependencyCaches The list of caches to search.
   * @returns The list of dependencies if found in any cache, otherwise an empty array.
   */
  static getDependenciesWithFallback(
    providerName: string,
    packageFilePath: string,
    ...dependencyCaches: DependencyCache[]
  ): PackageDependency[] {
    for (const cache of dependencyCaches) {
      const dependencies = cache.get(providerName, packageFilePath);
      if (dependencies) return dependencies;
    }
    return [];
  }

}