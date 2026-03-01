import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource, JsonClientResponse } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  DockerClientResponse,
  DockerConfig,
  DockerRepository
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export type DockerHubRepoResult = DockerRepository & {
  tag_status: 'active' | 'inactive'
}

export type DockerHubListReposResult = {
  count: number
  next: string
  name: string
  results: DockerHubRepoResult[]
}

/**
 * Client for fetching Docker image tag data from Docker Hub.
 */
export class DockerHubClient {

  /**
   * Initializes a new instance of the DockerHubClient class.
   * @param config The configuration for the Docker provider.
   * @param jsonClient The HTTP client for JSON requests.
   * @param requestCache The cache for registry responses.
   * @param logger The logger for this client.
   */
  constructor(
    readonly config: DockerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly requestCache: IExpiryCache<DockerClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('jsonClient', jsonClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  /**
   * Fetches active tags and their digests for a given repository from Docker Hub.
   * @param repository The name of the repository.
   * @param namespace The namespace (defaults to 'library').
   * @returns A promise resolving to the Docker client response.
   */
  async get(repository: string, namespace: string = 'library'): Promise<DockerClientResponse> {
    const url = `https://hub.docker.com/v2/namespaces/${namespace}/repositories/${repository}/tags`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const results: DockerHubRepoResult[] = [];
    let jsonResponse: JsonClientResponse<DockerHubListReposResult>;
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
        .map<DockerRepository>(x => ({ name: x.name, digest: x.digest }))
    };
    // cache and return
    return this.requestCache.set(url, result);
  }

}