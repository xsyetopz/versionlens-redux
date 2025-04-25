import type { IExpiryCache } from '#domain/caching';
import { type HttpClientResponse, type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  DotNetConfig,
  DotNetSource,
  NugetApiResponse,
  NugetServiceIndexResponse
} from '#domain/providers/dotnet';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class NuGetClient {

  constructor(
    readonly config: DotNetConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(packageName: string, [resourceUrl, ...fallbacks]: string[]): Promise<NugetApiResponse> {
    const url = ensureEndSlash(resourceUrl)
      + `${packageName.toLowerCase()}/index.json`;
    // check cache
    const cached = this.requestCache.get<NugetApiResponse>(
      url,
      this.config.caching.duration
    );
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    try {
      const result = await this.jsonClient.get(url) as NugetApiResponse;
      // cache and return
      return this.requestCache.set(url, result);
    } catch (error) {
      const errorResponse = error as HttpClientResponse;

      this.logger.debug(
        "request failed for '{packageName}' from '{resourceUrl}': {error}",
        packageName,
        new URL(resourceUrl),
        errorResponse
      );

      // retry if 404 and we have more urls to try
      if (errorResponse.status === 404 && fallbacks.length > 0) {
        this.logger.debug(
          "attempting to fetch '{packageName}' from '{url}'",
          packageName,
          new URL(fallbacks[0])
        );
        return this.get(packageName, fallbacks);
      }

      throw error;
    }
  }

  async fetchResource(source: DotNetSource): Promise<string> {
    this.logger.debug(
      "Requesting PackageBaseAddressService from {url}",
      new URL(source.url)
    );

    try {
      const response = await this.jsonClient.get(source.url) as NugetServiceIndexResponse;

      const packageBaseAddressServices = response.data.resources
        .filter(res => res["@type"].indexOf('PackageBaseAddress') > -1);

      // just take one service
      const url = packageBaseAddressServices[0]["@id"];

      this.logger.debug(
        "Resolved PackageBaseAddressService endpoint: {url}",
        new URL(url)
      );

      return url;
    }
    catch (error) {
      const responseError = error as HttpClientResponse;
      this.logger.error(
        "Could not resolve nuget service index {url}. {error}",
        new URL(source.url),
        responseError
      );
      return "";
    }
  }

}