import type { IExpiryCache } from '#domain/caching';
import { type IHttpClient, ClientResponseSource } from '#domain/clients';
import type { ILogger } from '#domain/logging';
import { XmlDoc } from '#domain/parsers';
import type { PypiConfig, PypiHttpClientResponse } from '#domain/providers/pypi';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class PypiHttpClient {

  constructor(
    readonly config: PypiConfig,
    readonly httpClient: IHttpClient,
    readonly requestCache: IExpiryCache<PypiHttpClientResponse>,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull('config', config);
    throwUndefinedOrNull('httpClient', httpClient);
    throwUndefinedOrNull('requestCache', requestCache);
    throwUndefinedOrNull('logger', logger);
  }

  async get(packageName: string): Promise<PypiHttpClientResponse> {
    const url = this.config.apiUrl.replace('{name}', packageName);
    // check cache
    const cached = this.requestCache.get(url, this.config.caching.duration);
    if (cached) return { ...cached, source: ClientResponseSource.cache };
    // fetch
    const httpResponse = await this.httpClient.get(url);
    // reduce
    const xmlDoc = new XmlDoc()
    xmlDoc.parse(httpResponse.data)
    const data = xmlDoc.findExactPaths("rss.channel.item.title")
      .map(x => x.text!);
    // cache and return
    const result: PypiHttpClientResponse = { ...httpResponse, data };
    return this.requestCache.set(url, result);
  }

}