import type { IJsonHttpClient, JsonClientResponse } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { DockerApiResponse, DockerApiTagResult, DockerConfig } from '#domain/providers/docker';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class DockerHubClient {

  constructor(
    readonly config: DockerConfig,
    readonly jsonClient: IJsonHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("config", config);
    throwUndefinedOrNull("jsonClient", jsonClient);
    throwUndefinedOrNull("logger", logger);
  }

  async get(repository: string, namespace: string = 'library'): Promise<JsonClientResponse> {
    const url = this.config.apiUrl
      .replace('{namespace}', namespace)
      .replace('{repository}', repository);

    const results: DockerApiTagResult[] = [];
    let jsonResponse: JsonClientResponse;
    let pagedData: DockerApiResponse<DockerApiTagResult>;
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
      if (jsonResponse.rejected) return jsonResponse;

      pagedData = jsonResponse.data as DockerApiResponse<DockerApiTagResult>;
      results.push(...pagedData.results);
      page++;
    } while (pagedData.next && page < 4)

    return {
      ...jsonResponse,
      data: results
        .filter(x => x.tag_status === 'active')
        .filter(x => !!x.digest)
    };
  }

}