import type { IAuthorizer } from '#domain/authorization';
import {
  type HttpClientOptions,
  type HttpClientResponse,
  type IHttpClient,
  ClientResponseSource,
  HttpClientRequestMethods
} from '#domain/clients';
import type { IXhrRequest, IXhrResponse } from '#domain/clients/requestLight';
import { type KeyStringDictionary, createUrl } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { parse } from 'node:url';

export const httpClientDefaultHeaders = {
  'user-agent': 'vscode-versionlens (gitlab.com/versionlens/vscode-versionlens)'
};

export class RequestLightClient implements IHttpClient {

  constructor(
    readonly requestLight: IXhrRequest,
    readonly authorizer: IAuthorizer,
    readonly options: HttpClientOptions
  ) {
    throwUndefinedOrNull('requestLight', requestLight);
    throwUndefinedOrNull('authorizer', authorizer);
    throwUndefinedOrNull('options', options);
  }

  async get(
    baseUrl: string,
    query: KeyStringDictionary = {},
    headers: KeyStringDictionary = {}
  ): Promise<HttpClientResponse> {
    const url = createUrl(baseUrl, query);
    const parsedBaseUrl = parse(baseUrl, false);
    const host = `${parsedBaseUrl.protocol}//${parsedBaseUrl.host}`;
    const shouldAutoAuthorize = !headers.Authorization
      && this.authorizer.urlHasAuthConsent(host);
    const autoAuthHeaders: any = {};

    try {
      if (shouldAutoAuthorize) {
        const authToken = await this.authorizer.getToken(host);
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
          ? await this.authorizer.retryCredentials(host)
          : await this.authorizer.getConsent(host);

        if (consent) return await this.get(baseUrl, query, headers);
      }

      // throw the error response
      const result: HttpClientResponse = {
        source: ClientResponseSource.remote,
        status: errorResponse.status,
        data: errorResponse.responseText,
        rejected: true
      };

      throw result;
    }
  }

}