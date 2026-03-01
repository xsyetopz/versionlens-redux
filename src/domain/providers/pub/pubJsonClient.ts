import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  PubApiPackageResult,
  PubConfig,
  PubJsonClientResponse
} from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from the Pub registry.
 */
export class PubJsonClient {

  /**
   * Initializes a new instance of the PubJsonClient class.
   * @param config The configuration for the Pub provider.
   * @param jsonClient The HTTP client for JSON requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: PubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<PubJsonClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package URL from the Pub registry.
   * @param url The API URL for the package.
   * @returns A promise resolving to the Pub JSON client response.
   */
  async get(url: string): Promise<PubJsonClientResponse> {
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<PubApiPackageResult>(url);
    // reduce
    const data = {
      versions: jsonResponse.data.versions
        .filter(pkg => !pkg.retracted)
        .map(x => x.version)
    }
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(url, result);
  }

}