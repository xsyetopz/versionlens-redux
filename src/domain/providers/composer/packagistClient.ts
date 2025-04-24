import type { IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { ComposerConfig, PackagistPackagesResponse, PackagistVersionEntry } from '#domain/providers/composer';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PackagistClient {

  constructor(
    readonly config: ComposerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async get(packageName: string): Promise<PackagistPackagesResponse> {
    const url = `${this.config.apiUrl}${packageName}.json`;
    const jsonResponse = await this.jsonClient.get(url) as PackagistPackagesResponse;
    if (jsonResponse.rejected) return jsonResponse as any;
    // reduce the dataset
    let packageData = jsonResponse.data.packages[packageName]

    const data = {
      packages: {
        [packageName]:
          url.includes('/p/')
            ? Object.keys(packageData).map(version => ({ version }))
            : packageData.map<PackagistVersionEntry>(x => ({ version: x.version }))
      }
    };
    // return the response
    return { ...jsonResponse, data };
  }

}