import type { IExpiryCache } from '#domain/caching';
import { type IHttpClient, ClientResponseSource, HttpRequestError } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type MavenApiResponse,
  type MavenConfig,
  getVersionsFromPackageXml
} from '#domain/providers/maven';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching package version data from Maven repositories.
 */
export class MavenHttpClient {

  /**
   * Initializes a new instance of the MavenHttpClient class.
   * @param config The configuration for the Maven provider.
   * @param httpClient The HTTP client for making requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: MavenConfig,
    readonly httpClient: IHttpClient,
    readonly requestCache: IExpiryCache<MavenApiResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches version information for a given package from Maven repositories, trying multiple URLs if necessary.
   * @param packageName The name of the package.
   * @param repoUrls The list of repository URLs to try.
   * @returns A promise resolving to the Maven API response.
   */
  async get(packageName: string, [repoUrl, ...fallbacks]: string[]): Promise<MavenApiResponse> {
    const [group, artifact] = packageName.split(':');
    const search = group.replaceAll('.', '/') + '/' + artifact
    const url = `${repoUrl}${search}/maven-metadata.xml`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    try {
      // fetch
      const httpResponse = await this.httpClient.get(url);
      // reduce
      const data = getVersionsFromPackageXml(httpResponse.data);
      // cache and return
      const result = { ...httpResponse, data };
      return this.requestCache.set(url, result);
    } catch (error) {
      if (error instanceof HttpRequestError) {
        this.logger.debug(
          "request failed for '{packageName}' from '{resourceUrl}': {error}",
          packageName,
          new URL(url),
          error
        );

        // retry if 404 and we have more urls to try
        if (error.status === 404 && fallbacks.length > 0) {
          this.logger.debug(
            "attempting to fetch '{packageName}' from '{url}'",
            packageName,
            new URL(url)
          );
          return this.get(packageName, fallbacks);
        }
      }

      throw error;
    }
  }

}