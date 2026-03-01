import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DubApiResult, DubConfig, DubJsonClientResponse } from '#domain/providers/dub';
import { throwUndefinedOrNull } from '@esm-test/guards';

const query = { minimize: 'true' }

/**
 * Client for fetching package version data from the Dub registry.
 */
export class DubJsonClient {

  /**
   * Initializes a new instance of the DubJsonClient class.
   * @param config The configuration for the Dub provider.
   * @param jsonClient The HTTP client for JSON requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: DubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<DubJsonClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package from the Dub registry.
   * @param packageName The name of the package.
   * @returns A promise resolving to the Dub JSON client response.
   */
  async get(packageName: string): Promise<DubJsonClientResponse> {
    const url = `${this.config.apiUrl}${encodeURIComponent(packageName)}/info`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<DubApiResult>(url, query);
    // reduce
    const result = {
      ...jsonResponse,
      data: jsonResponse.data.versions.map(x => x.version)
    }
    // cache and return
    return this.requestCache.set(url, result);
  }

}