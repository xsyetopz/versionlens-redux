import type { IExpiryCache } from '#domain/caching';
import { type HttpClientResponse, type IHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  type MavenApiResponse,
  type MavenConfig,
  getVersionsFromPackageXml
} from '#domain/providers/maven';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class MavenHttpClient {

  constructor(
    readonly config: MavenConfig,
    readonly httpClient: IHttpClient,
    readonly requestCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(packageName: string, [repoUrl, ...fallbacks]: string[]): Promise<MavenApiResponse> {
    const [group, artifact] = packageName.split(':');
    const search = group.replaceAll('.', '/') + '/' + artifact
    const url = `${repoUrl}${search}/maven-metadata.xml`;
    // check cache
    const cached = this.requestCache.get<MavenApiResponse>(
      url,
      this.config.caching.duration
    );
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
      const errorResponse = error as HttpClientResponse;

      this.logger.debug(
        "request failed for '{packageName}' from '{resourceUrl}': {error}",
        packageName,
        new URL(url),
        errorResponse
      );

      // retry if 404 and we have more urls to try
      if (errorResponse.status === 404 && fallbacks.length > 0) {
        this.logger.debug(
          "attempting to fetch '{packageName}' from '{url}'",
          packageName,
          new URL(url)
        );
        return this.get(packageName, fallbacks);
      }

      throw error;
    }

  }

}