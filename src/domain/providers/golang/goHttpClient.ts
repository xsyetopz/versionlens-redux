import type { IExpiryCache } from '#domain/caching';
import { type IHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { GoApiClientResponse, GoConfig } from '#domain/providers/golang';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from a Go proxy or registry.
 */
export class GoHttpClient {

  /**
   * Initializes a new instance of the GoHttpClient class.
   * @param config The configuration for the Go provider.
   * @param httpClient The HTTP client for making requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: GoConfig,
    readonly httpClient: IHttpClient,
    readonly requestCache: IExpiryCache<GoApiClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package.
   * @param packageName The name of the package.
   * @returns A promise resolving to the Go API client response.
   */
  async get(packageName: string): Promise<GoApiClientResponse> {
    const url = this.config.apiUrl.replace('{base-module}', packageName.toLowerCase());
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const httpResponse = await this.httpClient.get(url);
    // reduce
    const data = {
      versions: httpResponse.data.split('\n').filter(x => !!x)
    };
    // cache and return
    const result = { ...httpResponse, data };
    return this.requestCache.set(url, result);
  }

}