import {
  type HttpClientOptions,
  type HttpClientResponse,
  type IHttpClient,
  ClientResponseSource,
  HttpClientRequestMethods,
} from '#domain/clients';
import type { IXhrResponse } from '#domain/clients/requestLight';
import type { ILogger } from '#domain/logging';
import { type KeyStringDictionary, createUrl } from '#domain/utils';
import { throwUndefinedOrNull } from '@esm-test/guards';
import { XHRRequest } from 'request-light';

const defaultHeaders = {
  'user-agent': 'vscode-versionlens (gitlab.com/versionlens/vscode-versionlens)'
};

export class RequestLightClient implements IHttpClient {

  constructor(
    readonly xhr: XHRRequest,
    readonly options: HttpClientOptions,
    readonly logger: ILogger
  ) {
    throwUndefinedOrNull("xhr", xhr);
    throwUndefinedOrNull("options", options);
    throwUndefinedOrNull("logger", logger);
  }

  async get(
    baseUrl: string,
    query: KeyStringDictionary = {},
    headers: KeyStringDictionary = {}
  ): Promise<HttpClientResponse> {
    const url = createUrl(baseUrl, query);

    try {
      // make the request
      const response = await this.xhr({
        url,
        type: HttpClientRequestMethods.get,
        headers: Object.assign({}, headers, defaultHeaders),
        strictSSL: this.options.http.strictSSL
      });

      // return the response
      const result: HttpClientResponse = {
        source: ClientResponseSource.remote,
        status: response.status,
        data: response.responseText,
        rejected: false
      };

      return result;
    } catch (error) {
      const errorResponse = error as IXhrResponse;

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