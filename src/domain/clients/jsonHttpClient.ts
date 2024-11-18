import type {
  HttpClientResponse,
  IHttpClient,
  IJsonHttpClient,
  JsonClientResponse
} from '#domain/clients';
import type { KeyStringDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export class JsonHttpClient implements IJsonHttpClient {

  constructor(readonly httpClient: IHttpClient) {
    throwUndefinedOrNull("httpClient", httpClient);

    this.httpClient = httpClient;
  }

  get(
    url: string,
    query: KeyStringDictionary = {},
    headers: KeyStringDictionary = {}
  ): Promise<JsonClientResponse> {

    return this.httpClient.get(url, query, headers)
      .then(function (response: HttpClientResponse) {
        return {
          source: response.source,
          status: response.status,
          data: JSON.parse(response.data),
        }
      });

  }

}