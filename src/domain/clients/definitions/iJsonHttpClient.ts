import { HttpClientRequestMethods, IHttpClient, JsonClientResponse } from '#domain/clients';
import { KeyStringDictionary } from '#domain/utils';

export interface IJsonHttpClient {

  httpClient: IHttpClient;

  request: (
    method: HttpClientRequestMethods,
    url: string,
    query: KeyStringDictionary,
    headers: KeyStringDictionary,
  ) => Promise<JsonClientResponse>;

}