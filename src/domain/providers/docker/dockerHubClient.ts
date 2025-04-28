import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import {
  DockerConfig,
  DockerHubListClientResponse,
  DockerHubListReposResponse,
  DockerHubListReposResult,
  DockerHubRepository
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DockerHubClient {

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
    const url = this.config.apiUrl
      .replace('{namespace}', namespace)
      .replace('{repository}', repository);
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const results: DockerHubRepository[] = [];
    let jsonResponse: DockerHubListReposResponse;
    let pagedData: DockerHubListReposResult;
    let page = 1;
    do {
      jsonResponse = await this.jsonClient.get(
        url,
        {
          page,
          page_size: 100,
          ordering: 'last_updated'
        }
      );

      pagedData = jsonResponse.data;
      results.push(...pagedData.results);
      page++;
    } while (pagedData.next && page < 4)
    // reduce
    const result = {
      ...jsonResponse,
      data: results
        .filter(x => x.tag_status === 'active')
        .filter(x => !!x.digest)
        .map<DockerHubRepository>(x => ({ name: x.name, digest: x.digest, tag_status: x.tag_status }))
    };
    // cache and return
    return this.requestCache.set(url, result);
  }

}