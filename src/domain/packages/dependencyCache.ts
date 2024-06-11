import { throwUndefinedOrNull } from "@esm-test/guards";
import { ICache, MemoryCache } from '#domain/caching';
import { PackageDependency } from "#domain/packages";
import { KeyDictionary } from '#domain/utils';

export class DependencyCache {

  readonly providerMaps: KeyDictionary<ICache> = {};

  constructor(providerNames: Array<string>) {
    throwUndefinedOrNull("providerNames", providerNames);

    providerNames.forEach(
      k => this.providerMaps[k] = new MemoryCache(`${k}-dependency-cache`)
    );
  }

  get(providerName: string, packageFilePath: string): PackageDependency[] {
    // get the package file cache for the provider
    const packageFilesCache = this.providerMaps[providerName];

    // get the cache entry
    return packageFilesCache.get(packageFilePath);
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
  }

}