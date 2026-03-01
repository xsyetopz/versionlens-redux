import type { IExpiryCache } from '#domain/caching';
import { type IJsonHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import type {
  DockerClientResponse,
  DockerConfig,
  DockerRepository
} from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

/**
 * Client for fetching Docker image tag data from the Microsoft Container Registry (MCR).
 */
export class MicrosoftDockerClient {

  /**
   * Initializes a new instance of the MicrosoftDockerClient class.
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
   * Fetches tags and their digests for a given repository from the Microsoft Container Registry.
   * @param repository The name of the repository.
   * @param namespace The namespace for the repository.
   * @returns A promise resolving to the Docker client response.
   */
  async get(repository: string, namespace: string): Promise<DockerClientResponse> {
    const url = `https://mcr.microsoft.com/api/v1/catalog/${namespace}/${repository}/tags?reg=mar`;
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    this.logger.debug('Fetching from {Url}', new URL(url));
    const jsonResponse = await this.jsonClient.get<DockerRepository[]>(url);
    // reduce
    const result = {
      ...jsonResponse,
      data: jsonResponse.data
        .map<DockerRepository>(x => ({ name: x.name, digest: x.digest }))
    };
    // cache and return
    return this.requestCache.set(url, result);
  }

}