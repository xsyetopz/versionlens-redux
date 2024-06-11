import { ICachingOptions } from '#domain/caching';
import { IHttpOptions } from '#domain/http';

export type HttpClientOptions = {

    caching: ICachingOptions,

    http: IHttpOptions,

}