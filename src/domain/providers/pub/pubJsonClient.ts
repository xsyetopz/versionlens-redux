import type { IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { PubConfig, PubJsonClientResponse, PubPackageResponse } from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PubJsonClient {

  constructor(
    readonly config: PubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('logger', logger);
  }

  async get(url: string): Promise<PubJsonClientResponse> {
    const jsonResponse = await this.jsonClient.get(url) as PubPackageResponse;

    // reduce the dataset
    const data = {
      versions: jsonResponse.data.versions
        .filter(pkg => !pkg.retracted)
        .map(x => x.version)
    }

    // return the response
    return { ...jsonResponse, data }
  }

}