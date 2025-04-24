import type { IJsonHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DenoConfig, JsrApiResponse, JsrClientResponse } from '#domain/providers/deno';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class JsrClient {

  constructor(
    readonly config: DenoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async get(packageName: string): Promise<JsrClientResponse> {
    const url = `https://jsr.io/${packageName}/meta.json`;

    const jsonResponse = await this.jsonClient.get(url) as JsrApiResponse;
    if (jsonResponse.rejected) return jsonResponse as any;

    // reduce the dataset
    const versions = Object.keys(jsonResponse.data.versions)
      .filter(k => !jsonResponse.data.versions[k].yanked)

    // return the response
    return { ...jsonResponse, data: versions };
  }

}