import type {
  IHttpClient,
  IJsonHttpClient,
  JsonClientResponse,
  QueryDictionary
} from '#domain/clients';
import type { KeyStringDictionary } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export const defaultHeaders = { 'Accept': 'application/json' };

/**
 * Client for making HTTP requests and parsing the response data as JSON.
 */
export class JsonHttpClient implements IJsonHttpClient {

  /**
   * Initializes a new instance of the JsonHttpClient class.
   * @param httpClient The underlying HTTP client.
   */
  constructor(readonly httpClient: IHttpClient) {
    throwUndefinedOrNull("httpClient", httpClient);
    this.httpClient = httpClient;
  }

  /**
   * Performs a GET request and parses the response data as JSON.
   * @template TData The expected type of the JSON data.
   * @param url The URL to request.
   * @param query Optional query parameters.
   * @param headers Optional HTTP headers.
   * @returns A promise resolving to the JSON client response.
   */
  async get<TData>(
    url: string,
    query: QueryDictionary = {},
    headers: KeyStringDictionary = {}
  ): Promise<JsonClientResponse<TData>> {
    const response = await this.httpClient.get(url, query, { ...defaultHeaders, ...headers })
    return {
      source: response.source,
      status: response.status,
      data: JSON.parse(response.data),
    }
  }

}