import type { IExpiryCache } from '#domain/caching';
import { type IHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { XmlDoc } from '#domain/parsers';
import type { PypiConfig, PypiHttpClientResponse } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from the PyPi registry.
 */
export class PypiHttpClient {

  /**
   * Initializes a new instance of the PypiHttpClient class.
   * @param config The configuration for the PyPi provider.
   * @param httpClient The HTTP client for making requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: PypiConfig,
    readonly httpClient: IHttpClient,
    readonly requestCache: IExpiryCache<PypiHttpClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package from the PyPi RSS feed.
   * @param packageName The name of the package.
   * @returns A promise resolving to the PyPi HTTP client response.
   */
  async get(packageName: string): Promise<PypiHttpClientResponse> {
    const url = this.config.apiUrl.replace('{name}', packageName);
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const httpResponse = await this.httpClient.get(url);
    // reduce
    const xmlDoc = new XmlDoc()
    xmlDoc.parse(httpResponse.data)
    const data = xmlDoc.findExactPaths("rss.channel.item.title")
      .map(x => x.text!);
    // cache and return
    const result: PypiHttpClientResponse = { ...httpResponse, data };
    return this.requestCache.set(url, result);
  }

}