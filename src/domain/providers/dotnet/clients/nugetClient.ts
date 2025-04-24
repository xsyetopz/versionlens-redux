import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DotNetSource, NugetApiResponse, NugetServiceIndexResponse } from '#domain/providers/dotnet';
import { ensureEndSlash } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class NuGetClient {

  constructor(
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async get(packageName: string, [url, ...fallbacks]: string[]): Promise<NugetApiResponse> {
    const packageUrl = ensureEndSlash(url)
      + `${packageName.toLowerCase()}/index.json`;

    try {
      return await this.jsonClient.get(packageUrl) as NugetApiResponse;
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