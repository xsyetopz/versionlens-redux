import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { ComposerConfig, PackagistPackagesResponse, PackagistVersionEntry } from '#domain/providers/composer';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from Packagist.
 */
export class PackagistClient {

  /**
   * Initializes a new instance of the PackagistClient class.
   * @param config The configuration for the Composer provider.
   * @param jsonClient The HTTP client for JSON requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: ComposerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<PackagistPackagesResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package from Packagist.
   * @param packageName The name of the package.
   * @returns A promise resolving to the package versions response.
   */
  async get(packageName: string): Promise<PackagistPackagesResponse> {
    const url = `${this.config.apiUrl}${packageName}.json`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get(url) as PackagistPackagesResponse;
    // reduce
    let packageData = jsonResponse.data.packages[packageName]
    const data = {
      packages: {
        [packageName]:
          url.includes('/p/')
            ? Object.keys(packageData).map(version => ({ version }))
            : packageData.map<PackagistVersionEntry>(x => ({ version: x.version }))
      }
    };
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(url, result);
  }

}