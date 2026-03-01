import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DenoConfig, JsrApiResult, JsrClientResponse } from '#domain/providers/deno';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from JSR.
 */
export class JsrClient {

  /**
   * Initializes a new instance of the JsrClient class.
   * @param config The configuration for the Deno provider.
   * @param jsonClient The HTTP client for JSON requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: DenoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<JsrClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull("logger", logger);
  }

  /**
   * Fetches version information for a given package from JSR.
   * @param packageName The name of the package.
   * @returns A promise resolving to the package versions response.
   */
  async get(packageName: string): Promise<JsrClientResponse> {
    const url = `https://jsr.io/${packageName}/meta.json`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<JsrApiResult>(url);
    // reduce
    const versions = Object.keys(jsonResponse.data.versions)
      .filter(k => !jsonResponse.data.versions[k].yanked)
    // cache and return
    const result = { ...jsonResponse, data: versions };
    return this.requestCache.set(url, result);
  }

}