import { HttpClientRequestMethods, HttpClientResponse } from '#domain/clients';
import { KeyStringDictionary } from 'domain/utils';

export interface THttpClientRequestFn {
  (
    method: HttpClientRequestMethods,
    url: string,
    query: KeyStringDictionary,
    headers: KeyStringDictionary,
  ): Promise<HttpClientResponse>;
}

export interface IHttpClient {

  request: THttpClientRequestFn;

}
