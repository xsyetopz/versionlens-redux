import type { IAuthorizer } from '#domain/authorization';
import {
  type HttpClientOptions,
  type HttpClientResponse,
  type IHttpClient,
  type QueryDictionary,
  ClientResponseSource,
  HttpClientRequestMethods,
  HttpRequestError,
} from '#domain/clients';
import type { IXhrRequest, IXhrResponse } from '#domain/clients/requestLight';
import { type KeyStringDictionary, createUrl } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';

export const httpClientDefaultHeaders = {
  'user-agent': 'vscode-versionlens (gitlab.com/versionlens/vscode-versionlens)'
};

/**
 * Client for making HTTP requests using the request-light library, with authorization support.
 */
export class RequestLightClient implements IHttpClient {

  /**
   * Initializes a new instance of the RequestLightClient class.
   * @param requestLight The low-level XHR request client.
   * @param authorizer The authorizer for handling credentials.
   * @param options HTTP client options.
   */
  constructor(
    readonly requestLight: IXhrRequest,
    readonly authorizer: IAuthorizer,
    readonly options: HttpClientOptions
  ) {
    throwUndefinedOrNull('requestLight', requestLight);
    throwUndefinedOrNull('authorizer', authorizer);
    throwUndefinedOrNull('options', options);
  }

  /**
   * Performs a GET request using request-light.
   * @param baseUrl The base URL for the request.
   * @param query Optional query parameters.
   * @param headers Optional HTTP headers.
   * @returns A promise resolving to the HTTP client response.
   */
  async get(
    baseUrl: string,
    query: QueryDictionary = {},
    headers: KeyStringDictionary = {}
  ): Promise<HttpClientResponse> {
    const url = createUrl(baseUrl, query);
    const authUrl = this.authorizer.getAuthorizationUrl(baseUrl);
    const shouldAutoAuthorize = !headers.Authorization
      && this.authorizer.hasAuthorizationUrl(authUrl);
    const autoAuthHeaders: any = {};

    try {
      if (shouldAutoAuthorize) {
        const authToken = await this.authorizer.getToken(authUrl);
        if (authToken) autoAuthHeaders.Authorization = authToken;
      }

      const request = {
        url,
        type: HttpClientRequestMethods.get,
        headers: {
          ...httpClientDefaultHeaders,
          ...autoAuthHeaders,
          ...headers,
        },
        strictSSL: this.options.http.strictSSL
      };

      const response = await this.requestLight.xhr(request);

      // return the response
      const result: HttpClientResponse = {
        source: ClientResponseSource.remote,
        status: response.status,
        data: response.responseText,
        rejected: false
      };

      return result;
    } catch (error) {
      if (error instanceof Error) throw error;

      const errorResponse = error as IXhrResponse;

      // retry when the status is 401
      if (errorResponse.status === 401) {
        const consent = shouldAutoAuthorize
          ? await this.authorizer.retryCredentials(authUrl)
          : await this.authorizer.getCredentials(authUrl, baseUrl);

        if (consent) return await this.get(baseUrl, query, headers);
      }

      // throw a request error
      throw new HttpRequestError(
        ClientResponseSource.remote,
        errorResponse.status,
        errorResponse.responseText,
      )
    }
  }

}