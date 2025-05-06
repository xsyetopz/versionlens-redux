import { type ICache, MemoryCache } from '#domain/caching';
import type { PackageDependency } from "#domain/packages";
import type { KeyDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from "@esm-test/guards";

export class DependencyCache {

  readonly providerMaps: KeyDictionary<ICache<PackageDependency[]>> = {};

  constructor(providerNames: Array<string>) {
    throwUndefinedOrNull("providerNames", providerNames);

    providerNames.forEach(
      k => this.providerMaps[k] = new MemoryCache<PackageDependency[]>(`${k}-dependency-cache`)
    );
  }

  get(providerName: string, packageFilePath: string): PackageDependency[] | undefined {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // get the cache entry
    return packageFilesCache.get(packageFilePath);
  }

  getFilePaths(providerName: string): string[] {
    const packageFilesCache = this.providerMaps[providerName];
    return [...packageFilesCache.keys()];
  }

  set(providerName: string, packageFilePath: string, dependencies: PackageDependency[]): void {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // set the cache entry
    packageFilesCache.set(packageFilePath, dependencies);
  }

  remove(providerName: string, packageFilePath: string) {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // remove the cache entry
    packageFilesCache.remove(packageFilePath);
  }

  clear(): void {
    // get the provider names
    const providerNames = Object.keys(this.providerMaps);

    // clear each provider cache
    providerNames.forEach(
      k => this.providerMaps[k].clear()
    );
  }

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