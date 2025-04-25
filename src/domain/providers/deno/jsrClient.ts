import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type { DenoConfig, JsrApiResult, JsrClientResponse } from '#domain/providers/deno';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class JsrClient {

  constructor(
    readonly config: DenoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull("logger", logger);
  }

  async get(packageName: string): Promise<JsrClientResponse> {
    const url = `https://jsr.io/${packageName}/meta.json`;
    // check cache
    const cached = this.requestCache.get<JsrClientResponse>(
      url,
      this.config.caching.duration
    );
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