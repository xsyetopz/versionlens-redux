import { type IExpiryCache, MemoryExpiryCache } from '#domain/caching';
import type { PackageClientResponse, PackageResource } from "#domain/packages";
import type { KeyDictionary, TAsyncFunction } from '#domain/utils';
import { throwUndefinedOrNull } from "@esm-test/guards";

export class PackageCache {

  readonly providerMaps: KeyDictionary<KeyDictionary<IExpiryCache>> = {};

  constructor(providerNames: Array<string>) {
    throwUndefinedOrNull("providerNames", providerNames);

    providerNames.forEach(
      k => this.providerMaps[k] = {}
    );
  }

  getOrCreate(
    providerName: string,
    packageRes: PackageResource,
    methodToCache: TAsyncFunction<PackageClientResponse>,
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

  clear(): void {
    // get the provider names
    const providerNames = Object.keys(this.providerMaps);

    // reset each provider map
    providerNames.forEach(
      k => this.providerMaps[k] = {}
    );
  }

}