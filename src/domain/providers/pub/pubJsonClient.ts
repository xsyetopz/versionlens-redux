import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  PubApiPackageResult,
  PubConfig,
  PubJsonClientResponse
} from '#domain/providers/pub';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PubJsonClient {

  constructor(
    readonly config: PubConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(url: string): Promise<PubJsonClientResponse> {
    // check cache
    const cached = this.requestCache.get<PubJsonClientResponse>(
      url,
      this.config.caching.duration
    );
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<PubApiPackageResult>(url);
    // reduce
    const data = {
      versions: jsonResponse.data.versions
        .filter(pkg => !pkg.retracted)
        .map(x => x.version)
    }
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(url, result);
  }

}