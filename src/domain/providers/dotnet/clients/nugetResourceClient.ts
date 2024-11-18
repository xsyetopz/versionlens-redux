import type { HttpClientResponse, IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DotNetSource, NugetServiceIndexResponse } from '#domain/providers/dotnet';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class NuGetResourceClient {

  constructor(
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async fetchResource(source: DotNetSource): Promise<string> {
    this.logger.debug("Requesting PackageBaseAddressService from %s", source.url)

    try {
      const response = await this.jsonClient.get(source.url) as NugetServiceIndexResponse;

      const packageBaseAddressServices = response.data.resources
        .filter(res => res["@type"].indexOf('PackageBaseAddress') > -1);

      // just take one service for now
      const foundPackageBaseAddressServices = packageBaseAddressServices[0]["@id"];

      this.logger.debug(
        "Resolved PackageBaseAddressService endpoint: %O",
        foundPackageBaseAddressServices
      );

      return foundPackageBaseAddressServices;
    }
    catch (error) {
      const responseError = error as HttpClientResponse;
      this.logger.error(
        "Could not resolve nuget service index. %O",
        responseError
      )
      return "";
    }
  }

}