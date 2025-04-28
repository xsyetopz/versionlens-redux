import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  CargoConfig,
  CratesPackageVersionEntry,
  CratesPackageVersionsResponse
} from '#domain/providers/cargo';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class CratesClient {

  constructor(
    readonly config: CargoConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<CratesPackageVersionsResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(packageName: string): Promise<CratesPackageVersionsResponse> {
    const url = `${this.config.apiUrl}${packageName}/versions`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get(url) as CratesPackageVersionsResponse;
    // reduce
    const data = {
      versions: jsonResponse.data.versions.map<CratesPackageVersionEntry>(
        x => ({ num: x.num, yanked: x.yanked })
      )
    };
    // cache and return
    const result = { ...jsonResponse, data };
    return this.requestCache.set(url, result);
  }

}