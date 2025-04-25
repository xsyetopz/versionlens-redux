import type { IHttpClient } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { XmlDoc } from '#domain/parsers';
import type { PypiConfig, PypiHttpClientResponse } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PypiHttpClient {

  constructor(
    readonly config: PypiConfig,
    readonly httpClient: IHttpClient,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('logger', logger);
  }

  async get(packageName: string): Promise<PypiHttpClientResponse> {
    const url = this.config.apiUrl.replace('{name}', packageName);
    const httpResponse = await this.httpClient.get(url);

    // reduce the dataset
    const xmlDoc = new XmlDoc()
    xmlDoc.parse(httpResponse.data)
    const data = xmlDoc.findExactPaths("rss.channel.item.title")
      .map(x => x.text)

    // return the response
    return { ...httpResponse, data };
  }

}