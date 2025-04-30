import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  DockerConfig,
  DockerHubListClientResponse,
  DockerHubRepository
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class MicrosoftHubClient {

  constructor(
    readonly config: DockerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<DockerHubListClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(repository: string, namespace: string = 'library'): Promise<DockerHubListClientResponse> {
    const url = `https://mcr.microsoft.com/api/v1/catalog/${namespace}/${repository}/tags?reg=mar`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const jsonResponse = await this.jsonClient.get<DockerHubRepository[]>(url);
    // reduce
    const result = {
      ...jsonResponse,
      data: jsonResponse.data
        .map<DockerHubRepository>(x => ({ name: x.name, digest: x.digest, tag_status: 'active' }))
    };
    // cache and return
    return this.requestCache.set(url, result);
  }

}